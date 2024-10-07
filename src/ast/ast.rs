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

use crate::type_system::type_system::Type;

#[derive(Clone, PartialEq, Debug)]
pub enum ASTNode {
    Module {
        body: Vec<ASTNode>,
    },
    Import {
        names: Vec<String>,
    },
    FunctionDef {
        name: String,
        args: Vec<(String, Type)>,
        return_type: Type,
        body: Vec<ASTNode>,
    },
    ClassDef {
        name: String,
        methods: Vec<ASTNode>,
    },
    Assign {
        target: String,
        value: Box<ASTNode>,
    },
    Expr {
        value: Box<ASTNode>,
    },
    Call {
        func: String,
        args: Vec<ASTNode>,
    },
    Str {
        value: String,
    },
    Num {
        value: f64,
    },
    Bool {
        value: bool,
    },
    Name {
        id: String,
    },
    BinOp {
        left: Box<ASTNode>,
        op: String,
        right: Box<ASTNode>,
    },
    If {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
        orelse: Vec<ASTNode>,
    },
    While {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
    },
    For {
        target: String,
        iter: Box<ASTNode>,
        body: Vec<ASTNode>,
    },
    Return {
        value: Option<Box<ASTNode>>,
    },
    Closure {
        params: Vec<(String, Type)>,
        body: Box<ASTNode>,
    },
    Generator {
        body: Vec<ASTNode>,
    },
    Yield {
        value: Box<ASTNode>,
    },
    Async {
        body: Vec<ASTNode>,
    },
    Await {
        value: Box<ASTNode>,
    },
    List {
        elements: Vec<ASTNode>,
    },
    Dict {
        pairs: Vec<(ASTNode, ASTNode)>,
    },
    Attribute {
        value: Box<ASTNode>,
        attr: String,
    },
    InteropCall {
        language: String,
        code: String,
    },
}
