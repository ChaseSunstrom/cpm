// src/package_manager.rs
use crate::project::Project;
use log::{info};
use reqwest::blocking::get;
use std::fs::{self, File};
use std::io::copy;
use std::path::Path;
use zip::ZipArchive;

const PACKAGE_REPO_URL: &str = "http://example.com/packages";

pub fn install_project_dependencies(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
    for (dep_name, version) in &project.dependencies {
        install_package(dep_name, version)?;
    }
    Ok(())
}

pub fn install_package(package: &str, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/{}/{}.zip", PACKAGE_REPO_URL, package, version);
    let destination = format!("deps/{}_{}.zip", package, version);
    info!("Downloading {} version {}", package, version);
    download_package(&url, &destination)?;
    info!("Extracting {} version {}", package, version);
    extract_package(&destination, &format!("deps/{}", package))?;
    Ok(())
}

pub fn reinstall_package(package: &str, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let package_path = format!("deps/{}", package);
    if Path::new(&package_path).exists() {
        fs::remove_dir_all(&package_path)?;
    }
    install_package(package, version)
}

fn download_package(url: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = get(url)?;
    fs::create_dir_all("deps")?;
    let mut dest = File::create(destination)?;
    copy(&mut response, &mut dest)?;
    Ok(())
}

fn extract_package(file_path: &str, extract_to: &str) -> zip::result::ZipResult<()> {
    let file = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => Path::new(extract_to).join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
