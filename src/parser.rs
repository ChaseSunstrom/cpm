// src/parser.rs
use crate::project::ProjectsFile;
use std::fs;

pub fn parse_project_file(file_path: &str) -> Result<ProjectsFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let projects_file: ProjectsFile = toml::from_str(&content)?;
    Ok(projects_file)
}
