use std::collections::HashMap;

use linefeed::chumsky::Parser as _;
use linefeed::grammar::ast::{AstValue, Expr, Func, Pattern, Span, Spanned};
use linefeed::grammar::lexer::Token;
use tower_lsp::lsp_types::*;

use crate::capabilities::*;

/// Information about an identifier discovered from AST analysis
#[derive(Debug, Clone)]
struct IdentifierInfo {
    /// The semantic token type for this identifier
    token_type: u32,
    /// Bitset of modifiers for this identifier
    modifiers: u32,
}

impl IdentifierInfo {
    fn new(token_type: u32, modifiers: u32) -> Self {
        Self {
            token_type,
            modifiers,
        }
    }
}

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
        // Keywords
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
        | Token::Bool(_) => Some(TOKEN_TYPE_KEYWORD),

        // Numbers
        Token::Int(_) | Token::Float(_) => Some(TOKEN_TYPE_NUMBER),

        // Strings
        Token::Str(_) => Some(TOKEN_TYPE_STRING),

        // Regex
        Token::Regex(_) => Some(TOKEN_TYPE_REGEXP),

        // Operators
        Token::Op(_) | Token::RangeExclusive | Token::RangeInclusive => Some(TOKEN_TYPE_OPERATOR),

        // Identifiers
        Token::Ident(_) => Some(TOKEN_TYPE_VARIABLE),

        // Control characters - skip (punctuation)
        Token::Ctrl(_) => None,
    }
}

/// Walk the AST and collect identifier information
fn analyze_ast(ast: &Spanned<Expr>) -> HashMap<Span, IdentifierInfo> {
    let mut symbols = HashMap::new();
    visit_expr(ast, &mut symbols);
    symbols
}

/// Visit a pattern and extract identifier information
fn visit_pattern(pattern: &Spanned<Pattern>, symbols: &mut HashMap<Span, IdentifierInfo>, is_declaration: bool) {
    match &pattern.0 {
        Pattern::Ident(_) => {
            if is_declaration {
                symbols.insert(
                    pattern.1,
                    IdentifierInfo::new(TOKEN_TYPE_VARIABLE, MODIFIER_DECLARATION),
                );
            }
        }
        Pattern::Sequence(patterns) => {
            for p in patterns {
                visit_pattern(p, symbols, is_declaration);
            }
        }
        Pattern::Index(target, index) => {
            visit_expr(target, symbols);
            visit_expr(index, symbols);
        }
        Pattern::Value(_) => {
            // Literal patterns (e.g., in match expressions)
            // No identifiers to extract
        }
    }
}

/// Recursively visit an expression and collect identifier information
fn visit_expr(expr: &Spanned<Expr>, symbols: &mut HashMap<Span, IdentifierInfo>) {
    match &expr.0 {
        Expr::Assign(pattern, value) => {
            // Check if this is a function definition
            if let Expr::Value(AstValue::Func(func)) = &value.0 {
                // Mark the function name as a function definition
                if let Pattern::Ident(_) = &pattern.0 {
                    symbols.insert(
                        pattern.1,
                        IdentifierInfo::new(
                            TOKEN_TYPE_FUNCTION,
                            MODIFIER_DECLARATION | MODIFIER_DEFINITION,
                        ),
                    );
                }

                // Visit the function to mark parameters
                visit_func(func, symbols);
            } else {
                // Regular variable assignment
                visit_pattern(pattern, symbols, true);
                visit_expr(value, symbols);
            }
        }

        Expr::Call(func_expr, args) => {
            // If the function expression is a simple identifier, mark it as a function call
            if let Expr::Local(_) = &func_expr.0 {
                symbols.insert(
                    func_expr.1,
                    IdentifierInfo::new(TOKEN_TYPE_FUNCTION, 0),
                );
            } else {
                visit_expr(func_expr, symbols);
            }

            for arg in args {
                visit_expr(arg, symbols);
            }
        }

        Expr::MethodCall(receiver, _method_name, args) => {
            // Note: method_name is a &str without direct span information
            // We'll handle this in the token generation phase
            visit_expr(receiver, symbols);
            for arg in args {
                visit_expr(arg, symbols);
            }
        }

        Expr::For(pattern, iter, body) => {
            // Loop variables are readonly
            if let Pattern::Ident(_) = &pattern.0 {
                symbols.insert(
                    pattern.1,
                    IdentifierInfo::new(
                        TOKEN_TYPE_VARIABLE,
                        MODIFIER_DECLARATION | MODIFIER_READONLY,
                    ),
                );
            } else {
                visit_pattern(pattern, symbols, true);
            }

            visit_expr(iter, symbols);
            visit_expr(body, symbols);
        }

        Expr::Value(AstValue::Func(func)) => {
            visit_func(func, symbols);
        }

        Expr::Local(_) => {
            // Variable reference - already handled by lexer as VARIABLE with no modifiers
        }

        // Recursively visit child expressions
        Expr::Value(_) => {}
        Expr::ParseError => {}

        Expr::List(items) | Expr::Tuple(items) => {
            for item in items {
                visit_expr(item, symbols);
            }
        }

        Expr::Map(entries) => {
            for (key, value) in entries {
                visit_expr(key, symbols);
                visit_expr(value, symbols);
            }
        }

        Expr::Index(target, index) => {
            visit_expr(target, symbols);
            visit_expr(index, symbols);
        }

        Expr::Unary(_, operand) => {
            visit_expr(operand, symbols);
        }

        Expr::Binary(left, _, right) => {
            visit_expr(left, symbols);
            visit_expr(right, symbols);
        }

        Expr::If(cond, then_expr, else_expr) => {
            visit_expr(cond, symbols);
            visit_expr(then_expr, symbols);
            visit_expr(else_expr, symbols);
        }

        Expr::Block(body) | Expr::Return(body) => {
            visit_expr(body, symbols);
        }

        Expr::Sequence(exprs) => {
            for e in exprs {
                visit_expr(e, symbols);
            }
        }

        Expr::While(cond, body) => {
            visit_expr(cond, symbols);
            visit_expr(body, symbols);
        }

        Expr::Break | Expr::Continue => {}

        Expr::ListComprehension(expr, pattern, iter) => {
            // The pattern variables are declarations
            if let Pattern::Ident(_) = &pattern.0 {
                symbols.insert(
                    pattern.1,
                    IdentifierInfo::new(
                        TOKEN_TYPE_VARIABLE,
                        MODIFIER_DECLARATION | MODIFIER_READONLY,
                    ),
                );
            } else {
                visit_pattern(pattern, symbols, true);
            }

            visit_expr(expr, symbols);
            visit_expr(iter, symbols);
        }

        Expr::Match(target, arms) => {
            visit_expr(target, symbols);
            for (pattern_expr, body_expr) in arms {
                visit_expr(pattern_expr, symbols);
                visit_expr(body_expr, symbols);
            }
        }
    }
}

