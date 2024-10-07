// Copyright (C) 2024 Bellande Architecture Mechanism Research Innovation Center, Ronaldson Bellande

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::ast::ast::ASTNode;
use crate::error::error::BellronosError;
use crate::lexer::lexer::Token;
use crate::type_system::type_system::Type;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, BellronosError> {
        let mut body = Vec::new();
        while self.position < self.tokens.len() {
            body.push(self.parse_statement()?);
        }
        Ok(ASTNode::Module { body })
    }

    fn parse_statement(&mut self) -> Result<ASTNode, BellronosError> {
        match self.current_token() {
            Token::Import => self.parse_import(),
            Token::Define => self.parse_function_def(),
            Token::Class => self.parse_class_def(),
            Token::Set => self.parse_assignment(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Return => self.parse_return(),
            Token::Async => self.parse_async(),
            Token::Yield => self.parse_yield(),
            Token::Closure => self.parse_closure(),
            _ => self.parse_expression(),
        }
    }

    fn parse_import(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'import'
        let mut names = Vec::new();
        while let Token::Identifier(name) = self.current_token() {
            names.push(name.clone());
            self.advance();
            if self.current_token() == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_token(Token::Newline)?;
        Ok(ASTNode::Import { names })
    }

    fn parse_function_def(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'define'
        let name = self.expect_identifier()?;
        let args = self.parse_function_args()?;
        let return_type = self.parse_return_type()?;
        self.expect_token(Token::Colon)?;
        self.expect_token(Token::Newline)?;
        let body = self.parse_block()?;
        Ok(ASTNode::FunctionDef {
            name,
            args,
            return_type,
            body,
        })
    }

    fn parse_class_def(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'class'
        let name = self.expect_identifier()?;
        self.expect_token(Token::Colon)?;
        self.expect_token(Token::Newline)?;
        let methods = self.parse_block()?;
        Ok(ASTNode::ClassDef { name, methods })
    }

    fn parse_assignment(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'set'
        let target = self.expect_identifier()?;
        self.expect_token(Token::To)?;
        let value = Box::new(self.parse_expression()?);
        self.expect_token(Token::Newline)?;
        Ok(ASTNode::Assign { target, value })
    }

    fn parse_if(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'if'
        let condition = Box::new(self.parse_expression()?);
        self.expect_token(Token::Colon)?;
        self.expect_token(Token::Newline)?;
        let body = self.parse_block()?;
        let mut orelse = Vec::new();
        if self.current_token() == Token::Else {
            self.advance();
            self.expect_token(Token::Colon)?;
            self.expect_token(Token::Newline)?;
            orelse = self.parse_block()?;
        }
        Ok(ASTNode::If {
            condition,
            body,
            orelse,
        })
    }

    fn parse_while(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'while'
        let condition = Box::new(self.parse_expression()?);
        self.expect_token(Token::Colon)?;
        self.expect_token(Token::Newline)?;
        let body = self.parse_block()?;
        Ok(ASTNode::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'for'
        let target = self.expect_identifier()?;
        self.expect_token(Token::In)?;
        let iter = Box::new(self.parse_expression()?);
        self.expect_token(Token::Colon)?;
        self.expect_token(Token::Newline)?;
        let body = self.parse_block()?;
        Ok(ASTNode::For { target, iter, body })
    }

    fn parse_return(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'return'
        let value = if self.current_token() == Token::Newline {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        self.expect_token(Token::Newline)?;
        Ok(ASTNode::Return { value })
    }

    fn parse_async(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'async'
        self.expect_token(Token::Define)?;
        let function = self.parse_function_def()?;
        if let ASTNode::FunctionDef {
            name,
            args,
            return_type,
            body,
        } = function
        {
            Ok(ASTNode::Async {
                body: vec![ASTNode::FunctionDef {
                    name,
                    args,
                    return_type,
                    body,
                }],
            })
        } else {
            Err(BellronosError::Parser(
                "Expected function definition after 'async'".to_string(),
            ))
        }
    }

    fn parse_yield(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'yield'
        let value = Box::new(self.parse_expression()?);
        self.expect_token(Token::Newline)?;
        Ok(ASTNode::Yield { value })
    }

    fn parse_closure(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume 'closure'
        let params = self.parse_function_args()?;
        self.expect_token(Token::Colon)?;
        let body = Box::new(self.parse_expression()?);
        Ok(ASTNode::Closure { params, body })
    }

    fn parse_expression(&mut self) -> Result<ASTNode, BellronosError> {
        self.parse_binary_operation()
    }

    fn parse_binary_operation(&mut self) -> Result<ASTNode, BellronosError> {
        let mut left = self.parse_unary()?;

        while matches!(
            self.current_token(),
            Token::Plus
                | Token::Minus
                | Token::Multiply
                | Token::Divide
                | Token::Equals
                | Token::NotEquals
                | Token::LessThan
                | Token::GreaterThan
                | Token::LessThanOrEqual
                | Token::GreaterThanOrEqual
        ) {
            let op = format!("{:?}", self.current_token());
            self.advance();
            let right = self.parse_unary()?;
            left = ASTNode::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<ASTNode, BellronosError> {
        match self.current_token() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(ASTNode::BinOp {
                    left: Box::new(ASTNode::Num { value: 0.0 }),
                    op: "-".to_string(),
                    right: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<ASTNode, BellronosError> {
        match self.current_token() {
            Token::Identifier(name) => {
                self.advance();
                if self.current_token() == Token::LeftParen {
                    self.parse_function_call(&name)
                } else {
                    Ok(ASTNode::Name { id: name })
                }
            }
            Token::String(value) => {
                self.advance();
                Ok(ASTNode::Str { value })
            }
            Token::Number(value) => {
                self.advance();
                Ok(ASTNode::Num { value })
            }
            Token::True => {
                self.advance();
                Ok(ASTNode::Bool { value: true })
            }
            Token::False => {
                self.advance();
                Ok(ASTNode::Bool { value: false })
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => self.parse_list(),
            Token::LeftBrace => self.parse_dict(),
            _ => Err(BellronosError::Parser(format!(
                "Unexpected token: {:?}",
                self.current_token()
            ))),
        }
    }

    fn parse_function_call(&mut self, name: &str) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume '('
        let mut args = Vec::new();
        if self.current_token() != Token::RightParen {
            loop {
                args.push(self.parse_expression()?);
                if self.current_token() == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_token(Token::RightParen)?;
        Ok(ASTNode::Call {
            func: name.to_string(),
            args,
        })
    }

    fn parse_list(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume '['
        let mut elements = Vec::new();
        if self.current_token() != Token::RightBracket {
            loop {
                elements.push(self.parse_expression()?);
                if self.current_token() == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_token(Token::RightBracket)?;
        Ok(ASTNode::List { elements })
    }

    fn parse_dict(&mut self) -> Result<ASTNode, BellronosError> {
        self.advance(); // Consume '{'
        let mut pairs = Vec::new();
        if self.current_token() != Token::RightBrace {
            loop {
                let key = self.parse_expression()?;
                self.expect_token(Token::Colon)?;
                let value = self.parse_expression()?;
                pairs.push((key, value));
                if self.current_token() == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_token(Token::RightBrace)?;
        Ok(ASTNode::Dict { pairs })
    }

    fn parse_function_args(&mut self) -> Result<Vec<(String, Type)>, BellronosError> {
        let mut args = Vec::new();
        self.expect_token(Token::LeftParen)?;
        if self.current_token() != Token::RightParen {
            loop {
                let name = self.expect_identifier()?;
                self.expect_token(Token::Colon)?;
                let type_ = self.parse_type()?;
                args.push((name, type_));
                if self.current_token() == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_token(Token::RightParen)?;
        Ok(args)
    }

    fn parse_return_type(&mut self) -> Result<Type, BellronosError> {
        if self.current_token() == Token::Arrow {
            self.advance();
            self.parse_type()
        } else {
            Ok(Type::None)
        }
    }

    fn parse_type(&mut self) -> Result<Type, BellronosError> {
        match self.current_token() {
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "int" => Ok(Type::Int),
                    "float" => Ok(Type::Float),
                    "string" => Ok(Type::String),
                    "bool" => Ok(Type::Bool),
                    "list" => {
                        self.expect_token(Token::LeftBracket)?;
                        let inner_type = self.parse_type()?;
                        self.expect_token(Token::RightBracket)?;
                        Ok(Type::List(Box::new(inner_type)))
                    }
                    "dict" => {
                        self.expect_token(Token::LeftBrace)?;
                        let key_type = self.parse_type()?;
                        self.expect_token(Token::Colon)?;
                        let value_type = self.parse_type()?;
                        self.expect_token(Token::RightBrace)?;
                        Ok(Type::Dict(Box::new(key_type), Box::new(value_type)))
                    }
                    _ => Ok(Type::Custom(name)),
                }
            }
            _ => Err(BellronosError::Parser("Expected type".to_string())),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<ASTNode>, BellronosError> {
        let mut body = Vec::new();
        while self.current_token() != Token::EOF && !matches!(self.current_token(), Token::Else) {
            body.push(self.parse_statement()?);
        }
        Ok(body)
    }

    fn expect_identifier(&mut self) -> Result<String, BellronosError> {
        if let Token::Identifier(name) = self.current_token() {
            self.advance();
            Ok(name)
        } else {
            Err(BellronosError::Parser(format!(
                "Expected identifier, found {:?}",
                self.current_token()
            )))
        }
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), BellronosError> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(BellronosError::Parser(format!(
                "Expected {:?}, found {:?}",
                expected,
                self.current_token()
            )))
        }
    }

    fn current_token(&self) -> Token {
        self.tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::EOF)
    }

    fn advance(&mut self) {
        self.position += 1;
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
