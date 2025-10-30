use tower_lsp::LanguageServer;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::backend::Backend;
use crate::capabilities;
use crate::semantic_tokens;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: capabilities::build_server_capabilities(),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // FULL sync mode means we get the entire document
        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_change(params.text_document.uri, change.text).await;
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();

        self.client
            .log_message(MessageType::LOG, "semantic_token_full")
            .await;

        // Get source from cache
        let src = match self.sources.lock().await.get(&uri).cloned() {
            Some(src) => src,
            None => {
                // File not found in cache, return empty tokens
                return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: vec![],
                })));
            }
        };

        // Generate semantic tokens using the semantic_tokens module
        let tokens = semantic_tokens::generate_semantic_tokens(&src).unwrap_or_default();

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }
}

impl Backend {
    /// Process document changes: cache source, parse, compile, and publish diagnostics
    async fn on_change(&self, uri: Url, text: String) {
        let uri_string = uri.to_string();

        // Cache the source
        self.sources.lock().await.insert(uri_string, text.clone());

        // Validate syntax and compilation, publish diagnostics
        let (_symbol_table, diagnostics) = semantic_tokens::safe_parse_and_compile(&text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}
