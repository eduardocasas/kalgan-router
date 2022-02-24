use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_yaml::Value;
use std::ffi::OsStr;
use std::{fs, path::Path, vec::Vec};

pub fn generate(source: &str) -> crate::Router {
    info!("");
    info!("***********************************************************************************");
    info!("Crate: Router");
    info!("Start Route Generation");
    let mut router = crate::Router {
        collection: Vec::new(),
    };
    if Path::new(source).exists() {
        if Path::new(&source).is_dir() {
            walk_folder_files(Path::new(&source), &mut router.collection);
        } else {
            get_file_content(Path::new(&source), &mut router.collection);
        }
    } else {
        error!("Source path {} not found.", source);
    }
    match router.collection.len() {
        0 => info!("Result: No routes have been parsed"),
        1 => info!("Result: 1 route has been parsed"),
        _ => info!(
            "Result: {} routes have been parsed",
            router.collection.len()
        ),
    }
    info!("End Route Generation");
    info!("***********************************************************************************");
    router
}
fn walk_folder_files(dir: &Path, routes: &mut Vec<crate::Route>) {
    debug!("Reading folder {}...", dir.display());
    match fs::read_dir(dir) {
        Ok(read_dir) => {
            for entry in read_dir {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    debug!("{} is dir", path.display());
                    walk_folder_files(&path, routes);
                } else {
                    match path.extension().and_then(OsStr::to_str) {
                        Some(extension) => {
                            if extension.to_lowercase() == "yaml" {
                                debug!("{} is file", path.display());
                                get_file_content(&path, routes);
                            } else {
                                debug!("{} is skipped.", path.display());
                            }
                        }
                        None => debug!("{} is skipped.", path.display()),
                    }
                }
            }
        }
        Err(e) => warn!("{}", e),
    }
}
fn get_file_content(path: &Path, routes: &mut Vec<crate::Route>) {
    debug!("Reading file {}...", path.display());
    match fs::read_to_string(path) {
        Ok(config_file) => {
            let document = serde_yaml::Deserializer::from_str(&config_file);
            match Value::deserialize(document) {
                Ok(parsed_file_content) => {
                    for route in parsed_file_content["routes"].as_sequence().unwrap() {
                        let route = route.as_mapping().unwrap().iter().next().unwrap();
                        routes.push(crate::Route::new(
                            route.0.as_str().unwrap().to_string(),
                            route.1.as_mapping().unwrap(),
                        ));
                    }
                }
                Err(e) => warn!("{}", e),
            }
        }
        Err(e) => warn!("{}", e),
    }
}
