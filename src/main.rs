use async_trait::async_trait;
use rust_mcp_schema::{
    CompleteResult, ListResourceTemplatesResult, ListResourcesResult, ReadResourceRequestParams,
    ReadResourceResult,
};
use rust_mcp_sdk::mcp_server::{
    hyper_server, server_runtime, HyperServerOptions, ServerHandler, ToMcpServerHandler,
    McpServerOptions, ServerRuntime,
};
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{
    schema_utils::CallToolError, CallToolRequestParams, CallToolResult, Implementation,
    InitializeResult, ListToolsResult, ProtocolVersion, ServerCapabilities, ServerCapabilitiesTools,
    RpcError, PaginatedRequestParams, CompleteRequestParams, TextContent,
};
use rust_mcp_sdk::{StdioTransport, TransportOptions, McpServer, tool_box};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;

mod db;
mod models;

use db as db_module;
use models::*;

pub struct AppState {
    pub pool: SqlitePool,
}

pub struct MenuServerHandler {
    pub state: Arc<Mutex<AppState>>,
}

#[async_trait]
impl ServerHandler for MenuServerHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: MenuTools::tools(),
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_params = MenuTools::try_from(params).map_err(CallToolError::new)?;
        let state = self.state.clone();
        match tool_params {
            MenuTools::SetMenu(tool) => tool.call_tool(state).await,
            MenuTools::GetMenu(tool) => tool.call_tool(state).await,
            MenuTools::ListMenus(tool) => tool.call_tool(state).await,
            MenuTools::ClearDay(tool) => tool.call_tool(state).await,
        }
    }

    async fn handle_list_resources_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListResourcesResult, RpcError> {
        Ok(ListResourcesResult { meta: None, next_cursor: None, resources: vec![] })
    }

    async fn handle_list_resource_templates_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListResourceTemplatesResult, RpcError> {
        Ok(ListResourceTemplatesResult { meta: None, next_cursor: None, resource_templates: vec![] })
    }

    async fn handle_read_resource_request(
        &self,
        _params: ReadResourceRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ReadResourceResult, RpcError> {
        Err(RpcError::method_not_found())
    }

    async fn handle_complete_request(
        &self,
        _params: CompleteRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CompleteResult, RpcError> {
        Err(RpcError::method_not_found())
    }
}

fn build_server_details() -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: "MCP Menu Server".into(),
            version: "0.1.0".into(),
            title: Some("MCP Menu Server".into()),
            description: Some("Assign recipes to daily lunch and dinner menus.".into()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            resources: None,
            completions: None,
            tasks: None,
            ..Default::default()
        },
        meta: None,
        instructions: Some("Use this server to manage daily menus. Each day can have a lunch (comida) recipe and a dinner (cena) recipe assigned.".into()),
        protocol_version: ProtocolVersion::V2025_11_25.into(),
    }
}

#[mcp_tool(
    name = "set_menu",
    description = "Assign recipes to a specific date's lunch and/or dinner."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SetMenu {
    /// Date in YYYY-MM-DD format
    date: String,
    /// Recipe UUID for lunch (comida)
    comida_id: Option<String>,
    /// Recipe title for lunch (comida)
    comida_name: Option<String>,
    /// Recipe UUID for dinner (cena)
    cena_id: Option<String>,
    /// Recipe title for dinner (cena)
    cena_name: Option<String>,
}

impl SetMenu {
    pub async fn call_tool(&self, state: Arc<Mutex<AppState>>) -> Result<CallToolResult, CallToolError> {
        let state = state.lock().await;
        let pool = &state.pool;

        let params = SetMenuParams {
            date: self.date.clone(),
            comida_id: self.comida_id.clone(),
            comida_name: self.comida_name.clone(),
            cena_id: self.cena_id.clone(),
            cena_name: self.cena_name.clone(),
        };

        let entry = db_module::set_menu(pool, params)
            .await
            .map_err(|e| CallToolError::from_message(e.to_string()))?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&entry).unwrap_or_default(),
        )]))
    }
}

#[mcp_tool(
    name = "get_menu",
    description = "Get the menu for a specific date."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetMenu {
    /// Date in YYYY-MM-DD format
    date: String,
}

