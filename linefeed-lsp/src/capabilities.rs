use tower_lsp::lsp_types::*;

/// Semantic token types legend used by the LSP server
/// Includes all standard LSP semantic token types
pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::TYPE,
    SemanticTokenType::CLASS,
    SemanticTokenType::ENUM,
    SemanticTokenType::INTERFACE,
    SemanticTokenType::STRUCT,
    SemanticTokenType::TYPE_PARAMETER,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::EVENT,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::METHOD,
    SemanticTokenType::MACRO,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::MODIFIER,
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::REGEXP,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::DECORATOR,
];

/// Semantic token modifiers legend used by the LSP server
/// Includes all standard LSP semantic token modifiers
pub const LEGEND_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
    SemanticTokenModifier::STATIC,
    SemanticTokenModifier::DEPRECATED,
    SemanticTokenModifier::ABSTRACT,
    SemanticTokenModifier::ASYNC,
    SemanticTokenModifier::MODIFICATION,
    SemanticTokenModifier::DOCUMENTATION,
    SemanticTokenModifier::DEFAULT_LIBRARY,
];

/// Token type indices corresponding to positions in LEGEND_TYPE
/// These constants should be used instead of magic numbers
/// Note: Some constants are unused in the current lexer-based implementation
/// but will be used when AST-based semantic tokens are added
#[allow(dead_code)]
pub const TOKEN_TYPE_NAMESPACE: u32 = 0;
#[allow(dead_code)]
pub const TOKEN_TYPE_TYPE: u32 = 1;
#[allow(dead_code)]
pub const TOKEN_TYPE_CLASS: u32 = 2;
#[allow(dead_code)]
pub const TOKEN_TYPE_ENUM: u32 = 3;
#[allow(dead_code)]
pub const TOKEN_TYPE_INTERFACE: u32 = 4;
#[allow(dead_code)]
pub const TOKEN_TYPE_STRUCT: u32 = 5;
#[allow(dead_code)]
pub const TOKEN_TYPE_TYPE_PARAMETER: u32 = 6;
#[allow(dead_code)]
pub const TOKEN_TYPE_PARAMETER: u32 = 7;
pub const TOKEN_TYPE_VARIABLE: u32 = 8;
#[allow(dead_code)]
pub const TOKEN_TYPE_PROPERTY: u32 = 9;
#[allow(dead_code)]
pub const TOKEN_TYPE_ENUM_MEMBER: u32 = 10;
#[allow(dead_code)]
pub const TOKEN_TYPE_EVENT: u32 = 11;
#[allow(dead_code)]
pub const TOKEN_TYPE_FUNCTION: u32 = 12;
#[allow(dead_code)]
pub const TOKEN_TYPE_METHOD: u32 = 13;
#[allow(dead_code)]
pub const TOKEN_TYPE_MACRO: u32 = 14;
pub const TOKEN_TYPE_KEYWORD: u32 = 15;
#[allow(dead_code)]
pub const TOKEN_TYPE_MODIFIER: u32 = 16;
#[allow(dead_code)]
pub const TOKEN_TYPE_COMMENT: u32 = 17;
pub const TOKEN_TYPE_STRING: u32 = 18;
pub const TOKEN_TYPE_NUMBER: u32 = 19;
pub const TOKEN_TYPE_REGEXP: u32 = 20;
pub const TOKEN_TYPE_OPERATOR: u32 = 21;
#[allow(dead_code)]
pub const TOKEN_TYPE_DECORATOR: u32 = 22;

/// Build the server capabilities for initialization
pub fn build_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        semantic_tokens_provider: Some(
            SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                SemanticTokensRegistrationOptions {
                    text_document_registration_options: TextDocumentRegistrationOptions {
                        document_selector: Some(vec![DocumentFilter {
                            language: Some("lf".to_string()),
                            scheme: Some("file".to_string()),
                            pattern: None,
                        }]),
                    },
                    semantic_tokens_options: SemanticTokensOptions {
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        legend: SemanticTokensLegend {
                            token_types: LEGEND_TYPE.into(),
                            token_modifiers: LEGEND_MODIFIERS.into(),
                        },
                        range: Some(true),
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                    },
                    static_registration_options: StaticRegistrationOptions::default(),
                },
            ),
        ),
        ..ServerCapabilities::default()
    }
}
