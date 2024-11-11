// src/build_systems/visual_studio_generator.rs
use crate::project::Project;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

pub fn generate_visual_studio(project: &Project) -> std::io::Result<()> {
    let project_dir = &project.name;
    fs::create_dir_all(project_dir)?;

    // Create project structure directories
    let src_path = Path::new(project_dir).join(&project.structure.src_dir);
    let include_path = Path::new(project_dir).join(&project.structure.include_dir);
    let output_path = Path::new(project_dir).join(&project.structure.output_dir);
    let intermediate_path = output_path.join("Intermediate");

    fs::create_dir_all(&src_path)?;
    fs::create_dir_all(&include_path)?;
    fs::create_dir_all(&output_path)?;
    fs::create_dir_all(&intermediate_path)?;

    // Generate .sln file
    let project_guid = Uuid::new_v4();
    let vs_version = &project.visual_studio_version;
    let (format_version, vs_version_name, platform_toolset) = match vs_version.as_str() {
        "15" => ("12.00", "# Visual Studio 15", "v141"), // Visual Studio 2017
        "16" => ("12.00", "# Visual Studio Version 16", "v142"), // Visual Studio 2019
        "17" => ("12.00", "# Visual Studio Version 17", "v143"), // Visual Studio 2022
        _ => ("12.00", "# Visual Studio Version 17", "v143"),    // Default to VS 2022
    };

    // Correct C++ Project Type GUID
    let cpp_project_type_guid = "8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942";

    // Generate project configurations for .sln
    let mut solution_configurations = String::new();
    let mut project_configuration_platforms = String::new();
    for platform in &project.platforms {
        for configuration in &project.configurations {
            solution_configurations.push_str(&format!(
                "        {0}|{1} = {0}|{1}\n",
                configuration, platform
            ));
            project_configuration_platforms.push_str(&format!(
                "        {{{project_guid}}}.{0}|{1}.ActiveCfg = {0}|{1}\n",
                configuration, platform,
                project_guid = project_guid
            ));
            project_configuration_platforms.push_str(&format!(
                "        {{{project_guid}}}.{0}|{1}.Build.0 = {0}|{1}\n",
                configuration, platform,
                project_guid = project_guid
            ));
        }
    }

    let solution_content = format!(
        r#"Microsoft Visual Studio Solution File, Format Version {}
{}
Project("{{{project_type_guid}}}") = "{project_name}", "{project_name}.vcxproj", "{{{project_guid}}}"
EndProject
Global
    GlobalSection(SolutionConfigurationPlatforms) = preSolution
{solution_configurations}
    EndGlobalSection
    GlobalSection(ProjectConfigurationPlatforms) = postSolution
{project_configuration_platforms}
    EndGlobalSection
EndGlobal
"#,
        format_version,
        vs_version_name,
        project_type_guid = cpp_project_type_guid,
        project_name = project.name,
        project_guid = project_guid,
        solution_configurations = solution_configurations,
        project_configuration_platforms = project_configuration_platforms
    );

    let sln_path = format!("{}/{}.sln", project_dir, project.name);
    let mut sln_file = File::create(&sln_path)?;
    sln_file.write_all(solution_content.as_bytes())?;

    // Generate .vcxproj file
    let mut includes = vec![project.structure.include_dir.clone()];
    includes.extend(project.additional_include_dirs.clone());
    includes.extend(
        project
            .dependencies
            .keys()
            .map(|dep| format!("..\\deps\\{}\\include", dep)),
    );
    let additional_includes = includes.join(";");

    let mut libraries = Vec::new();
    libraries.extend(
        project
            .dependencies
            .keys()
            .map(|dep| format!("..\\deps\\{}\\lib", dep)),
    );
    let additional_libraries = libraries.join(";");

    let preprocessor_definitions = if !project.preprocessor_definitions.is_empty() {
        project.preprocessor_definitions.join(";") + ";%(PreprocessorDefinitions)"
    } else {
        "%(PreprocessorDefinitions)".to_string()
    };

    let compiler_flags = project.compiler_flags.join(" ");
    let linker_flags = project.linker_flags.join(" ");

    // Generate ItemGroup for source and header files
    let source_files = format!(
        r#"<ClCompile Include="{}\\**\\*.cpp" />"#,
        project.structure.src_dir
    );

    let header_files = format!(
        r#"<ClInclude Include="{}\\**\\*.h" />"#,
        project.structure.include_dir
    );

    // Generate PropertyGroups and ItemDefinitionGroups for each configuration and platform
    let mut property_groups = String::new();
    let mut item_definition_groups = String::new();
    for platform in &project.platforms {
        for configuration in &project.configurations {
            // PropertyGroup
            let use_debug_libraries = if configuration.to_lowercase() == "debug" {
                "true"
            } else {
                "false"
            };
            let whole_program_optimization = if configuration.to_lowercase() == "release" {
                "true"
            } else {
                "false"
            };
            let optimization = if configuration.to_lowercase() == "release" {
                "MaxSpeed"
            } else {
                "Disabled"
            };

            property_groups.push_str(&format!(
                r#"<PropertyGroup Condition="'$(Configuration)|$(Platform)'=='{configuration}|{platform}'" Label="Configuration">
    <ConfigurationType>{config_type}</ConfigurationType>
    <UseDebugLibraries>{use_debug_libraries}</UseDebugLibraries>
    <PlatformToolset>{platform_toolset}</PlatformToolset>
    <WholeProgramOptimization>{whole_program_optimization}</WholeProgramOptimization>
    <CharacterSet>{character_set}</CharacterSet>
  </PropertyGroup>
"#,
                configuration = configuration,
                platform = platform,
                config_type = map_configuration_type(&project.project_type),
                use_debug_libraries = use_debug_libraries,
                platform_toolset = platform_toolset,
                whole_program_optimization = whole_program_optimization,
                character_set = project.character_set,
            ));

            // ItemDefinitionGroup
            item_definition_groups.push_str(&format!(
                r#"<ItemDefinitionGroup Condition="'$(Configuration)|$(Platform)'=='{configuration}|{platform}'">
    <ClCompile>
      <WarningLevel>Level3</WarningLevel>
      <Optimization>{optimization}</Optimization>
      <PreprocessorDefinitions>{preprocessor_definitions}</PreprocessorDefinitions>
      <AdditionalIncludeDirectories>{includes};%(AdditionalIncludeDirectories)</AdditionalIncludeDirectories>
      <AdditionalOptions>{compiler_flags} %(AdditionalOptions)</AdditionalOptions>
      <LanguageStandard>{language_standard}</LanguageStandard>
    </ClCompile>
    <Link>
      <SubSystem>{subsystem}</SubSystem>
      <GenerateDebugInformation>true</GenerateDebugInformation>
      <AdditionalLibraryDirectories>{libraries};%(AdditionalLibraryDirectories)</AdditionalLibraryDirectories>
      <AdditionalDependencies>kernel32.lib;user32.lib;{}</AdditionalDependencies>
      <AdditionalOptions>{linker_flags} %(AdditionalOptions)</AdditionalOptions>
    </Link>
  </ItemDefinitionGroup>
"#,
                configuration = configuration,
                platform = platform,
                optimization = optimization,
                preprocessor_definitions = preprocessor_definitions,
                includes = additional_includes,
                compiler_flags = compiler_flags,
                language_standard = map_language_standard(&project.language),
                subsystem = map_subsystem(&project.project_type),
                libraries = additional_libraries,
                linker_flags = linker_flags,
                // You can add more default dependencies if needed
            ));
        }
    }

    let output_name = project.output_name.clone().unwrap_or_else(|| project.name.clone());

    // Generate ProjectConfigurations and PropertySheets
    let project_configurations = generate_project_configurations(&project);
    let property_sheets = generate_property_sheets(&project);

    let solution_dir = Path::new(project_dir).canonicalize()?.to_str().unwrap().to_string();

    let project_content = format!(
        r#"<Project DefaultTargets="Build" xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
  <ItemGroup Label="ProjectConfigurations">
{project_configurations}
  </ItemGroup>
  <PropertyGroup Label="Globals">
    <ProjectGuid>{{{project_guid}}}</ProjectGuid>
    <RootNamespace>{project_name}</RootNamespace>
    <WindowsTargetPlatformVersion>10.0</WindowsTargetPlatformVersion>
  </PropertyGroup>
  <Import Project="$(VCTargetsPath)\Microsoft.Cpp.Default.props" />
{property_groups}
  <Import Project="$(VCTargetsPath)\Microsoft.Cpp.props" />
  <ImportGroup Label="ExtensionSettings">
  </ImportGroup>
  <ImportGroup Label="Shared">
  </ImportGroup>
{property_sheets}
  <PropertyGroup Label="UserMacros" />
  <PropertyGroup>
    <OutDir>{solution_dir}\\{output_dir}\\</OutDir>
    <IntDir>{solution_dir}\\{output_dir}\\Intermediate\\</IntDir>
    <TargetName>{output_name}</TargetName>
    <TargetPath>{solution_dir}\\{output_dir}\\{output_name}.exe</TargetPath>
    <LinkIncremental>false</LinkIncremental>
  </PropertyGroup>
{item_definition_groups}
  <ItemGroup>
    {source_files}
  </ItemGroup>
  <ItemGroup>
    {header_files}
  </ItemGroup>
  <Import Project="$(VCTargetsPath)\Microsoft.Cpp.targets" />
  <ImportGroup Label="ExtensionTargets">
  </ImportGroup>
</Project>
"#,
        project_guid = project_guid,
        project_name = project.name,
        project_configurations = project_configurations,
        property_groups = property_groups,
        property_sheets = property_sheets,
        solution_dir = solution_dir,
        output_dir = project.structure.output_dir,
        output_name = output_name,
        item_definition_groups = item_definition_groups,
        source_files = source_files,
        header_files = header_files,
    );

    let proj_path = format!("{}/{}.vcxproj", project_dir, project.name);
    let mut proj_file = File::create(&proj_path)?;
    proj_file.write_all(project_content.as_bytes())?;

    Ok(())
}

