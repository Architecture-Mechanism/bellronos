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
use crate::interpreter::interpreter::Value;
use std::process::Command;

pub struct LanguageInterop;

impl LanguageInterop {
    pub fn new() -> Self {
        LanguageInterop
    }

    pub fn execute(&self, language: &str, code: &str) -> Result<Value, BellronosError> {
        match language {
            "c" => self.execute_c(code),
            "python" => self.execute_python(code),
            "javascript" => self.execute_javascript(code),
            "java" => self.execute_java(code),
            "rust" => self.execute_rust(code),
            "swift" => self.execute_swift(code),
            _ => Err(BellronosError::Runtime(format!(
                "Unsupported language: {}",
                language
            ))),
        }
    }

    fn execute_c(&self, code: &str) -> Result<Value, BellronosError> {
        let temp_file = "temp.c";
        std::fs::write(temp_file, code)?;

        let output = Command::new("gcc")
            .args(&[temp_file, "-o", "temp_c"])
            .output()?;

        if !output.status.success() {
            return Err(BellronosError::Runtime(format!(
                "C compilation error: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output = Command::new("./temp_c").output()?;
        std::fs::remove_file(temp_file)?;
        std::fs::remove_file("temp_c")?;

        Ok(Value::String(
            String::from_utf8_lossy(&output.stdout).to_string(),
        ))
    }

    fn execute_python(&self, code: &str) -> Result<Value, BellronosError> {
        let output = Command::new("python").arg("-c").arg(code).output()?;

        Ok(Value::String(
            String::from_utf8_lossy(&output.stdout).to_string(),
        ))
    }

    fn execute_javascript(&self, code: &str) -> Result<Value, BellronosError> {
        let output = Command::new("node").arg("-e").arg(code).output()?;

        Ok(Value::String(
            String::from_utf8_lossy(&output.stdout).to_string(),
        ))
    }

    fn execute_java(&self, code: &str) -> Result<Value, BellronosError> {
        let temp_file = "Temp.java";
        std::fs::write(temp_file, code)?;

        let output = Command::new("javac").arg(temp_file).output()?;

        if !output.status.success() {
            return Err(BellronosError::Runtime(format!(
                "Java compilation error: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output = Command::new("java").arg("Temp").output()?;

        std::fs::remove_file(temp_file)?;
        std::fs::remove_file("Temp.class")?;

        Ok(Value::String(
            String::from_utf8_lossy(&output.stdout).to_string(),
        ))
    }

    fn execute_rust(&self, code: &str) -> Result<Value, BellronosError> {
        let temp_file = "temp.rs";
        std::fs::write(temp_file, code)?;

        let output = Command::new("rustc").arg(temp_file).output()?;

        if !output.status.success() {
            return Err(BellronosError::Runtime(format!(
                "Rust compilation error: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output = Command::new("./temp").output()?;

        std::fs::remove_file(temp_file)?;
        std::fs::remove_file("temp")?;

        Ok(Value::String(
            String::from_utf8_lossy(&output.stdout).to_string(),
        ))
    }

    fn execute_swift(&self, code: &str) -> Result<Value, BellronosError> {
        let temp_file = "temp.swift";
        std::fs::write(temp_file, code)?;

        let output = Command::new("swift").arg(temp_file).output()?;

        std::fs::remove_file(temp_file)?;

        if output.status.success() {
            Ok(Value::String(
                String::from_utf8_lossy(&output.stdout).to_string(),
            ))
        } else {
            Err(BellronosError::Runtime(format!(
                "Swift execution error: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}
