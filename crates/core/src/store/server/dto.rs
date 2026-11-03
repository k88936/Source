//! Server payloads: the selected server as it is persisted.

#[derive(Debug, Clone)]
pub struct StoredServer {
    pub base_url: String,
    pub user_id: String,
    pub display_name: Option<String>,
}
