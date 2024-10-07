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
use crate::interop::interop::LanguageInterop;
use crate::lexer::lexer::Lexer;
use crate::package_manager::package_manager::PackageManager;
use crate::parser::parser::Parser;
use crate::standard_library::standard_library::StandardLibrary;
use crate::type_system::type_system::TypeChecker;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

type Environment = Rc<RefCell<HashMap<String, Value>>>;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Function(Vec<String>, Vec<ASTNode>, Environment),
    Class {
        methods: HashMap<String, Value>,
    },
    Instance {
        class: String,
        attributes: HashMap<String, Value>,
    },
    Closure(Vec<String>, Vec<ASTNode>, Environment),
    Generator(Vec<ASTNode>, Environment, usize),
    None,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Dict(a), Value::Dict(b)) => a == b,
            // For Function, Class, Instance, Closure, and Generator, compare memory addresses
            (Value::Function(_, _, _), Value::Function(_, _, _)) => std::ptr::eq(self, other),
            (Value::Class { .. }, Value::Class { .. }) => std::ptr::eq(self, other),
            (Value::Instance { .. }, Value::Instance { .. }) => std::ptr::eq(self, other),
            (Value::Closure(_, _, _), Value::Closure(_, _, _)) => std::ptr::eq(self, other),
            (Value::Generator(_, _, _), Value::Generator(_, _, _)) => std::ptr::eq(self, other),
            (Value::None, Value::None) => true,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Int(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::List(l) => l.hash(state),
            Value::Dict(d) => {
                for (k, v) in d {
                    k.hash(state);
                    v.hash(state);
                }
            }
            // For Function, Class, Instance, Closure, and Generator, hash memory addresses
            Value::Function(_, _, _)
            | Value::Class { .. }
            | Value::Instance { .. }
            | Value::Closure(_, _, _)
            | Value::Generator(_, _, _) => (self as *const Value).hash(state),
            Value::None => 0.hash(state),
        }
    }
}

pub struct BellronosInterpreter {
    global_env: Environment,
    type_checker: TypeChecker,
    stdlib: StandardLibrary,
    package_manager: PackageManager,
    language_interop: LanguageInterop,
}

impl BellronosInterpreter {
    pub fn new() -> Self {
        let global_env = Rc::new(RefCell::new(HashMap::new()));
        let type_checker = TypeChecker::new();
        let stdlib = StandardLibrary::new();
        let package_manager = PackageManager::new("packages".to_string());
        let language_interop = LanguageInterop::new();

        BellronosInterpreter {
            global_env,
            type_checker,
            stdlib,
            package_manager,
            language_interop,
        }
    }

    pub fn run(&mut self, code: &str, _filename: &str) -> Result<(), BellronosError> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        self.interpret(&ast)?;