impl GetMenu {
    pub async fn call_tool(&self, state: Arc<Mutex<AppState>>) -> Result<CallToolResult, CallToolError> {
        let state = state.lock().await;
        let pool = &state.pool;

        let entry = db_module::get_menu(pool, &self.date)
            .await
            .map_err(|e| CallToolError::from_message(e.to_string()))?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&entry).unwrap_or_default(),
        )]))
    }
}

#[mcp_tool(
    name = "list_menus",
    description = "List menus in a date range. Defaults to the next 7 days."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ListMenus {
    /// Start date in YYYY-MM-DD format (default: today)
    start_date: Option<String>,
    /// End date in YYYY-MM-DD format (default: 7 days from start)
    end_date: Option<String>,
}

impl ListMenus {
    pub async fn call_tool(&self, state: Arc<Mutex<AppState>>) -> Result<CallToolResult, CallToolError> {
        let state = state.lock().await;
        let pool = &state.pool;

        let params = ListMenusParams {
            start_date: self.start_date.clone(),
            end_date: self.end_date.clone(),
        };

        let entries = db_module::list_menus(pool, params)
            .await
            .map_err(|e| CallToolError::from_message(e.to_string()))?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&entries).unwrap_or_default(),
        )]))
    }
}

#[mcp_tool(
    name = "clear_day",
    description = "Delete the menu entry for a specific date."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ClearDay {
    /// Date in YYYY-MM-DD format
    date: String,
}

impl ClearDay {
    pub async fn call_tool(&self, state: Arc<Mutex<AppState>>) -> Result<CallToolResult, CallToolError> {
        let state = state.lock().await;
        let pool = &state.pool;

        let deleted = db_module::clear_day(pool, &self.date)
            .await
            .map_err(|e| CallToolError::from_message(e.to_string()))?;

        let msg = if deleted {
            format!("Menu entry for {} has been deleted.", self.date)
        } else {
            format!("No menu entry found for {}.", self.date)
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(msg)]))
    }
}

tool_box!(MenuTools, [SetMenu, GetMenu, ListMenus, ClearDay]);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let db_path = std::env::var("MENUS_DB_PATH").unwrap_or_else(|_| "menus.db".to_string());

    let pool = match db_module::create_pool(&db_path).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to create database pool: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = db_module::init_db(&pool).await {
        tracing::error!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--clear" {
        sqlx::query("DELETE FROM menu_entries")
            .execute(&pool)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to clear menus: {}", e);
                std::process::exit(1);
            });
        tracing::info!("All menu entries have been deleted.");
        return;
    }

    let state = Arc::new(Mutex::new(AppState { pool }));

    let transport = std::env::var("MCP_TRANSPORT").unwrap_or_else(|_| "stdio".to_string());

    match transport.as_str() {
        "stdio" => run_stdio(state).await,
        "http" => run_http(state).await,
        _ => {
            eprintln!("Unknown transport: {}. Use 'stdio' or 'http'", transport);
            std::process::exit(1);
        }
    }
}

async fn run_stdio(state: Arc<Mutex<AppState>>) {
    tracing::info!("Starting MCP Menu server (stdio transport)");

    let server_details = build_server_details();
    let transport = match StdioTransport::new(TransportOptions::default()) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create transport: {}", e);
            std::process::exit(1);
        }
    };

    let handler = MenuServerHandler { state };

    let server: Arc<ServerRuntime> = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
        message_observer: None,
    });

    if let Err(e) = server.start().await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn run_http(state: Arc<Mutex<AppState>>) {
    let port: u16 = std::env::var("MCP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3020);

    let host = std::env::var("MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    tracing::info!(
        "Starting MCP Menu server (HTTP transport) on {}:{}",
        host,
        port
    );

    let server_details = build_server_details();
    let handler = MenuServerHandler { state };

    let server = hyper_server::create_server(
        server_details,
        handler.to_mcp_server_handler(),
        HyperServerOptions {
            host,
            port,
            ..Default::default()
        },
    );

    if let Err(e) = server.start().await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}