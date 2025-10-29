use linefeed::chumsky::Parser as _;
use linefeed::grammar::lexer::Token;
use tower_lsp::lsp_types::*;

/// Convert byte offset to (line, column) position
pub fn byte_offset_to_position(source: &str, offset: usize) -> (u32, u32) {
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
pub fn token_to_semantic_type(token: &Token) -> Option<u32> {
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

/// Generate semantic tokens from source code
pub fn generate_semantic_tokens(source: &str) -> Option<Vec<SemanticToken>> {
    // Parse source with lexer
    let tokens = match linefeed::grammar::lexer::lexer()
        .parse(source)
        .into_output_errors()
    {
        (Some(tokens), errors) if errors.is_empty() => tokens,
        (Some(tokens), _) => tokens, // Return partial tokens even with errors
        (None, _) => return None,    // Failed to parse
    };

    let mut semantic_tokens: Vec<SemanticToken> = vec![];
    let mut prev_line = 0;
    let mut prev_col = 0;

    for spanned_token in tokens {
        let token = &spanned_token.0;
        let span = spanned_token.1;

        // Skip tokens that don't map to semantic types (e.g., punctuation)
        let token_type = match token_to_semantic_type(token) {
            Some(t) => t,
            None => continue,
        };

        // Convert byte offsets to line/column
        let start = span.start;
        let end = span.end;
        let (line, col) = byte_offset_to_position(source, start);
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

    Some(semantic_tokens)
}
