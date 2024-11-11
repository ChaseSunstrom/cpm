// src/compiler.rs
pub struct Compiler {
    pub name: String,
    pub command: String,
    pub flags: String,
}

impl Compiler {
    pub fn new(name: &str) -> Self {
        match name {
            "gcc" | "G++" => Compiler {
                name: "GCC".to_string(),
                command: "g++".to_string(),
                flags: "-Wall -O2".to_string(),
            },
            "clang" | "Clang" => Compiler {
                name: "Clang".to_string(),
                command: "clang++".to_string(),
                flags: "-Wall -O2".to_string(),
            },
            "MSVC" => Compiler {
                name: "MSVC".to_string(),
                command: "cl".to_string(),
                flags: "/EHsc".to_string(),
            },
            _ => Compiler {
                name: "GCC".to_string(),
                command: "g++".to_string(),
                flags: "-Wall -O2".to_string(),
            },
        }
    }
}
