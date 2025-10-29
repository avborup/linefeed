use std::collections::HashMap;

use linefeed::chumsky::Parser as _;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    sources: Mutex<HashMap<String, String>>,
}

impl Backend {
    /// Convert byte offset to (line, column) position
    fn byte_offset_to_position(source: &str, offset: usize) -> (u32, u32) {
        let mut line = 0;
        let mut col = 0;

        for (i, ch) in source.chars().enumerate() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Map lexer token to LSP semantic token type index
    fn token_to_semantic_type(token: &linefeed::grammar::lexer::Token) -> Option<u32> {
        use linefeed::grammar::lexer::Token;

        match token {
            // Keywords -> KEYWORD (index 5)
            Token::If
            | Token::Else
            | Token::Or
            | Token::And
            | Token::Not
            | Token::Xor
            | Token::Fn
            | Token::Return
            | Token::Unless
            | Token::While
            | Token::For
            | Token::In
            | Token::Break
            | Token::Continue
            | Token::Match
            | Token::Null
            | Token::Bool(_) => Some(5),

            // Numbers -> NUMBER (index 4)
            Token::Int(_) | Token::Float(_) => Some(4),

            // Strings/Regex -> STRING (index 2)
            Token::Str(_) | Token::Regex(_) => Some(2),

            // Operators -> OPERATOR (index 6)
            Token::Op(_) | Token::RangeExclusive | Token::RangeInclusive => Some(6),

            // Identifiers -> VARIABLE (index 1)
            Token::Ident(_) => Some(1),

            // Control characters - skip (punctuation)
            Token::Ctrl(_) => None,
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        const LEGEND_TYPE: &[SemanticTokenType] = &[
            SemanticTokenType::FUNCTION,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::STRING,
            SemanticTokenType::COMMENT,
            SemanticTokenType::NUMBER,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::PARAMETER,
        ];

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                inlay_hint_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                execute_command_provider: None,
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("lf".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(true),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
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
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        self.sources.lock().await.insert(uri, text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        // FULL sync mode means we get the entire document
        if let Some(change) = params.content_changes.into_iter().next() {
            self.sources.lock().await.insert(uri, change.text);
        }
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "Linefeed file saved!")
            .await;
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

        // Parse source with lexer
        let tokens = match linefeed::grammar::lexer::lexer()
            .parse(src.as_str())
            .into_output_errors()
        {
            (Some(tokens), errors) if errors.is_empty() => tokens,
            (Some(tokens), _) => tokens, // Return partial tokens even with errors
            (None, _) => {
                // Failed to parse, return empty tokens
                return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: vec![],
                })));
            }
        };

        let mut semantic_tokens: Vec<SemanticToken> = vec![];
        let mut prev_line = 0;
        let mut prev_col = 0;

        for spanned_token in tokens {
            let token = &spanned_token.0;
            let span = spanned_token.1;

            // Skip tokens that don't map to semantic types (e.g., punctuation)
            let token_type = match Self::token_to_semantic_type(token) {
                Some(t) => t,
                None => continue,
            };

            // Convert byte offsets to line/column
            let start = span.start;
            let end = span.end;
            let (line, col) = Self::byte_offset_to_position(&src, start);
            let length = (end - start) as u32;

            // Calculate deltas
            let delta_line = line - prev_line;
            let delta_start = if delta_line == 0 { col - prev_col } else { col };

            semantic_tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset: 0,
            });

            // Update previous position
            prev_line = line;
            prev_col = col;
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        sources: Mutex::new(HashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
