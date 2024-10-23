use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Error { error: ApiError },
    Success { data: T, cache: ApiCache },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCache {
    //todo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEntries<T> {
    pub entries: Vec<T>,
}

pub type ApiEntriesOf<T> = ApiResponse<ApiEntries<T>>;
