// src/main.rs
mod parser;
mod project;
mod build_systems;
mod package_manager;
mod dependency_resolver;
mod compiler;

use clap::{Arg, Command};
use env_logger;

use crate::project::Project;
use crate::parser::parse_project_file;
use crate::dependency_resolver::resolve_dependencies;
use crate::package_manager::{install_project_dependencies, reinstall_package};
use crate::build_systems::{generate_build_configs};

use std::path::PathBuf;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let matches = Command::new("CPM")
        .version("1.0")
        .author("Chase Sunstrom <casunstrom@gmail.com>")
        .about("Package Manager and Build Tool")
        .subcommand(
            Command::new("generate")
                .about("Generates build configurations")
                .arg(
                    Arg::new("project_file")
                        .short('f')
                        .help("Path to the project file")
                        .value_name("FILE")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("build")
                .about("Builds the projects")
                .arg(
                    Arg::new("project_file")
                        .short('f')
                        .help("Path to the project file")
                        .value_name("FILE")
                        .num_args(1),
                )
                .arg(
                    Arg::new("build_system")
                        .short('b')
                        .help("Build system to use for building")
                        .value_name("BUILD_SYSTEM")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("install")
                .about("Installs dependencies for the projects")
                .arg(
                    Arg::new("project_file")
                        .short('f')
                        .help("Path to the project file")
                        .value_name("FILE")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("reinstall")
                .about("Reinstalls a package with a different version")
                .arg(
                    Arg::new("package_name")
                        .help("Name of the package to reinstall")
                        .required(true)
                        .value_name("PACKAGE")
                        .num_args(1),
                )
                .arg(
                    Arg::new("version")
                        .help("Version of the package to reinstall")
                        .required(true)
                        .value_name("VERSION")
                        .num_args(1),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        let file_path = matches
            .get_one::<String>("project_file")
            .map_or("project.toml", |s| s.as_str());
        match parse_project_file(file_path) {
            Ok(projects_file) => {
                for project in &projects_file.projects {
                    if let Err(e) = generate_build_configs(project) {
                        eprintln!("Error generating build configs for {}: {}", project.name, e);
                    }
                }
            }
            Err(e) => eprintln!("Error parsing project file: {}", e),
        }
    } else if let Some(matches) = matches.subcommand_matches("build") {
        let file_path = matches
            .get_one::<String>("project_file")
            .map_or("project.toml", |s| s.as_str());
        let build_system = matches.get_one::<String>("build_system");

        match parse_project_file(file_path) {
            Ok(projects_file) => {
                let build_order = resolve_dependencies(&projects_file.projects);
                for project in build_order {
                    if let Err(e) = install_project_dependencies(project) {
                        eprintln!("Error installing dependencies for {}: {}", project.name, e);
                        continue;
                    }
                    if let Err(e) = generate_build_configs(project) {
                        eprintln!("Error generating build configs for {}: {}", project.name, e);
                        continue;
                    }
                    if let Err(e) = build_project_with_system(project, build_system) {
                        eprintln!("Error building {}: {}", project.name, e);
                    }
                }
            }
            Err(e) => eprintln!("Error parsing project file: {}", e),
        }
    } else if let Some(matches) = matches.subcommand_matches("install") {
        let file_path = matches
            .get_one::<String>("project_file")
            .map_or("project.toml", |s| s.as_str());
        match parse_project_file(file_path) {
            Ok(projects_file) => {
                for project in &projects_file.projects {
                    if let Err(e) = install_project_dependencies(project) {
                        eprintln!("Error installing dependencies for {}: {}", project.name, e);
                    }
                }
            }
            Err(e) => eprintln!("Error parsing project file: {}", e),
        }
    } else if let Some(matches) = matches.subcommand_matches("reinstall") {
        let package_name = matches
            .get_one::<String>("package_name")
            .expect("package_name is required");
        let version = matches
            .get_one::<String>("version")
            .expect("version is required");
        if let Err(e) = reinstall_package(package_name, version) {
            eprintln!("Error reinstalling package {}: {}", package_name, e);
        }
    } else {
        println!("No valid subcommand was provided. Use --help for more information.");
    }
}

fn build_project_with_system(
    project: &Project,
    build_system: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let system = build_system
        .as_deref()
        .or_else(|| project.build_systems.first())
        .map(String::as_str)
        .unwrap_or("Makefile"); // Default to Makefile if not specified

    match system {
        "Makefile" => {
            // Run Makefile
            let status = std::process::Command::new("make")
                .current_dir(&project.name)
                .status()?;
            if !status.success() {
                return Err(format!("Build failed for {}", project.name).into());
            }
        }
        "VisualStudio" => {
            // Run msbuild.exe
            let solution_file = format!("{}/{}.sln", project.name, project.name);
            let status = std::process::Command::new("msbuild.exe")
                .arg(&solution_file)
                .status()?;
            if !status.success() {
                return Err(format!("Build failed for {}", project.name).into());
            }
        }
        _ => {
            eprintln!("Unsupported build system: {}", system);
        }
    }
    Ok(())
}


// Function to locate msbuild.exe
fn find_msbuild_executable() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Common installation paths for msbuild.exe
    let possible_paths = vec![
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\MSBuild\Current\Bin\MSBuild.exe",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\MSBuild\Current\Bin\MSBuild.exe",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\MSBuild\Current\Bin\MSBuild.exe",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\MSBuild.exe",
        r"C:\Program Files\Microsoft Visual Studio\2022\Professional\MSBuild\Current\Bin\MSBuild.exe",
        r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\MSBuild\Current\Bin\MSBuild.exe",
    ];

    for path_str in possible_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    Err("msbuild.exe not found. Please ensure Visual Studio is installed, or add msbuild.exe to your PATH.".into())
}
