use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuEntry {
    pub date: String,
    pub comida_id: Option<String>,
    pub comida_name: Option<String>,
    pub cena_id: Option<String>,
    pub cena_name: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetMenuParams {
    pub date: String,
    pub comida_id: Option<String>,
    pub comida_name: Option<String>,
    pub cena_id: Option<String>,
    pub cena_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListMenusParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}