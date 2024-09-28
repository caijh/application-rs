use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sort {
    pub column: String,
    pub order: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(rename = "pageNo")]
    pub page_no: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
    pub sorts: Option<Vec<Sort>>,
}
