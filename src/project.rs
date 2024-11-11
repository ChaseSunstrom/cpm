// src/project.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ProjectsFile {
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub compiler: String,
    pub language: String,
    #[serde(default)]
    pub build_systems: Vec<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub structure: ProjectStructure,
    #[serde(default = "default_visual_studio_version")]
    pub visual_studio_version: String,
    #[serde(default)]
    pub additional_include_dirs: Vec<String>,
    #[serde(default)]
    pub preprocessor_definitions: Vec<String>,
    #[serde(default)]
    pub compiler_flags: Vec<String>,
    #[serde(default)]
    pub linker_flags: Vec<String>,
    #[serde(default = "default_configurations")]
    pub configurations: Vec<String>,
    #[serde(default = "default_platforms")]
    pub platforms: Vec<String>,
    #[serde(default = "default_character_set")]
    pub character_set: String, // e.g., "Unicode" or "MultiByte"
    #[serde(default)]
    pub output_name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectStructure {
    #[serde(default = "default_src_dir")]
    pub src_dir: String,
    #[serde(default = "default_include_dir")]
    pub include_dir: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_src_dir() -> String {
    "src".to_string()
}

fn default_include_dir() -> String {
    "include".to_string()
}

fn default_output_dir() -> String {
    "build".to_string()
}

fn default_visual_studio_version() -> String {
    "17".to_string() // Default to Visual Studio 2022
}

fn default_configurations() -> Vec<String> {
    vec!["Debug".to_string(), "Release".to_string()]
}

fn default_platforms() -> Vec<String> {
    vec!["x64".to_string()]
}

fn default_character_set() -> String {
    "Unicode".to_string()
}
