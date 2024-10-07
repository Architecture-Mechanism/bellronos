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

use crate::error::error::BellronosError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Import,
    Define,
    Class,
    Set,
    To,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Async,
    Await,
    Yield,
    Closure,
    True,
    False,
    Identifier(String),
    String(String),
    Number(f64),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Dot,
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Arrow,
    Newline,
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, BellronosError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        tokens.push(Token::EOF);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, BellronosError> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Ok(None);
        }

        let token = match self.current_char() {
            '(' => {
                self.advance();
                Ok(Some(Token::LeftParen))
            }
            ')' => {
                self.advance();
                Ok(Some(Token::RightParen))
            }
            '{' => {
                self.advance();
                Ok(Some(Token::LeftBrace))
            }
            '}' => {
                self.advance();
                Ok(Some(Token::RightBrace))
            }
            '[' => {
                self.advance();
                Ok(Some(Token::LeftBracket))
            }
            ']' => {
                self.advance();
                Ok(Some(Token::RightBracket))
            }
            ':' => {
                self.advance();
                Ok(Some(Token::Colon))
            }
            ',' => {
                self.advance();
                Ok(Some(Token::Comma))
            }
            '.' => {
                self.advance();
                Ok(Some(Token::Dot))
            }
            '+' => {
                self.advance();
                Ok(Some(Token::Plus))
            }
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    Ok(Some(Token::Arrow))
                } else {
                    Ok(Some(Token::Minus))
                }
            }
            '*' => {
                self.advance();
                Ok(Some(Token::Multiply))
            }
            '/' => {
                self.advance();
                Ok(Some(Token::Divide))
            }
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Some(Token::Equals))
                } else {
                    Ok(Some(Token::Set))
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Some(Token::NotEquals))
                } else {
                    Err(BellronosError::Parser(format!(
                        "Unexpected character: {}",
                        self.current_char()
                    )))
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Some(Token::LessThanOrEqual))
                } else {
                    Ok(Some(Token::LessThan))
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Some(Token::GreaterThanOrEqual))
                } else {
                    Ok(Some(Token::GreaterThan))
                }
            }
            '\n' => {
                self.advance_line();
                Ok(Some(Token::Newline))
            }
            '"' => self.tokenize_string(),
            '0'..='9' => self.tokenize_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.tokenize_identifier(),
            '#' => {
                while self.current_char() != '\n' && self.position < self.input.len() {
                    self.advance();
                }
                self.next_token()
            }
            _ => Err(BellronosError::Parser(format!(
                "Unexpected character: {}",
                self.current_char()
            ))),
        }?;

        Ok(token)
    }

    fn tokenize_string(&mut self) -> Result<Option<Token>, BellronosError> {
        self.advance(); // Skip opening quote
        let start = self.position;
        while self.current_char() != '"' && self.position < self.input.len() {
            if self.current_char() == '\n' {
                return Err(BellronosError::Parser(format!(
                    "Unexpected character: {}",
                    self.current_char()
                )));
            }
            self.advance();
        }
        if self.position >= self.input.len() {
            return Err(BellronosError::Parser(format!(
                "Unexpected character: {}",
                self.current_char()
            )));
        }
        let value: String = self.input[start..self.position].iter().collect();
        self.advance(); // Skip closing quote
        Ok(Some(Token::String(value)))
    }

    fn tokenize_number(&mut self) -> Result<Option<Token>, BellronosError> {
        let start = self.position;
        while self.position < self.input.len()
            && (self.current_char().is_digit(10) || self.current_char() == '.')
        {
            self.advance();
        }
        let value: String = self.input[start..self.position].iter().collect();
        value
            .parse::<f64>()
            .map(|n| Some(Token::Number(n)))
            .map_err(|_| {
                BellronosError::Parser(format!("Unexpected character: {}", self.current_char()))
            })
    }

    fn tokenize_identifier(&mut self) -> Result<Option<Token>, BellronosError> {
        let start = self.position;
        while self.position < self.input.len()
            && (self.current_char().is_alphanumeric() || self.current_char() == '_')
        {
            self.advance();
        }
        let value: String = self.input[start..self.position].iter().collect();
        Ok(Some(match value.as_str() {
            "import" => Token::Import,
            "define" => Token::Define,
            "class" => Token::Class,
            "set" => Token::Set,
            "to" => Token::To,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "return" => Token::Return,
            "async" => Token::Async,
            "await" => Token::Await,
            "yield" => Token::Yield,
            "closure" => Token::Closure,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(value),
        }))
    }

    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;
    }

    fn advance_line(&mut self) {
        self.position += 1;
        self.line += 1;
        self.column = 1;
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len()
            && self.current_char().is_whitespace()
            && self.current_char() != '\n'
        {
            self.advance();
        }
    }
}
