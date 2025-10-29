use std::collections::HashMap;

use tokio::sync::Mutex;
use tower_lsp::Client;

/// The LSP backend managing server state
#[derive(Debug)]
pub struct Backend {
    /// LSP client for communication
    pub client: Client,
    /// Cache of file contents, keyed by URI
    pub sources: Mutex<HashMap<String, String>>,
}

impl Backend {
    /// Create a new Backend instance
    pub fn new(client: Client) -> Self {
        Self {
            client,
            sources: Mutex::new(HashMap::new()),
        }
    }
}
