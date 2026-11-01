//! Lexer for the Apsis programming language.

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Mut,
    Const,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Loop,
    Break,
    Continue,
    Defer,
    Unsafe,
    Import,
    As,
    Pub,
    Struct,
    Enum,
    Match,
    When,
    Is,
    Interface,
    Trait,
    Impl,
    Async,
    Await,

    // Literals
    Identifier(String),
    IntLiteral(i128),
    FloatLiteral(f64),
    StringLiteral(String),
    FStringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // Operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    FloorDiv,      // // (Integer Floor Division)
    Percent,       // %
    Ampersand,     // &
    Pipe,          // |
    Caret,         // ^
    Tilde,         // ~
    Shl,           // <<
    Shr,           // >>
    Pipeline,      // |>
    Question,      // ?

    // Compound Assignments
    PlusEq,        // +=
    MinusEq,       // -=
    StarEq,        // *=
    SlashEq,       // /=
    PercentEq,     // %=
    AmpEq,         // &=
    PipeEq,        // |=
    CaretEq,       // ^=
    ShlEq,         // <<=
    ShrEq,         // >>=

    // Comparison & Logical
    Eq,            // ==
    NotEq,         // !=
    Lt,            // <
    LtEq,          // <=
    Gt,            // >
    GtEq,          // >=
    Assign,        // =
    LogicalAnd,    // &&
    LogicalOr,     // ||
    Bang,          // !

    // Ranges & Arrows
    Range,         // ..
    RangeInclusive,// ..=
    Arrow,         // ->
    FatArrow,      // =>

    // Delimiters
    LParen,        // (
    RParen,        // )
    LBrace,        // {
    RBrace,        // }
    LBracket,      // [
    RBracket,      // ]
    Comma,         // ,
    Colon,         // :
    Semicolon,     // ;
    Dot,           // .
    Hash,          // #

    // Documentation
    DocComment(String), // ###

    // End of File
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _marker: std::marker::PhantomData,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments()?;

        let line = self.line;
        let col = self.col;

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::Eof,
                    line,
                    col,
                })
            }
        };

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier_or_keyword();
        }

        // Numbers (0x, 0b, 0o, 1e2, 123, 3.14)
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // String literals
        if ch == '"' {
            return self.read_string();
        }

        // Char literals
        if ch == '\'' {
            return self.read_char();
        }

        // Operators and Delimiters
        self.advance();
        let kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '?' => TokenKind::Question,
            '~' => TokenKind::Tilde,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::RangeInclusive
                    } else {
                        TokenKind::Range
                    }
                } else {
                    TokenKind::Dot
                }
            }
            '+' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.peek() == Some('/') {
                    self.advance(); // consume second /
                    TokenKind::FloorDiv // // is Integer Floor Division in Apsis!
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PercentEq
                } else {
                    TokenKind::Percent
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Eq
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Assign
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::NotEq
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.peek() == Some('<') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::ShlEq
                    } else {
                        TokenKind::Shl
                    }
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('>') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::ShrEq
                    } else {
                        TokenKind::Shr
                    }
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::LogicalAnd
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::AmpEq
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::LogicalOr
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Pipeline // |>
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::CaretEq
                } else {
                    TokenKind::Caret
                }
            }
            '#' => TokenKind::Hash,
            _ => return Err(format!("Unexpected character: '{}' at {}:{}", ch, line, col)),
        };

        Ok(Token { kind, line, col })
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        loop {
            // Skip whitespace
            while let Some(ch) = self.peek() {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            // Check comments: ## or ###
            if self.peek() == Some('#') && self.peek_next() == Some('#') {
                self.advance(); // first #
                self.advance(); // second #

                // Check if doc comment (###)
                let _is_doc = if self.peek() == Some('#') {
                    self.advance();
                    true
                } else {
                    false
                };

                // Read until newline
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        self.advance();
                        break;
                    }
                    self.advance();
                }
                continue; // loop again to skip following whitespace or more comments
            }

            break;
        }
        Ok(())
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token, String> {
        let line = self.line;
        let col = self.col;
        let mut text = String::new();

        // Check if prefixed with f" for f-strings
        if self.peek() == Some('f') && self.peek_next() == Some('"') {
            self.advance(); // consume 'f'
            return self.read_fstring(line, col);
        }

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match text.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "const" => TokenKind::Const,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "defer" => TokenKind::Defer,
            "unsafe" => TokenKind::Unsafe,
            "import" => TokenKind::Import,
            "as" => TokenKind::As,
            "pub" => TokenKind::Pub,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "match" => TokenKind::Match,
            "when" => TokenKind::When,
            "is" => TokenKind::Is,
            "interface" => TokenKind::Interface,
            "trait" => TokenKind::Trait,
            "impl" => TokenKind::Impl,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            _ => TokenKind::Identifier(text),
        };

        Ok(Token { kind, line, col })
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let line = self.line;
        let col = self.col;
        let mut text = String::new();
        let mut is_float = false;

        // Check hex 0x, bin 0b, octal 0o
        if self.peek() == Some('0') {
            if let Some(next) = self.peek_next() {
                if next == 'x' || next == 'X' {
                    self.advance(); // 0
                    self.advance(); // x
                    let mut hex_str = String::new();
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_hexdigit() {
                            hex_str.push(ch);
                            self.advance();
                        } else if ch == '_' {
                            self.advance(); // skip underscore
                        } else {
                            break;
                        }
                    }
                    let val = i128::from_str_radix(&hex_str, 16)
                        .map_err(|e| format!("Invalid hex integer: {}", e))?;
                    return Ok(Token {
                        kind: TokenKind::IntLiteral(val),
                        line,
                        col,
                    });
                } else if next == 'b' || next == 'B' {
                    self.advance(); // 0
                    self.advance(); // b
                    let mut bin_str = String::new();
                    while let Some(ch) = self.peek() {
                        if ch == '0' || ch == '1' {
                            bin_str.push(ch);
                            self.advance();
                        } else if ch == '_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val = i128::from_str_radix(&bin_str, 2)
                        .map_err(|e| format!("Invalid binary integer: {}", e))?;
                    return Ok(Token {
                        kind: TokenKind::IntLiteral(val),
                        line,
                        col,
                    });
                } else if next == 'o' || next == 'O' {
                    self.advance(); // 0
                    self.advance(); // o
                    let mut oct_str = String::new();
                    while let Some(ch) = self.peek() {
                        if ch >= '0' && ch <= '7' {
                            oct_str.push(ch);
                            self.advance();
                        } else if ch == '_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val = i128::from_str_radix(&oct_str, 8)
                        .map_err(|e| format!("Invalid octal integer: {}", e))?;
                    return Ok(Token {
                        kind: TokenKind::IntLiteral(val),
                        line,
                        col,
                    });
                }
            }
        }

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.advance();
            } else if ch == '_' {
                self.advance(); // skip underscore in numbers: 1_000_000
            } else if ch == '.' {
                // Disambiguate float dot vs method call: 10.to_string() vs 10.5
                if let Some(next) = self.peek_next() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        text.push(ch);
                        self.advance();
                    } else {
                        // Dot followed by non-digit is a method call! Break so '.' becomes TokenKind::Dot
                        break;
                    }
                } else {
                    break;
                }
            } else if ch == 'e' || ch == 'E' {
                is_float = true;
                text.push(ch);
                self.advance();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        text.push(sign);
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        // Skip optional type suffix like _u8, _i64, _f64
        if self.peek() == Some('_') {
            if let Some(next) = self.peek_next() {
                if next.is_alphabetic() {
                    self.advance();
                    while let Some(s) = self.peek() {
                        if s.is_alphanumeric() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        if is_float {
            let val = text
                .parse::<f64>()
                .map_err(|e| format!("Invalid float: {}", e))?;
            Ok(Token {
                kind: TokenKind::FloatLiteral(val),
                line,
                col,
            })
        } else {
            let val = text
                .parse::<i128>()
                .map_err(|e| format!("Invalid integer: {}", e))?;
            Ok(Token {
                kind: TokenKind::IntLiteral(val),
                line,
                col,
            })
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let line = self.line;
        let col = self.col;
        self.advance(); // consume opening quote "

        let mut content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // consume closing quote
                return Ok(Token {
                    kind: TokenKind::StringLiteral(content),
                    line,
                    col,
                });
            } else if ch == '\\' {
                self.advance();
                let esc = self
                    .advance()
                    .ok_or_else(|| "Unexpected EOF in escape sequence".to_string())?;
                match esc {
                    'n' => content.push('\n'),
                    't' => content.push('\t'),
                    'r' => content.push('\r'),
                    '0' => content.push('\0'),
                    '\\' => content.push('\\'),
                    '"' => content.push('"'),
                    other => content.push(other),
                }
            } else {
                content.push(ch);
                self.advance();
            }
        }

        Err("Unterminated string literal".to_string())
    }

    fn read_fstring(&mut self, line: usize, col: usize) -> Result<Token, String> {
        self.advance(); // consume opening quote "
        let mut content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::FStringLiteral(content),
                    line,
                    col,
                });
            }
            content.push(ch);
            self.advance();
        }
        Err("Unterminated f-string literal".to_string())
    }

    fn read_char(&mut self) -> Result<Token, String> {
        let line = self.line;
        let col = self.col;
        self.advance(); // consume opening '

        let ch = self
            .advance()
            .ok_or_else(|| "Unexpected EOF in char literal".to_string())?;

        let val = if ch == '\\' {
            let esc = self
                .advance()
                .ok_or_else(|| "Unexpected EOF in escape sequence".to_string())?;
            match esc {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '0' => '\0',
                '\\' => '\\',
                '\'' => '\'',
                other => other,
            }
        } else {
            ch
        };

        if self.peek() == Some('\'') {
            self.advance();
            Ok(Token {
                kind: TokenKind::CharLiteral(val),
                line,
                col,
            })
        } else {
            Err("Unterminated char literal".to_string())
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let source = r#"
            ## Single line comment
            fn main() void {
                let mut x = 10 // 2
                let result = 1e2 |> double()
            }
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("main".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
    }

    #[test]
    fn test_lexer_floor_div_and_comment() {
        let source = "7 // 2 ## comment";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(7));
        assert_eq!(tokens[1].kind, TokenKind::FloorDiv); // // is FloorDiv
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral(2));
    }

    #[test]
    fn test_scientific_and_hex() {
        let source = "1e2 0xFF 0b1010";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::FloatLiteral(100.0));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(255));
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral(10));
    }
}
