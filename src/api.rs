use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub cache: ApiCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCache {
    //todo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEntries<T> {
    pub entries: Vec<T>,
}

pub type EntriesOf<T> = ApiResponse<ApiEntries<T>>;
