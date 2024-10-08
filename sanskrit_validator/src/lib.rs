
use std::{env, fs};
use std::path::{Path, PathBuf};
use std::time::Instant;
use sanskrit_common::errors::*;

pub trait PathExtensions {
    fn extensions(&self, count: usize) -> Option<String>;
}

impl PathExtensions for Path {
    fn extensions(&self, count: usize) -> Option<String> {
        let file_name = self.file_name()?.to_str()?;
        let split = file_name.split('.').collect::<Vec<&str>>();
        if split.len() <= count {
            None
        } else {
            let start = split.len()-count;
            Some(split[start..].join("."))
        }
    }
}


fn load_dep_from_file(cols:&mut [(&str, Vec<Vec<u8>>)], exts:usize, file:PathBuf) -> Result<()> {
    if file.is_file() {
        match file.extensions(exts) {
            Some(ext) => {
                for col in cols {
                    if ext.eq(col.0) {
                        return match fs::read(file) {
                            Ok(content) => {
                                col.1.push(content);
                                Ok(())
                            }
                            Err(_) => error(|| "Could not read file")
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

fn load_deps_from_dir(cols:&mut [(&str, Vec<Vec<u8>>)], exts:usize, dir:PathBuf) -> Result<()> {
    match fs::read_dir(dir) {
        Ok(dir) => {
            for entry in dir {
                match entry {
                    Ok(e) => {
                        let path = e.path();
                        load_dep_from_file(cols, exts, path)?;
                    }
                    Err(_) => error(||"Could not read directory content")?
                }
            }
            Ok(())
        }
        Err(_) => error(||"Could not read file")
    }
}

//Todo: use clap for args parsing
pub fn execute_with_args<T,F:FnOnce(Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<Vec<u8>>, bool) -> Result<T>>(args:&[String], f:F) -> Result<T>{
    let work_dir = match env::current_dir(){
        Ok(w_dir) => w_dir,
        Err(_) => error(||"could not get working directory")?
    };

    if args.len() < 1 {
        return error(||"not enough arguments");
    }

    //Collection configuration (inkl. collectors)
    let exts = 2;
    let mut mod_and_txts = vec![("mod.sans", Vec::new()), ("txt.sans", Vec::new())];
    let mut deps =vec![("mod.sans", Vec::new())];

    let t0 = Instant::now();
    let mut active_cols = &mut mod_and_txts[..];
    let mut system_mode = false;
    for arg in args {
        if arg.eq("-s") || arg.eq("-system") {
            system_mode = true;
        } else if arg.eq("-mp") {
            active_cols = &mut deps[..];
        } else {
            let mod_path = work_dir.join(arg);
            if mod_path.is_dir() {
                load_deps_from_dir(active_cols, exts, mod_path)?;
            } else {
                load_dep_from_file(active_cols, exts, mod_path)?;
            }
        }
    }

    //Todo: have a flag to filter unneeded ones
    //      1. Parse modules & transactions
    //      2. go over the definition and extract module hashes from public imports
    //      3. put them in a map
    //      4. filter dependencies with a store hash not in map
    //         (remove them from the map in the process) -- deduplication
    //      5. Print stats (filtered, missing - remaining map size)

    println!("loading input took {}us", t0.elapsed().as_micros());
    let system_mode_on= true; //Todo: make configurable
    let transactions = mod_and_txts.pop().unwrap().1;
    if system_mode {
        println!("system mode is enabled");
    }
    println!("loaded {} transactions", transactions.len());
    let modules = mod_and_txts.pop().unwrap().1;
    println!("loaded {} modules", modules.len());
    let dependencies = deps.pop().unwrap().1;
    println!("loaded {} dependencies", dependencies.len());
    let res = f(modules, transactions, dependencies, system_mode_on);
    return res;

}