fn generate_project_configurations(project: &Project) -> String {
    let mut configurations = String::new();
    for platform in &project.platforms {
        for configuration in &project.configurations {
            configurations.push_str(&format!(
                r#"    <ProjectConfiguration Include="{configuration}|{platform}">
      <Configuration>{configuration}</Configuration>
      <Platform>{platform}</Platform>
    </ProjectConfiguration>
"#,
                configuration = configuration,
                platform = platform,
            ));
        }
    }
    configurations
}

fn generate_property_sheets(project: &Project) -> String {
    let mut property_sheets = String::new();
    for platform in &project.platforms {
        for configuration in &project.configurations {
            property_sheets.push_str(&format!(
                r#"<ImportGroup Label="PropertySheets" Condition="'$(Configuration)|$(Platform)'=='{configuration}|{platform}'">
    <Import Project="$(UserRootDir)\Microsoft.Cpp.$(Platform).user.props"
            Condition="exists('$(UserRootDir)\Microsoft.Cpp.$(Platform).user.props')"
            Label="LocalAppDataPlatform" />
  </ImportGroup>
"#,
                configuration = configuration,
                platform = platform,
            ));
        }
    }
    property_sheets
}

fn map_configuration_type(project_type: &str) -> &str {
    match project_type {
        "Console" => "Application",
        "StaticLib" => "StaticLibrary",
        "SharedLib" => "DynamicLibrary",
        _ => "Application",
    }
}

fn map_language_standard(language: &str) -> &str {
    match language {
        "C89" => "stdc89",
        "C99" => "stdc99",
        "C11" => "stdc11",
        "C17" => "stdc17",
        "C++98" => "stdcpp98",
        "C++11" => "stdcpp11",
        "C++14" => "stdcpp14",
        "C++17" => "stdcpp17",
        "C++20" => "stdcpp20",
        "C++23" => "stdcpplatest",
        _ => "stdcpp17",
    }
}

fn map_subsystem(project_type: &str) -> &str {
    match project_type {
        "Console" => "Console",
        "SharedLib" => "Windows",
        "StaticLib" => "Console", // Static libraries don't produce an executable
        _ => "Console",
    }
}
