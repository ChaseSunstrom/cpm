// src/dependency_resolver.rs
use crate::project::Project;
use std::collections::{HashMap, HashSet};

pub fn resolve_dependencies<'a>(projects: &'a [Project]) -> Vec<&'a Project> {
    let mut build_order = Vec::new();
    let mut visited = HashSet::new();
    let project_map = projects
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect::<HashMap<_, _>>();

    for project in projects {
        visit(project, &mut visited, &mut build_order, &project_map);
    }

    build_order
}

fn visit<'a>(
    project: &'a Project,
    visited: &mut HashSet<String>,
    build_order: &mut Vec<&'a Project>,
    project_map: &HashMap<String, &'a Project>,
) {
    if visited.contains(&project.name) {
        return;
    }
    visited.insert(project.name.clone());

    for dep_name in project.dependencies.keys() {
        if let Some(dep_project) = project_map.get(dep_name) {
            visit(dep_project, visited, build_order, project_map);
        }
    }
    build_order.push(project);
}