/// Visit a function and mark its parameters
fn visit_func(func: &Func, symbols: &mut HashMap<Span, IdentifierInfo>) {
    // Note: func.args is Vec<&str> without span information
    // We can't directly mark parameters here without spans
    // This will be handled during token generation by matching identifiers in the function signature

    // Visit the function body
    visit_expr(&func.body, symbols);
}

/// State machine for pattern-based token detection
#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenContext {
    /// Normal state
    Normal,
    /// Just saw a 'fn' keyword - next identifier is a function definition
    AfterFn,
    /// Just saw a '.' - next identifier is a method call
    AfterDot,
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

    // Try to parse AST for enhanced semantic analysis
    let symbol_table = match linefeed::parse_tokens(source, &tokens) {
        Ok(ast) => analyze_ast(&ast),
        Err(_) => HashMap::new(), // Fall back to lexer-only if parsing fails
    };

    let mut semantic_tokens: Vec<SemanticToken> = vec![];
    let mut prev_line = 0;
    let mut prev_col = 0;
    let mut context = TokenContext::Normal;

    for spanned_token in tokens {
        let token = &spanned_token.0;
        let span = spanned_token.1;

        // Pattern-based detection for function definitions and method calls
        let pattern_based_info = match (context, token) {
            // Function definition: 'fn' followed by identifier
            (TokenContext::AfterFn, Token::Ident(_)) => {
                Some(IdentifierInfo::new(
                    TOKEN_TYPE_FUNCTION,
                    MODIFIER_DECLARATION | MODIFIER_DEFINITION,
                ))
            }
            // Method call: '.' followed by identifier
            (TokenContext::AfterDot, Token::Ident(_)) => {
                Some(IdentifierInfo::new(TOKEN_TYPE_METHOD, 0))
            }
            _ => None,
        };

        // Update state machine for next iteration
        context = match token {
            Token::Fn => TokenContext::AfterFn,
            Token::Ctrl('.') => TokenContext::AfterDot,
            _ => TokenContext::Normal,
        };

        // Determine token type and modifiers (priority: AST > Pattern > Lexer)
        let (token_type, modifiers) = if let Some(info) = symbol_table.get(&span) {
            // AST-based classification (highest priority)
            (info.token_type, info.modifiers)
        } else if let Some(info) = pattern_based_info {
            // Pattern-based classification (medium priority)
            (info.token_type, info.modifiers)
        } else {
            // Fall back to lexer-based classification (lowest priority)
            match token_to_semantic_type(token) {
                Some(t) => (t, 0),
                None => continue, // Skip tokens that don't map to semantic types
            }
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
            token_modifiers_bitset: modifiers,
        });

        // Update previous position
        prev_line = line;
        prev_col = col;
    }

    Some(semantic_tokens)
}
