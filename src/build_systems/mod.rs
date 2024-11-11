// src/build_systems/mod.rs
pub mod makefile_generator;
pub mod visual_studio_generator;

use crate::project::Project;
use crate::build_systems::makefile_generator::generate_makefile;
use crate::build_systems::visual_studio_generator::generate_visual_studio;

pub fn generate_build_configs(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
    for build_system in &project.build_systems {
        match build_system.as_str() {
            "Makefile" => generate_makefile(project)?,
            "VisualStudio" => generate_visual_studio(project)?,
            _ => eprintln!("Unsupported build system: {}", build_system),
        }
    }
    Ok(())
}

pub fn build_project(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
    // Determine which build system to use for building
    // For simplicity, we'll use the first build system specified
    if let Some(build_system) = project.build_systems.first() {
        match build_system.as_str() {
            "Makefile" => {
                // Run Makefile
                let status = std::process::Command::new("make")
                    .current_dir(&project.name)
                    .status()?;
                if !status.success() {
                    return Err("Build failed".into());
                }
            }
            "VisualStudio" => {
                // Run msbuild.exe
                let solution_file = format!("{}.sln", project.name);
                let status = std::process::Command::new("msbuild.exe")
                    .arg(solution_file)
                    .current_dir(&project.name)
                    .status();

                match status {
                    Ok(status) => {
                        if !status.success() {
                            return Err(format!("Build failed for {} with exit code {:?}", project.name, status.code()).into());
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to execute msbuild.exe: {}", e).into());
                    }
                }
            }
            _ => {
                eprintln!("Unsupported build system: {}", build_system);
            }
        }
    } else {
        eprintln!("No build systems specified for project {}", project.name);
    }
    Ok(())
}