        Ok(())
    }

    fn interpret(&mut self, node: &ASTNode) -> Result<Value, BellronosError> {
        match node {
            ASTNode::Module { body } => {
                let mut result = Value::None;
                for stmt in body {
                    result = self.interpret(stmt)?;
                }
                Ok(result)
            }
            ASTNode::Import { names } => {
                for name in names {
                    if let Some(module) = self.stdlib.get_module(name) {
                        self.global_env
                            .borrow_mut()
                            .insert(name.clone(), Value::Dict(module.clone()));
                    } else {
                        let package_code = self.package_manager.load_package(name)?;
                        self.run(&package_code, name)?;
                    }
                }
                Ok(Value::None)
            }
            ASTNode::FunctionDef {
                name,
                args,
                return_type: _,
                body,
            } => {
                let func = Value::Function(
                    args.iter().map(|(name, _)| name.clone()).collect(),
                    body.clone(),
                    Rc::clone(&self.global_env),
                );
                self.global_env.borrow_mut().insert(name.clone(), func);
                Ok(Value::None)
            }
            ASTNode::ClassDef { name, methods } => {
                let mut class_methods = HashMap::new();
                for method in methods {
                    if let ASTNode::FunctionDef {
                        name: method_name,
                        args,
                        return_type: _,
                        body,
                    } = method
                    {
                        let method_func = Value::Function(
                            args.iter().map(|(name, _)| name.clone()).collect(),
                            body.clone(),
                            Rc::clone(&self.global_env),
                        );
                        class_methods.insert(method_name.clone(), method_func);
                    }
                }
                let class = Value::Class {
                    methods: class_methods,
                };
                self.global_env.borrow_mut().insert(name.clone(), class);
                Ok(Value::None)
            }
            ASTNode::Assign { target, value } => {
                let val = self.interpret(value)?;
                self.global_env.borrow_mut().insert(target.clone(), val);
                Ok(Value::None)
            }
            ASTNode::Expr { value } => self.interpret(value),
            ASTNode::Call { func, args } => {
                let f = self.global_env.borrow().get(func).cloned().ok_or_else(|| {
                    BellronosError::Runtime(format!("Undefined function: {}", func))
                })?;
                match f {
                    Value::Function(params, body, env) => {
                        let mut local_env = env.borrow().clone();
                        for (param, arg) in params.iter().zip(args) {
                            local_env.insert(param.clone(), self.interpret(arg)?);
                        }
                        let local_env = Rc::new(RefCell::new(local_env));
                        let mut result = Value::None;
                        for stmt in &body {
                            result = self.interpret_with_env(stmt, &local_env)?;
                        }
                        Ok(result)
                    }
                    Value::Class { methods: _ } => {
                        let instance = Value::Instance {
                            class: func.clone(),
                            attributes: HashMap::new(),
                        };
                        Ok(instance)
                    }
                    _ => Err(BellronosError::Runtime(format!("{} is not callable", func))),
                }
            }
            ASTNode::Str { value } => Ok(Value::String(value.clone())),
            ASTNode::Num { value } => Ok(Value::Float(*value)),
            ASTNode::Bool { value } => Ok(Value::Bool(*value)),
            ASTNode::Name { id } => self
                .global_env
                .borrow()
                .get(id)
                .cloned()
                .ok_or_else(|| BellronosError::Runtime(format!("Undefined variable: {}", id))),
            ASTNode::BinOp { left, op, right } => {
                let left_val = self.interpret(left)?;
                let right_val = self.interpret(right)?;
                match (left_val, op.as_str(), right_val) {
                    (Value::Int(l), "+", Value::Int(r)) => Ok(Value::Int(l + r)),
                    (Value::Float(l), "+", Value::Float(r)) => Ok(Value::Float(l + r)),
                    (Value::String(l), "+", Value::String(r)) => Ok(Value::String(l + &r)),
                    (Value::Int(l), "-", Value::Int(r)) => Ok(Value::Int(l - r)),
                    (Value::Float(l), "-", Value::Float(r)) => Ok(Value::Float(l - r)),
                    (Value::Int(l), "*", Value::Int(r)) => Ok(Value::Int(l * r)),
                    (Value::Float(l), "*", Value::Float(r)) => Ok(Value::Float(l * r)),
                    (Value::Int(l), "/", Value::Int(r)) => Ok(Value::Float(l as f64 / r as f64)),
                    (Value::Float(l), "/", Value::Float(r)) => Ok(Value::Float(l / r)),
                    (Value::Int(l), ">", Value::Int(r)) => Ok(Value::Bool(l > r)),
                    (Value::Float(l), ">", Value::Float(r)) => Ok(Value::Bool(l > r)),
                    (Value::Int(l), "<", Value::Int(r)) => Ok(Value::Bool(l < r)),
                    (Value::Float(l), "<", Value::Float(r)) => Ok(Value::Bool(l < r)),
                    (Value::Int(l), ">=", Value::Int(r)) => Ok(Value::Bool(l >= r)),
                    (Value::Float(l), ">=", Value::Float(r)) => Ok(Value::Bool(l >= r)),
                    (Value::Int(l), "<=", Value::Int(r)) => Ok(Value::Bool(l <= r)),
                    (Value::Float(l), "<=", Value::Float(r)) => Ok(Value::Bool(l <= r)),
                    (Value::Int(l), "==", Value::Int(r)) => Ok(Value::Bool(l == r)),
                    (Value::Float(l), "==", Value::Float(r)) => Ok(Value::Bool(l == r)),
                    (Value::String(l), "==", Value::String(r)) => Ok(Value::Bool(l == r)),
                    (Value::Bool(l), "==", Value::Bool(r)) => Ok(Value::Bool(l == r)),
                    _ => Err(BellronosError::Runtime(format!(
                        "Unsupported operation: {:?} {} {:?}",
                        left, op, right
                    ))),
                }
            }

            ASTNode::If {
                condition,
                body,
                orelse,
            } => {
                let cond_value = self.interpret(condition)?;
                if let Value::Bool(true) = cond_value {
                    for stmt in body {
                        self.interpret(stmt)?;
                    }
                } else {
                    for stmt in orelse {
                        self.interpret(stmt)?;
                    }
                }
                Ok(Value::None)
            }
            ASTNode::While { condition, body } => {
                loop {
                    let cond_value = self.interpret(condition)?;
                    if let Value::Bool(true) = cond_value {
                        for stmt in body {
                            self.interpret(stmt)?;
                        }
                    } else {
                        break;
                    }
                }
                Ok(Value::None)
            }
            ASTNode::For { target, iter, body } => {
                let iter_value = self.interpret(iter)?;
                if let Value::List(items) = iter_value {
                    for item in items {
                        self.global_env.borrow_mut().insert(target.clone(), item);
                        for stmt in body {
                            self.interpret(stmt)?;
                        }
                    }
                } else {
                    return Err(BellronosError::Runtime(
                        "For loop iterable must be a list".to_string(),
                    ));
                }
                Ok(Value::None)
            }
            ASTNode::Return { value } => {
                if let Some(v) = value {
                    self.interpret(v)
                } else {
                    Ok(Value::None)
                }
            }
            ASTNode::Closure { params, body } => Ok(Value::Closure(
                params.iter().map(|(name, _)| name.clone()).collect(),
                vec![*body.clone()],
                Rc::clone(&self.global_env),
            )),
            ASTNode::Generator { body } => Ok(Value::Generator(
                body.clone(),
                Rc::clone(&self.global_env),
                0,
            )),
            ASTNode::Yield { value: _ } => Err(BellronosError::Runtime(
                "Yield outside of generator".to_string(),
            )),
            ASTNode::Async { body } => {
                let mut result = Value::None;
                for stmt in body {
                    result = self.interpret(stmt)?;
                }
                Ok(result)
            }
            ASTNode::Await { value } => self.interpret(value),
            ASTNode::List { elements } => {
                let mut list = Vec::new();
                for elem in elements {
                    list.push(self.interpret(elem)?);
                }
                Ok(Value::List(list))
            }
            ASTNode::Dict { pairs } => {
                let mut dict = HashMap::new();
                for (key, value) in pairs {
                    if let Value::String(k) = self.interpret(key)? {
                        let v = self.interpret(value)?;
                        dict.insert(k, v);
                    } else {
                        return Err(BellronosError::Runtime(
                            "Dictionary keys must be strings".to_string(),
                        ));
                    }
                }
                Ok(Value::Dict(dict))
            }
            ASTNode::Attribute { value, attr } => {
                let obj = self.interpret(value)?;
                match obj {
                    Value::Instance { class, attributes } => {
                        if let Some(attr_value) = attributes.get(attr) {
                            Ok(attr_value.clone())
                        } else if let Some(Value::Class { methods }) =
                            self.global_env.borrow().get(&class)
                        {
                            if let Some(method) = methods.get(attr) {
                                Ok(method.clone())
                            } else {
                                Err(BellronosError::Runtime(format!(
                                    "Attribute '{}' not found on instance of class '{}'",
                                    attr, class
                                )))
                            }
                        } else {
                            Err(BellronosError::Runtime(format!(
                                "Class '{}' not found",
                                class
                            )))
                        }
                    }
                    _ => Err(BellronosError::Runtime(format!(
                        "Cannot access attribute '{}' on non-instance type",
                        attr
                    ))),
                }
            }
            ASTNode::InteropCall { language, code } => {
                self.language_interop.execute(language, code)
            }
        }
    }

    fn interpret_with_env(
        &mut self,
        node: &ASTNode,
        env: &Environment,
    ) -> Result<Value, BellronosError> {
        let old_env = self.global_env.clone();
        *self.global_env.borrow_mut() = env.borrow().clone();
        let result = self.interpret(node);
        *self.global_env.borrow_mut() = old_env.borrow().clone();
        result
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, item) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", item)?;
                }
                write!(f, "]")
            }
            Value::Dict(d) => {
                write!(f, "{{")?;
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {:?}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Function(_, _, _) => write!(f, "<function>"),
            Value::Class { .. } => write!(f, "<class>"),
            Value::Instance { class, .. } => write!(f, "<instance of {}>", class),
            Value::Closure(_, _, _) => write!(f, "<closure>"),
            Value::Generator(_, _, _) => write!(f, "<generator>"),
            Value::None => write!(f, "None"),
        }
    }
}
