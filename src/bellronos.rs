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

mod ast;
mod error;
mod interop;
mod interpreter;
mod lexer;
mod package_manager;
mod parser;
mod standard_library;
mod type_system;

use error::error::BellronosError;
use interpreter::interpreter::BellronosInterpreter;
use package_manager::package_manager::PackageManager;
use std::env;
use std::fs;

#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

fn main() -> Result<(), BellronosError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: bellronos <filename> [--install <package>]");
        return Ok(());
    }

    if args[1] == "--install" && args.len() == 3 {
        let package_manager = PackageManager::new("packages".to_string());
        package_manager.install_package(&args[2])?;
        println!("Package {} installed successfully", args[2]);
        return Ok(());
    }

    let filename = &args[1];
    let contents = fs::read_to_string(filename)?;

    let mut interpreter = BellronosInterpreter::new();
    interpreter.run(&contents, filename)?;

    Ok(())
}
