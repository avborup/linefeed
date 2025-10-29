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

pub use token_consts::*;
pub use modifier_consts::*;

/// Token type indices corresponding to positions in LEGEND_TYPE
/// These constants should be used instead of magic numbers
#[allow(dead_code)]
mod token_consts {
    pub const TOKEN_TYPE_NAMESPACE: u32 = 0;
    pub const TOKEN_TYPE_TYPE: u32 = 1;
    pub const TOKEN_TYPE_CLASS: u32 = 2;
    pub const TOKEN_TYPE_ENUM: u32 = 3;
    pub const TOKEN_TYPE_INTERFACE: u32 = 4;
    pub const TOKEN_TYPE_STRUCT: u32 = 5;
    pub const TOKEN_TYPE_TYPE_PARAMETER: u32 = 6;
    pub const TOKEN_TYPE_PARAMETER: u32 = 7;
    pub const TOKEN_TYPE_VARIABLE: u32 = 8;
    pub const TOKEN_TYPE_PROPERTY: u32 = 9;
    pub const TOKEN_TYPE_ENUM_MEMBER: u32 = 10;
    pub const TOKEN_TYPE_EVENT: u32 = 11;
    pub const TOKEN_TYPE_FUNCTION: u32 = 12;
    pub const TOKEN_TYPE_METHOD: u32 = 13;
    pub const TOKEN_TYPE_MACRO: u32 = 14;
    pub const TOKEN_TYPE_KEYWORD: u32 = 15;
    pub const TOKEN_TYPE_MODIFIER: u32 = 16;
    pub const TOKEN_TYPE_COMMENT: u32 = 17;
    pub const TOKEN_TYPE_STRING: u32 = 18;
    pub const TOKEN_TYPE_NUMBER: u32 = 19;
    pub const TOKEN_TYPE_REGEXP: u32 = 20;
    pub const TOKEN_TYPE_OPERATOR: u32 = 21;
    pub const TOKEN_TYPE_DECORATOR: u32 = 22;
}

/// Semantic token modifier bitset values
/// These correspond to positions in LEGEND_MODIFIERS and should be combined using bitwise OR
#[allow(dead_code)]
mod modifier_consts {
    /// Declaration modifier (index 0) - e.g., variable/function declaration
    pub const MODIFIER_DECLARATION: u32 = 1 << 0;
    /// Definition modifier (index 1) - e.g., function body definition
    pub const MODIFIER_DEFINITION: u32 = 1 << 1;
    /// Readonly modifier (index 2) - e.g., constants, loop variables
    pub const MODIFIER_READONLY: u32 = 1 << 2;
    /// Static modifier (index 3)
    pub const MODIFIER_STATIC: u32 = 1 << 3;
    /// Deprecated modifier (index 4)
    pub const MODIFIER_DEPRECATED: u32 = 1 << 4;
    /// Abstract modifier (index 5)
    pub const MODIFIER_ABSTRACT: u32 = 1 << 5;
    /// Async modifier (index 6)
    pub const MODIFIER_ASYNC: u32 = 1 << 6;
    /// Modification modifier (index 7) - e.g., variable being modified
    pub const MODIFIER_MODIFICATION: u32 = 1 << 7;
    /// Documentation modifier (index 8)
    pub const MODIFIER_DOCUMENTATION: u32 = 1 << 8;
    /// Default library modifier (index 9)
    pub const MODIFIER_DEFAULT_LIBRARY: u32 = 1 << 9;
}

/// Build the server capabilities for initialization
pub fn build_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
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
