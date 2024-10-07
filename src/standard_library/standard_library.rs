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

use crate::interpreter::interpreter::Value;
use std::collections::HashMap;

pub struct StandardLibrary {
    modules: HashMap<String, HashMap<String, Value>>,
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = StandardLibrary {
            modules: HashMap::new(),
        };
        stdlib.init_math();
        stdlib.init_io();
        stdlib.init_string();
        stdlib
    }

    fn init_math(&mut self) {
        let mut math = HashMap::new();
        math.insert("pi".to_string(), Value::Float(std::f64::consts::PI));
        math.insert("e".to_string(), Value::Float(std::f64::consts::E));
        math.insert(
            "sqrt".to_string(),
            Value::Function(
                vec!["x".to_string()],
                vec![],
                std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            ),
        );
        self.modules.insert("math".to_string(), math);
    }

    fn init_io(&mut self) {
        let mut io = HashMap::new();
        io.insert(
            "print".to_string(),
            Value::Function(
                vec!["args".to_string()],
                vec![],
                std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            ),
        );
        io.insert(
            "input".to_string(),
            Value::Function(
                vec!["prompt".to_string()],
                vec![],
                std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            ),
        );
        self.modules.insert("io".to_string(), io);
    }

    fn init_string(&mut self) {
        let mut string = HashMap::new();
        string.insert(
            "length".to_string(),
            Value::Function(
                vec!["s".to_string()],
                vec![],
                std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            ),
        );
        string.insert(
            "to_upper".to_string(),
            Value::Function(
                vec!["s".to_string()],
                vec![],
                std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            ),
        );
        string.insert(
            "to_lower".to_string(),
            Value::Function(
                vec!["s".to_string()],
                vec![],
                std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            ),
        );
        self.modules.insert("string".to_string(), string);
    }

    pub fn get_module(&self, name: &str) -> Option<&HashMap<String, Value>> {
        self.modules.get(name)
    }
}
