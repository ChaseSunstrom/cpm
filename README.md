Example project.toml:

```
[[projects]]
name = "MyApp"
type = "Console"
compiler = "MSVC"
language = "C++17"
build_systems = ["VisualStudio"]
visual_studio_version = "17"
output_name = "MyApplication"
character_set = "Unicode"
configurations = ["Debug", "Release"]
platforms = ["x64", "Win32"]

additional_include_dirs = ["third_party/includes"]
preprocessor_definitions = ["USE_FEATURE_X", "ENABLE_LOGGING"]
compiler_flags = ["/W4", "/WX"]
linker_flags = ["/LTCG"]

[projects.structure]
src_dir = "app_src"
include_dir = "app_include"
output_dir = "app_build"
```
