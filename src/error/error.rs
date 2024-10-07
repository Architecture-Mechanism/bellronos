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

use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum BellronosError {
    IO(io::Error),
    Parser(String),
    Type(String),
    Runtime(String),
    Network(String),
    Package(String),
}

impl fmt::Display for BellronosError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BellronosError::IO(err) => write!(f, "IO error: {}", err),
            BellronosError::Parser(msg) => write!(f, "Parser error: {}", msg),
            BellronosError::Type(msg) => write!(f, "Type error: {}", msg),
            BellronosError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            BellronosError::Network(msg) => write!(f, "Network error: {}", msg),
            BellronosError::Package(msg) => write!(f, "Package error: {}", msg),
        }
    }
}

impl Error for BellronosError {}

impl From<io::Error> for BellronosError {
    fn from(err: io::Error) -> BellronosError {
        BellronosError::IO(err)
    }
}
