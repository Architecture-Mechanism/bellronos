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
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum InteropType {
    CInt,
    CFloat,
    CString,
    PyInt,
    PyFloat,
    PyString,
    PyList,
    PyDict,
    JsNumber,
    JsString,
    JsBoolean,
    JsArray,
    JsObject,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteropLanguage {
    C,
    Python,
    JavaScript,
}

impl FromStr for InteropLanguage {
    type Err = BellronosError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "c" => Ok(InteropLanguage::C),
            "python" => Ok(InteropLanguage::Python),
            "javascript" | "js" => Ok(InteropLanguage::JavaScript),
            _ => Err(BellronosError::Type(format!(
                "Unknown interop language: {}",
                s
            ))),
        }
    }
}

impl InteropLanguage {
    pub fn infer_type(&self, code: &str) -> Result<InteropType, BellronosError> {
        match self {
            InteropLanguage::C => {
                if code.contains("int") {
                    Ok(InteropType::CInt)
                } else if code.contains("float") {
                    Ok(InteropType::CFloat)
                } else if code.contains("char*") {
                    Ok(InteropType::CString)
                } else {
                    Ok(InteropType::Unknown)
                }
            }
            InteropLanguage::Python => {
                if code.contains("int(") {
                    Ok(InteropType::PyInt)
                } else if code.contains("float(") {
                    Ok(InteropType::PyFloat)
                } else if code.contains("str(") {
                    Ok(InteropType::PyString)
                } else if code.contains("list(") {
                    Ok(InteropType::PyList)
                } else if code.contains("dict(") {
                    Ok(InteropType::PyDict)
                } else {
                    Ok(InteropType::Unknown)
                }
            }
            InteropLanguage::JavaScript => {
                if code.contains("Number(") {
                    Ok(InteropType::JsNumber)
                } else if code.contains("String(") {
                    Ok(InteropType::JsString)
                } else if code.contains("Boolean(") {
                    Ok(InteropType::JsBoolean)
                } else if code.contains("Array(") {
                    Ok(InteropType::JsArray)
                } else if code.contains("Object(") {
                    Ok(InteropType::JsObject)
                } else {
                    Ok(InteropType::Unknown)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Class(String),
    Instance(String),
    None,
    Any,
    Custom(String),
    Interop(InteropType),
}

#[derive(Clone)]
pub struct TypeChecker {
    type_env: HashMap<String, Type>,
    class_env: HashMap<String, HashMap<String, Type>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            type_env: HashMap::new(),
            class_env: HashMap::new(),
        }
    }

    pub fn check(&mut self, node: &ASTNode) -> Result<Type, BellronosError> {
        match node {
            ASTNode::Module { body } => {
                for stmt in body {
                    self.check(stmt)?;
                }
                Ok(Type::None)
            }
            ASTNode::Import { names } => {
                for name in names {
                    self.type_env.insert(name.clone(), Type::Any);
                }
                Ok(Type::None)
            }
            ASTNode::FunctionDef {
                name,
                args,
                return_type,
                body,
            } => {
                let arg_types: Vec<Type> = args.iter().map(|(_, t)| t.clone()).collect();
                let func_type = Type::Function(arg_types.clone(), Box::new(return_type.clone()));
                self.type_env.insert(name.clone(), func_type);

                let mut func_checker = self.clone();
                for (arg_name, arg_type) in args {
                    func_checker
                        .type_env
                        .insert(arg_name.clone(), arg_type.clone());
                }

                for stmt in body {
                    let stmt_type = func_checker.check(stmt)?;
                    if let ASTNode::Return { .. } = stmt {
                        if stmt_type != *return_type {
                            return Err(BellronosError::Type(format!(
                                "Function {} return type mismatch: expected {:?}, found {:?}",
                                name, return_type, stmt_type
                            )));
                        }
                    }
                }
                Ok(Type::None)
            }
            ASTNode::ClassDef { name, methods } => {
                let mut class_methods = HashMap::new();
                for method in methods {
                    if let ASTNode::FunctionDef {
                        name: method_name,
                        args,
                        return_type,
                        ..
                    } = method
                    {
                        let arg_types: Vec<Type> = args.iter().map(|(_, t)| t.clone()).collect();
                        class_methods.insert(
                            method_name.clone(),
                            Type::Function(arg_types, Box::new(return_type.clone())),
                        );
                    }
                }
                self.class_env.insert(name.clone(), class_methods);
                self.type_env
                    .insert(name.clone(), Type::Class(name.clone()));
                Ok(Type::None)
            }
            ASTNode::Assign { target, value } => {
                let value_type = self.check(value)?;
                self.type_env.insert(target.clone(), value_type);
                Ok(Type::None)
            }
            ASTNode::Expr { value } => self.check(value),
            ASTNode::Call { func, args } => {
                let func_type =
                    self.type_env.get(func).cloned().ok_or_else(|| {
                        BellronosError::Type(format!("Undefined function: {}", func))
                    })?;

                if let Type::Function(param_types, return_type) = func_type {
                    if args.len() != param_types.len() {
                        return Err(BellronosError::Type(format!(
                            "Function {} expects {} arguments, but {} were given",
                            func,
                            param_types.len(),
                            args.len()
                        )));
                    }
                    for (arg, expected_type) in args.iter().zip(param_types.iter()) {
                        let arg_type = self.check(arg)?;
                        if !self.is_compatible(&arg_type, expected_type) {
                            return Err(BellronosError::Type(format!(
                                "Type mismatch: expected {:?}, found {:?}",
                                expected_type, arg_type
                            )));
                        }
                    }
                    Ok(*return_type)
                } else {
                    Err(BellronosError::Type(format!("{} is not a function", func)))
                }
            }
            ASTNode::Str { .. } => Ok(Type::String),
            ASTNode::Num { .. } => Ok(Type::Float),
            ASTNode::Bool { .. } => Ok(Type::Bool),
            ASTNode::Name { id } => self
                .type_env
                .get(id)
                .cloned()
                .ok_or_else(|| BellronosError::Type(format!("Undefined variable: {}", id))),
            ASTNode::BinOp { left, op, right } => {
                let left_type = self.check(left)?;
                let right_type = self.check(right)?;
                self.check_binary_op(&left_type, op, &right_type)
            }
            ASTNode::If {
                condition,
                body,
                orelse,
            } => {
                let cond_type = self.check(condition)?;
                if cond_type != Type::Bool {
                    return Err(BellronosError::Type(
                        "If condition must be a boolean".to_string(),
                    ));
                }
                for stmt in body {
                    self.check(stmt)?;
                }
                for stmt in orelse {
                    self.check(stmt)?;
                }
                Ok(Type::None)
            }
            ASTNode::While { condition, body } => {
                let cond_type = self.check(condition)?;
                if cond_type != Type::Bool {
                    return Err(BellronosError::Type(
                        "While condition must be a boolean".to_string(),
                    ));
                }
                for stmt in body {
                    self.check(stmt)?;
                }
                Ok(Type::None)
            }
            ASTNode::For { target, iter, body } => {
                let iter_type = self.check(iter)?;
                if let Type::List(element_type) = iter_type {
                    self.type_env.insert(target.clone(), *element_type);
                    for stmt in body {
                        self.check(stmt)?;
                    }
                    Ok(Type::None)
                } else {
                    Err(BellronosError::Type(
                        "For loop iterable must be a list".to_string(),
                    ))
                }
            }
            ASTNode::Return { value } => {
                if let Some(v) = value {
                    self.check(v)
                } else {
                    Ok(Type::None)
                }
            }
            ASTNode::Closure { params, body } => {
                let param_types: Vec<Type> = params.iter().map(|(_, t)| t.clone()).collect();
                let mut closure_checker = self.clone();
                for (param_name, param_type) in params {
                    closure_checker
                        .type_env
                        .insert(param_name.clone(), param_type.clone());
                }
                let return_type = closure_checker.check(body)?;
                Ok(Type::Function(param_types, Box::new(return_type)))
            }
            ASTNode::Generator { body } => {
                for stmt in body {
                    self.check(stmt)?;
                }
                Ok(Type::List(Box::new(Type::Any)))
            }
            ASTNode::Yield { value } => self.check(value),
            ASTNode::Async { body } => {
                for stmt in body {
                    self.check(stmt)?;
                }
                Ok(Type::Any)
            }
            ASTNode::Await { value } => self.check(value),
            ASTNode::List { elements } => {
                if elements.is_empty() {
                    Ok(Type::List(Box::new(Type::Any)))
                } else {
                    let first_type = self.check(&elements[0])?;
                    for element in elements.iter().skip(1) {
                        let element_type = self.check(element)?;
                        if !self.is_compatible(&element_type, &first_type) {
                            return Err(BellronosError::Type(
                                "All list elements must have compatible types".to_string(),
                            ));
                        }
                    }
                    Ok(Type::List(Box::new(first_type)))
                }
            }
            ASTNode::Dict { pairs } => {
                if pairs.is_empty() {
                    Ok(Type::Dict(Box::new(Type::Any), Box::new(Type::Any)))
                } else {
                    let (first_key, first_value) = &pairs[0];
                    let key_type = self.check(first_key)?;
                    let value_type = self.check(first_value)?;
                    for (key, value) in pairs.iter().skip(1) {
                        let k_type = self.check(key)?;
                        let v_type = self.check(value)?;
                        if !self.is_compatible(&k_type, &key_type)
                            || !self.is_compatible(&v_type, &value_type)
                        {
                            return Err(BellronosError::Type(
                                "All dictionary keys must have compatible types, and all values must have compatible types".to_string(),
                            ));
                        }
                    }
                    Ok(Type::Dict(Box::new(key_type), Box::new(value_type)))
                }
            }
            ASTNode::Attribute { value, attr } => {
                let value_type = self.check(value)?;
                if let Type::Instance(class_name) = value_type {
                    if let Some(class_methods) = self.class_env.get(&class_name) {
                        class_methods.get(attr).cloned().ok_or_else(|| {
                            BellronosError::Type(format!(
                                "Attribute '{}' not found in class '{}'",
                                attr, class_name
                            ))
                        })
                    } else {
                        Err(BellronosError::Type(format!(
                            "Class '{}' not found",
                            class_name
                        )))
                    }
                } else {
                    Err(BellronosError::Type(format!(
                        "Cannot access attribute '{}' on non-instance type {:?}",
                        attr, value_type
                    )))
                }
            }
            ASTNode::InteropCall { language, code } => {
                let interop_lang = InteropLanguage::from_str(language)?;
                Ok(Type::Interop(interop_lang.infer_type(code)?))
            }
        }
    }

    fn check_binary_op(&self, left: &Type, op: &str, right: &Type) -> Result<Type, BellronosError> {
        match (left, op, right) {
            (Type::Int, "+", Type::Int) => Ok(Type::Int),
            (Type::Float, "+", Type::Float) => Ok(Type::Float),
            (Type::String, "+", Type::String) => Ok(Type::String),
            (Type::Int, "-", Type::Int) => Ok(Type::Int),
            (Type::Float, "-", Type::Float) => Ok(Type::Float),
            (Type::Int, "*", Type::Int) => Ok(Type::Int),
            (Type::Float, "*", Type::Float) => Ok(Type::Float),
            (Type::Int, "/", Type::Int) => Ok(Type::Float),
            (Type::Float, "/", Type::Float) => Ok(Type::Float),
            (Type::Int | Type::Float, op, Type::Int | Type::Float)
                if op == ">" || op == "<" || op == ">=" || op == "<=" =>
            {
                Ok(Type::Bool)
            }
            (_, "==" | "!=", _) => Ok(Type::Bool),
            _ => Err(BellronosError::Type(format!(
                "Invalid operation: {:?} {} {:?}",
                left, op, right
            ))),
        }
    }

    fn is_compatible(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            (Type::List(a), Type::List(b)) => self.is_compatible(a, b),
            (Type::Dict(ka, va), Type::Dict(kb, vb)) => {
                self.is_compatible(ka, kb) && self.is_compatible(va, vb)
            }
            (Type::Function(params_a, return_a), Type::Function(params_b, return_b)) => {
                params_a.len() == params_b.len()
                    && params_a
                        .iter()
                        .zip(params_b.iter())
                        .all(|(a, b)| self.is_compatible(a, b))
                    && self.is_compatible(return_a, return_b)
            }
            (Type::Interop(a), Type::Interop(b)) => a == b,
            (Type::Interop(_), _) | (_, Type::Interop(_)) => false, // Interop types are only compatible with themselves
            (a, b) => a == b,
        }
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.type_env.get(name)
    }

    pub fn set_type(&mut self, name: String, typ: Type) {
        self.type_env.insert(name, typ);
    }

    pub fn get_class_method(&self, class_name: &str, method_name: &str) -> Option<&Type> {
        self.class_env
            .get(class_name)
            .and_then(|methods| methods.get(method_name))
    }

    pub fn add_class_method(&mut self, class_name: &str, method_name: String, method_type: Type) {
        self.class_env
            .entry(class_name.to_string())
            .or_insert_with(HashMap::new)
            .insert(method_name, method_type);
    }
}

impl From<InteropType> for Type {
    fn from(interop_type: InteropType) -> Self {
        match interop_type {
            InteropType::CInt | InteropType::PyInt => Type::Int,
            InteropType::CFloat | InteropType::PyFloat | InteropType::JsNumber => Type::Float,
            InteropType::CString | InteropType::PyString | InteropType::JsString => Type::String,
            InteropType::PyList | InteropType::JsArray => Type::List(Box::new(Type::Any)),
            InteropType::PyDict | InteropType::JsObject => {
                Type::Dict(Box::new(Type::Any), Box::new(Type::Any))
            }
            InteropType::JsBoolean => Type::Bool,
            InteropType::Unknown => Type::Any,
        }
    }
}

pub fn interop_type_to_bellronos_type(interop_type: InteropType) -> Type {
    Type::from(interop_type)
}

pub fn bellronos_type_to_interop_type(
    bellronos_type: &Type,
    language: &InteropLanguage,
) -> InteropType {
    match (bellronos_type, language) {
        (Type::Int, InteropLanguage::C) => InteropType::CInt,
        (Type::Float, InteropLanguage::C) => InteropType::CFloat,
        (Type::String, InteropLanguage::C) => InteropType::CString,
        (Type::Int, InteropLanguage::Python) => InteropType::PyInt,
        (Type::Float, InteropLanguage::Python) => InteropType::PyFloat,
        (Type::String, InteropLanguage::Python) => InteropType::PyString,
        (Type::List(_), InteropLanguage::Python) => InteropType::PyList,
        (Type::Dict(_, _), InteropLanguage::Python) => InteropType::PyDict,
        (Type::Int | Type::Float, InteropLanguage::JavaScript) => InteropType::JsNumber,
        (Type::String, InteropLanguage::JavaScript) => InteropType::JsString,
        (Type::Bool, InteropLanguage::JavaScript) => InteropType::JsBoolean,
        (Type::List(_), InteropLanguage::JavaScript) => InteropType::JsArray,
        (Type::Dict(_, _), InteropLanguage::JavaScript) => InteropType::JsObject,
        _ => InteropType::Unknown,
    }
}
