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
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Error as IoError, Write};
use std::path::PathBuf;

const PACKAGE_REGISTRY_URL: &str =
    "https://bellande-architecture-mechanism-research-innovation-center.org/bellronos/packages";
const GITHUB_REPO_URL: &str = "https://github.com/Architecture-Mechanism/bellronos_package_manager";

#[derive(Serialize, Deserialize)]
struct PackageMetadata {
    name: String,
    version: String,
    dependencies: Vec<String>,
}

pub struct PackageManager {
    package_dir: PathBuf,
}

impl PackageManager {
    pub fn new(package_dir: String) -> Self {
        PackageManager {
            package_dir: PathBuf::from(package_dir),
        }
    }

    pub fn install_package(&self, package_name: &str) -> Result<(), BellronosError> {
        println!("Installing package: {}", package_name);

        // Fetch package metadata
        let metadata = self.fetch_package_metadata(package_name)?;

        // Download package
        let package_content = self.download_package(&metadata)?;

        // Install dependencies
        for dependency in &metadata.dependencies {
            self.install_package(dependency)?;
        }

        // Write package content
        let package_path = self
            .package_dir
            .join(&metadata.name)
            .with_extension("bellronos");
        let mut file = File::create(&package_path).map_err(|e| {
            BellronosError::IO(IoError::new(
                e.kind(),
                format!("Failed to create package file: {}", e),
            ))
        })?;
        file.write_all(package_content.as_bytes()).map_err(|e| {
            BellronosError::IO(IoError::new(
                e.kind(),
                format!("Failed to write package content: {}", e),
            ))
        })?;

        println!("Successfully installed package: {}", package_name);
        Ok(())
    }

    pub fn load_package(&self, package_name: &str) -> Result<String, BellronosError> {
        let package_path = self
            .package_dir
            .join(package_name)
            .with_extension("bellronos");
        fs::read_to_string(&package_path).map_err(|e| {
            BellronosError::IO(IoError::new(
                e.kind(),
                format!(
                    "Failed to read package file {}: {}",
                    package_path.display(),
                    e
                ),
            ))
        })
    }

    pub fn list_installed_packages(&self) -> Result<Vec<String>, BellronosError> {
        let entries = fs::read_dir(&self.package_dir).map_err(|e| {
            BellronosError::IO(IoError::new(
                e.kind(),
                format!(
                    "Failed to read package directory {}: {}",
                    self.package_dir.display(),
                    e
                ),
            ))
        })?;

        let mut packages = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                BellronosError::IO(IoError::new(
                    e.kind(),
                    format!("Failed to read directory entry: {}", e),
                ))
            })?;
            if let Some(file_name) = entry.path().file_stem() {
                if let Some(name) = file_name.to_str() {
                    packages.push(name.to_string());
                }
            }
        }

        Ok(packages)
    }

    fn fetch_package_metadata(
        &self,
        package_name: &str,
    ) -> Result<PackageMetadata, BellronosError> {
        let url = format!("{}/{}/metadata.json", PACKAGE_REGISTRY_URL, package_name);
        let response = reqwest::blocking::get(&url).map_err(|e| {
            BellronosError::Network(format!("Failed to fetch package metadata: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(BellronosError::Network(format!(
                "Failed to fetch package metadata. Status: {}",
                response.status()
            )));
        }

        response
            .json::<PackageMetadata>()
            .map_err(|e| BellronosError::Parser(format!("Failed to parse package metadata: {}", e)))
    }

    fn download_package(&self, metadata: &PackageMetadata) -> Result<String, BellronosError> {
        let url = format!(
            "{}/{}/{}.bellronos",
            PACKAGE_REGISTRY_URL, metadata.name, metadata.version
        );
        let response = reqwest::blocking::get(&url)
            .map_err(|e| BellronosError::Network(format!("Failed to download package: {}", e)))?;

        if !response.status().is_success() {
            return Err(BellronosError::Network(format!(
                "Failed to download package. Status: {}",
                response.status()
            )));
        }

        response
            .text()
            .map_err(|e| BellronosError::Network(format!("Failed to read package content: {}", e)))
    }

    pub fn update_package(&self, package_name: &str) -> Result<(), BellronosError> {
        let installed_packages = self.list_installed_packages()?;
        if !installed_packages.contains(&package_name.to_string()) {
            return Err(BellronosError::Package(format!(
                "Package {} is not installed",
                package_name
            )));
        }

        // Fetch latest metadata
        let metadata = self.fetch_package_metadata(package_name)?;

        // Check if update is needed
        let current_version = self.get_installed_package_version(package_name)?;
        if current_version == metadata.version {
            println!("Package {} is already up to date", package_name);
            return Ok(());
        }

        // Perform update
        self.install_package(package_name)
    }

    fn get_installed_package_version(&self, package_name: &str) -> Result<String, BellronosError> {
        let package_path = self
            .package_dir
            .join(package_name)
            .with_extension("bellronos");
        let content = fs::read_to_string(&package_path).map_err(|e| {
            BellronosError::IO(IoError::new(
                e.kind(),
                format!(
                    "Failed to read package file {}: {}",
                    package_path.display(),
                    e
                ),
            ))
        })?;

        // Extract version from package content
        content
            .lines()
            .find(|line| line.starts_with("# Version:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|version| version.trim().to_string())
            .ok_or_else(|| BellronosError::Parser("Failed to extract package version".to_string()))
    }

    pub fn search_packages(&self, query: &str) -> Result<Vec<String>, BellronosError> {
        let url = format!("{}/search?q={}", PACKAGE_REGISTRY_URL, query);
        let response = reqwest::blocking::get(&url)
            .map_err(|e| BellronosError::Network(format!("Failed to search packages: {}", e)))?;

        if !response.status().is_success() {
            return Err(BellronosError::Network(format!(
                "Failed to search packages. Status: {}",
                response.status()
            )));
        }

        response
            .json::<Vec<String>>()
            .map_err(|e| BellronosError::Parser(format!("Failed to parse search results: {}", e)))
    }

    pub fn get_package_info(&self, package_name: &str) -> Result<String, BellronosError> {
        let url = format!("{}/{}/README.md", GITHUB_REPO_URL, package_name);
        let response = reqwest::blocking::get(&url)
            .map_err(|e| BellronosError::Network(format!("Failed to fetch package info: {}", e)))?;

        if !response.status().is_success() {
            return Err(BellronosError::Network(format!(
                "Failed to fetch package info. Status: {}",
                response.status()
            )));
        }

        response
            .text()
            .map_err(|e| BellronosError::Network(format!("Failed to read package info: {}", e)))
    }
}
