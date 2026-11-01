//! Recursive descent parser for the Apsis programming language.

use apsis_ast::*;
use apsis_lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> &Token {
        let prev = self.pos;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        &self.tokens[prev.min(self.tokens.len() - 1)]
    }

    fn expect(&mut self, expected: TokenKind) -> Result<&Token, String> {
        let tok = self.peek();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(&expected) {
            Ok(self.advance())
        } else {
            Err(format!(
                "Expected {:?}, found {:?} at line {}, col {}",
                expected, tok.kind, tok.line, tok.col
            ))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();

        while *self.peek_kind() != TokenKind::Eof {
            let func = self.parse_function()?;
            functions.push(func);
        }

        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, String> {
        let is_pub = if *self.peek_kind() == TokenKind::Pub {
            self.advance();
            true
        } else {
            false
        };

        let is_async = if *self.peek_kind() == TokenKind::Async {
            self.advance();
            true
        } else {
            false
        };

        self.expect(TokenKind::Fn)?;

        let name = match self.peek_kind() {
            TokenKind::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            other => return Err(format!("Expected function name, found {:?}", other)),
        };

        // Generics [T, U]
        let mut generics = Vec::new();
        if *self.peek_kind() == TokenKind::LBracket {
            self.advance();
            while *self.peek_kind() != TokenKind::RBracket {
                if let TokenKind::Identifier(gen_name) = self.peek_kind() {
                    generics.push(gen_name.clone());
                    self.advance();
                    if *self.peek_kind() == TokenKind::Comma {
                        self.advance();
                    }
                } else {
                    return Err("Expected generic type identifier".to_string());
                }
            }
            self.expect(TokenKind::RBracket)?;
        }

        // Parameters (a int, mut b string = "default")
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        while *self.peek_kind() != TokenKind::RParen {
            let is_mut = if *self.peek_kind() == TokenKind::Mut {
                self.advance();
                true
            } else {
                false
            };

            let param_name = match self.peek_kind() {
                TokenKind::Identifier(n) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                other => return Err(format!("Expected parameter name, found {:?}", other)),
            };

            let param_type = self.parse_type()?;

            let default_val = if *self.peek_kind() == TokenKind::Assign {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };

            params.push(Param {
                name: param_name,
                is_mut,
                param_type,
                default_value: default_val,
            });

            if *self.peek_kind() == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;

        // Return type (optional void)
        let return_type = if *self.peek_kind() != TokenKind::LBrace && *self.peek_kind() != TokenKind::Semicolon {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body block
        let body = if *self.peek_kind() == TokenKind::LBrace {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(FunctionDecl {
            name,
            is_pub,
            is_async,
            generics,
            params,
            return_type,
            body,
            attributes: Vec::new(),
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek_kind() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                match name.as_str() {
                    "i8" => Ok(Type::Primitive(PrimitiveType::I8)),
                    "i16" => Ok(Type::Primitive(PrimitiveType::I16)),
                    "i32" => Ok(Type::Primitive(PrimitiveType::I32)),
                    "i64" => Ok(Type::Primitive(PrimitiveType::I64)),
                    "i128" => Ok(Type::Primitive(PrimitiveType::I128)),
                    "u8" => Ok(Type::Primitive(PrimitiveType::U8)),
                    "u16" => Ok(Type::Primitive(PrimitiveType::U16)),
                    "u32" => Ok(Type::Primitive(PrimitiveType::U32)),
                    "u64" => Ok(Type::Primitive(PrimitiveType::U64)),
                    "u128" => Ok(Type::Primitive(PrimitiveType::U128)),
                    "f32" => Ok(Type::Primitive(PrimitiveType::F32)),
                    "f64" => Ok(Type::Primitive(PrimitiveType::F64)),
                    "bool" => Ok(Type::Primitive(PrimitiveType::Bool)),
                    "char" => Ok(Type::Primitive(PrimitiveType::Char)),
                    "void" => Ok(Type::Primitive(PrimitiveType::Void)),
                    other => Ok(Type::Custom(other.to_string())),
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                Ok(Type::List(Box::new(inner)))
            }
            TokenKind::LBrace => {
                self.advance();
                let key_type = self.parse_type()?;
                self.expect(TokenKind::Colon)?;
                let val_type = self.parse_type()?;
                self.expect(TokenKind::RBrace)?;
                Ok(Type::Map(Box::new(key_type), Box::new(val_type)))
            }
            other => Err(format!("Unexpected token while parsing type: {:?}", other)),
        }
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(TokenKind::LBrace)?;
        let mut statements = Vec::new();

        while *self.peek_kind() != TokenKind::RBrace && *self.peek_kind() != TokenKind::Eof {
            statements.push(self.parse_stmt()?);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(Block { statements })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek_kind() {
            TokenKind::Let => {
                self.advance();
                let is_mut = if *self.peek_kind() == TokenKind::Mut {
                    self.advance();
                    true
                } else {
                    false
                };

                let name = match self.peek_kind() {
                    TokenKind::Identifier(n) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    other => return Err(format!("Expected variable name, found {:?}", other)),
                };

                // Optional type annotation
                let declared_type = if *self.peek_kind() != TokenKind::Assign {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                self.expect(TokenKind::Assign)?;
                let init_expr = self.parse_expr()?;
                Ok(Stmt::Let {
                    name,
                    is_mut,
                    declared_type,
                    init_expr,
                })
            }
            TokenKind::Const => {
                self.advance();
                let name = match self.peek_kind() {
                    TokenKind::Identifier(n) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    other => return Err(format!("Expected constant name, found {:?}", other)),
                };

                let declared_type = if *self.peek_kind() != TokenKind::Assign {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                self.expect(TokenKind::Assign)?;
                let value = self.parse_expr()?;
                Ok(Stmt::Const {
                    name,
                    declared_type,
                    value,
                })
            }
            TokenKind::Defer => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Stmt::Defer(expr))
            }
            TokenKind::Return => {
                self.advance();
                if *self.peek_kind() != TokenKind::RBrace && *self.peek_kind() != TokenKind::Semicolon {
                    let expr = self.parse_expr()?;
                    Ok(Stmt::Return(Some(expr)))
                } else {
                    Ok(Stmt::Return(None))
                }
            }
            _ => {
                let expr = self.parse_expr()?;

                // Check assignment or compound assignment: target += value
                let op = match self.peek_kind() {
                    TokenKind::Assign => Some(None),
                    TokenKind::PlusEq => Some(Some(BinaryOperator::Add)),
                    TokenKind::MinusEq => Some(Some(BinaryOperator::Sub)),
                    TokenKind::StarEq => Some(Some(BinaryOperator::Mul)),
                    TokenKind::SlashEq => Some(Some(BinaryOperator::Div)),
                    TokenKind::PercentEq => Some(Some(BinaryOperator::Mod)),
                    TokenKind::AmpEq => Some(Some(BinaryOperator::BitAnd)),
                    TokenKind::PipeEq => Some(Some(BinaryOperator::BitOr)),
                    TokenKind::CaretEq => Some(Some(BinaryOperator::BitXor)),
                    TokenKind::ShlEq => Some(Some(BinaryOperator::Shl)),
                    TokenKind::ShrEq => Some(Some(BinaryOperator::Shr)),
                    _ => None,
                };

                if let Some(compound_op) = op {
                    self.advance();
                    let val = self.parse_expr()?;
                    Ok(Stmt::Assign {
                        target: expr,
                        op: compound_op,
                        value: val,
                    })
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_pipeline()
    }

    fn parse_pipeline(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_binary(0)?;

        while *self.peek_kind() == TokenKind::Pipeline {
            self.advance(); // consume |>
            let right = self.parse_binary(0)?;
            left = Expr::Pipe {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;

        loop {
            let (op, prec) = match self.peek_kind() {
                TokenKind::FloorDiv => (BinaryOperator::FloorDiv, 5), // //
                TokenKind::Plus => (BinaryOperator::Add, 3),
                TokenKind::Minus => (BinaryOperator::Sub, 3),
                TokenKind::Star => (BinaryOperator::Mul, 4),
                TokenKind::Slash => (BinaryOperator::Div, 4),
                TokenKind::Percent => (BinaryOperator::Mod, 4),
                TokenKind::Eq => (BinaryOperator::Eq, 2),
                TokenKind::NotEq => (BinaryOperator::NotEq, 2),
                TokenKind::Lt => (BinaryOperator::Lt, 2),
                TokenKind::LtEq => (BinaryOperator::LtEq, 2),
                TokenKind::Gt => (BinaryOperator::Gt, 2),
                TokenKind::GtEq => (BinaryOperator::GtEq, 2),
                _ => break,
            };

            if prec < min_prec {
                break;
            }

            self.advance();
            let right = self.parse_binary(prec + 1)?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let expr = match self.peek_kind() {
            TokenKind::IntLiteral(val) => {
                let v = *val;
                self.advance();
                Expr::IntLiteral(v)
            }
            TokenKind::FloatLiteral(val) => {
                let v = *val;
                self.advance();
                Expr::FloatLiteral(v)
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Expr::StringLiteral(s)
            }
            TokenKind::BoolLiteral(b) => {
                let b = *b;
                self.advance();
                Expr::BoolLiteral(b)
            }
            TokenKind::CharLiteral(c) => {
                let c = *c;
                self.advance();
                Expr::CharLiteral(c)
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Expr::Identifier(name)
            }
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                inner
            }
            other => return Err(format!("Unexpected token in expression: {:?}", other)),
        };

        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            match self.peek_kind() {
                TokenKind::LParen => {
                    // Function Call ()
                    self.advance();
                    let mut args = Vec::new();
                    while *self.peek_kind() != TokenKind::RParen {
                        let is_mut = if *self.peek_kind() == TokenKind::Mut {
                            self.advance();
                            true
                        } else {
                            false
                        };

                        let val = self.parse_expr()?;
                        args.push(CallArg {
                            name: None,
                            is_mut,
                            value: val,
                        });

                        if *self.peek_kind() == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                TokenKind::Dot => {
                    // Method Call .method(...)
                    self.advance();
                    let method_name = match self.peek_kind() {
                        TokenKind::Identifier(m) => {
                            let m = m.clone();
                            self.advance();
                            m
                        }
                        other => return Err(format!("Expected method name after dot, found {:?}", other)),
                    };

                    self.expect(TokenKind::LParen)?;
                    let mut args = Vec::new();
                    while *self.peek_kind() != TokenKind::RParen {
                        let is_mut = if *self.peek_kind() == TokenKind::Mut {
                            self.advance();
                            true
                        } else {
                            false
                        };

                        let val = self.parse_expr()?;
                        args.push(CallArg {
                            name: None,
                            is_mut,
                            value: val,
                        });

                        if *self.peek_kind() == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    expr = Expr::MethodCall {
                        receiver: Box::new(expr),
                        method_name,
                        args,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apsis_lexer::Lexer;

    #[test]
    fn test_parse_simple_function() {
        let code = r#"
            fn add(a i64, b i64 = 0) i64 {
                let mut total = a + b
                return total
            }
        "#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "add");
        assert_eq!(program.functions[0].params.len(), 2);
    }

    #[test]
    fn test_parse_floor_div_and_pipe() {
        let code = r#"
            fn calculate() {
                let x = 10 // 2 |> double()
            }
        "#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.functions.len(), 1);
    }
}
