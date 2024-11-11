use crate::project::Project;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn generate_makefile(project: &Project) -> std::io::Result<()> {
    let project_dir = &project.name;
    fs::create_dir_all(project_dir)?;

    // Create project structure directories
    let src_path = Path::new(project_dir).join(&project.structure.src_dir);
    let include_path = Path::new(project_dir).join(&project.structure.include_dir);
    let output_path = Path::new(project_dir).join(&project.structure.output_dir);

    fs::create_dir_all(&src_path)?;
    fs::create_dir_all(&include_path)?;
    fs::create_dir_all(&output_path)?;

    let file_name = format!("{}/Makefile", project_dir);
    let mut file = File::create(&file_name)?;

    // Map language standard
    let language_standard = map_language_standard(&project.language);

    // Collect source files based on custom src_dir
    writeln!(file, "CC={}", project.compiler)?;
    writeln!(file, "CFLAGS=-std={} -Wall -O2", language_standard)?;

    writeln!(
        file,
        "SRCS=$(wildcard {}/{}/*.c)",
        project.name, project.structure.src_dir
    )?;
    writeln!(file, "OBJS=$(SRCS:.c=.o)")?;

    writeln!(file, "LIBS={}", format_dependencies(&project.dependencies))?;

    writeln!(file, "INCLUDES={}", format_include_paths(project))?;
    writeln!(file, "LIBPATHS={}", format_lib_paths(project))?;

    writeln!(file, "TARGET={}/{}", project.structure.output_dir, project.name)?;

    // Create output directory command
    writeln!(file, "all: $(TARGET)")?;
    writeln!(file, "$(TARGET): $(OBJS)")?;
    writeln!(file, "\tmkdir -p {}", project.structure.output_dir)?;
    if project.project_type == "StaticLib" {
        writeln!(file, "\tar rcs $@ $^")?;
    } else if project.project_type == "SharedLib" {
        writeln!(file, "\t$(CC) $(CFLAGS) $(INCLUDES) $(LIBPATHS) -shared -o $@ $^ $(LIBS)")?;
    } else {
        writeln!(file, "\t$(CC) $(CFLAGS) $(INCLUDES) $(LIBPATHS) -o $@ $^ $(LIBS)")?;
    }
    writeln!(file, "clean:")?;
    writeln!(file, "\trm -f $(OBJS) $(TARGET)")?;

    Ok(())
}

fn format_dependencies(dependencies: &std::collections::HashMap<String, String>) -> String {
    dependencies
        .keys()
        .map(|dep| format!("-l{}", dep))
        .collect::<Vec<String>>()
        .join(" ")
}

fn format_include_paths(project: &Project) -> String {
    let mut include_paths = Vec::new();
    include_paths.push(format!("-I{}/{}", project.name, project.structure.include_dir));
    for dep_name in project.dependencies.keys() {
        include_paths.push(format!("-Ideps/{}/include", dep_name));
    }
    include_paths.join(" ")
}

fn format_lib_paths(project: &Project) -> String {
    let mut lib_paths = Vec::new();
    lib_paths.push(format!("-L{}/lib", project.name));
    for dep_name in project.dependencies.keys() {
        lib_paths.push(format!("-Ldeps/{}/lib", dep_name));
    }
    lib_paths.join(" ")
}

fn map_language_standard(language: &str) -> &str {
    match language {
        "C89" => "c89",
        "C99" => "c99",
        "C11" => "c11",
        "C17" => "c17",
        "C++98" => "c++98",
        "C++11" => "c++11",
        "C++14" => "c++14",
        "C++17" => "c++17",
        "C++20" => "c++20",
        "C++23" => "c++23",
        _ => "c11", // Default to C11
    }
}
