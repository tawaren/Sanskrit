
extern crate sanskrit_common;
extern crate sanskrit_preloaded_validation;

extern crate core;

use std::{env, fs};
use std::path::PathBuf;
use std::time::Instant;
use sanskrit_common::errors::*;
use sanskrit_preloaded_validation::process_preloaded_module_deploy;

fn load_dep_from_file(mods:&mut Vec<Vec<u8>>, file:PathBuf) -> Result<()> {
    if file.is_file() {
        match file.extension() {
            Some(ext) if ext.eq("sans") => {
                match fs::read(file) {
                    Ok(content) => {
                        mods.push(content);
                        Ok(())
                    }
                    Err(_) => error(||"Could not read file")
                }
            }
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

fn load_deps_from_dir(mods:&mut Vec<Vec<u8>>, dir:PathBuf) -> Result<()> {
    match fs::read_dir(dir) {
        Ok(dir) => {
            for entry in dir {
                match entry {
                    Ok(e) => {
                        let path = e.path();
                        load_dep_from_file(mods, path)?;
                    }
                    Err(_) => error(||"Could not read directory content")?
                }
            }
            Ok(())
        }
        Err(_) => error(||"Could not read file")
    }
}

pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let work_dir = match env::current_dir(){
        Ok(w_dir) => w_dir,
        Err(_) => error(||"could not get working directory")?
    };

    let mut modules = Vec::new();
    let mut dependencies = Vec::new();
    if args.len() < 2 {
        return error(||"not enough arguments");
    }

    let t0 = Instant::now();
    let mut in_deps = false;
    for arg in &args[1..] {
        if arg.eq("-mp") {
            in_deps = true;
        } else {
            let active_map = if in_deps {
                &mut dependencies
            } else {
                &mut modules
            };

            let mod_path = work_dir.join(arg);
            if mod_path.is_dir() {
                load_deps_from_dir(active_map, mod_path)?;
            } else {
                load_dep_from_file(active_map, mod_path)?;
            }
        }
    }

    println!("loading dependencies took {}us", t0.elapsed().as_micros());

    let system_mode_on= true; //Todo: make configurable
    let t1 = Instant::now();
    match process_preloaded_module_deploy(modules, dependencies, system_mode_on) {
        Ok(h) => {
            println!("validating module took {}us", t1.elapsed().as_micros());
            println!("Validation succeeded for the {} modules", h.len());
            Ok(())
        }
        Err(_) => error(||"validation failed")
    }
}
