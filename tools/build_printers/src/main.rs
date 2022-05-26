use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Error, Read, BufWriter};
use std::process::Command;

mod cura;
mod gridapps;
mod p3d;

use cura::*;
use gridapps::*;
use p3d::*;

enum Source {
    Gridapps,
    Cura,
    CuraExtruder,
}

fn generalized(name: &str) -> String {
    name.replace("/","")
        .replace("_","")
        .replace(" ","")
        .replace("(","")
        .replace(")",")")
        .replace("+","plus")
        .replace(".","")
        .to_lowercase()
        .to_string()
}

fn main() -> Result<(), Error> {
    let mut entries: HashMap<String, P3dPrinter> = HashMap::new();

    let (gridapps_entries, cura_entries, cura_extruder_entries) = load_and_parse()?;

    for (name, ga) in gridapps_entries {
        if let Some(p3d) = P3dPrinter::from_gridapps(name.clone(), ga) {
            let key = generalized(&p3d.name);
            entries.insert(key, p3d);
        }
        else {
            println!("Incomplete gridapps definition: {}", name);
        }
    }
    let mut collision = 0;
    for cura in cura_entries.values() {
        if let Some(p3d) = P3dPrinter::from_cura(&cura_entries, &cura_extruder_entries, &cura) {
            let p3d_name = p3d.name.clone();
            let key = generalized(&p3d_name);
            if let Some(ga) = entries.remove(&key) {
                collision += 1;
                if let Some(p3d_new) = P3dPrinter::resolve_conflict(ga, p3d) {
                    println!("{}: resolved collision {} into {}", collision, key, p3d_new.name);
                    entries.insert(key, p3d_new);
                }
                else {
                    println!("{}: UNRESOLVED collision {}", collision, key);
                }
            }
            else {
                entries.insert(key, p3d);
            }
        }
        else {
            println!("Incomplete cura definition: {}", cura.printer_name());
        }
    }

    for (_name, e) in entries {
        let mut fname = env::current_dir()?;
        fname.pop();
        fname.pop();
        fname.push("printer");
        fname.push("x");
        let printer_name = &e.name;
        let xname = printer_name.replace("/","_").replace(" ","_").replace("(","_").replace(")",")").replace("+","_plus");
        fname.set_file_name(format!("{}.json",xname));
//        println!("{:?}",fname);
        let mut file = File::create(fname)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &e);
//        println!("{}: {:#?}", name, e);
    }

    Ok(())
}

fn load_and_parse() -> Result<(HashMap<String, Box<dyn GridApps>>, HashMap<String, CuraV2>, HashMap<String, CuraExtruderV2>), Error>
{
    let mut gridapps_entries: HashMap<String, Box<dyn GridApps>> = HashMap::new();
    let mut cura_entries: HashMap<String, CuraV2> = HashMap::new();
    let mut cura_extruder_entries: HashMap<String, CuraExtruderV2> = HashMap::new();

    let repos: Vec<(&'static str, &'static str, &'static str, Source)> = vec![
        (
            "https://github.com/GridSpace/grid-apps.git",
            "grid-apps",
            "grid-apps/src/kiri-dev/fdm",
            Source::Gridapps,
        ),
        (
            "https://github.com/p3d-dev/Cura.git",
            "Cura",
            "Cura/resources/definitions",
            Source::Cura,
        ),
        (
            "https://github.com/p3d-dev/Cura.git",
            "Cura",
            "Cura/resources/extruders",
            Source::CuraExtruder,
        ),
    ];

    let temp_dir = env::temp_dir();
    println!("Temporary directory: {}", temp_dir.display());

    for (repo, repo_name, dir, source) in repos {
        println!("Process {}", repo);
        let output = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(repo)
            .arg(temp_dir.join(repo_name))
            .status()?;
        println!("{:?}", output);

        let cfg_dir = temp_dir.join(dir);

        match source {
            Source::CuraExtruder => {
                // the file fdmextruder is at a weird place
                let fname = temp_dir.join("Cura/resources/definitions/fdmextruder.def.json");
                let mut file = File::open(fname)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                        match serde_json::from_str::<CuraExtruderV2>(&contents) {
                            Ok(cfg) => {
                                cura_extruder_entries.insert("fdmextruder".to_string(), cfg);
                            },
                            Err(e1) => {
                                println!("fdm_extruder: {:?}", e1)
                            }
                        }
            },
            _ => {}
        }

        for entry in std::fs::read_dir(cfg_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                println!("skip sub directory {:?}", entry.path());
            } else {
                match (&source, entry.path().extension().and_then(|x| x.to_str())) {
                    (Source::Gridapps, _) => (),
                    (Source::Cura, Some(_)) => (),
                    (Source::CuraExtruder, Some(_)) => (),
                    (_, _) => {
                        println!("skip file due to extension {:?}", entry.path().file_name());
                        continue;
                    }
                }

                //println!("process {:?}",entry.path());
                let mut file = File::open(entry.path())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                if let Some(name) = entry
                    .path()
                    .file_name()
                    .and_then(|x| x.to_str())
                    .map(|x| x.to_string())
                {
                    match source {
                        Source::Gridapps => match serde_json::from_str::<GridApps01>(&contents) {
                            Ok(cfg) => {
                                gridapps_entries.insert(name, Box::new(cfg));
                            }
                            Err(e1) => match serde_json::from_str::<GridApps02>(&contents) {
                                Ok(cfg) => {
                                    gridapps_entries.insert(name, Box::new(cfg));
                                }
                                Err(e2) => println!("{:?}: {:?} {:?}", entry.file_name(), e1, e2),
                            },
                        },
                        Source::Cura => match serde_json::from_str::<CuraV2>(&contents) {
                            Ok(cfg) => {
                                let key = name.replace(".def.json","");
                                cura_entries.insert(key, cfg);
                            }
                            Err(e1) => {
                                println!("{:?}: {:?}", entry.file_name(), e1)
                            }
                        },
                        Source::CuraExtruder => match serde_json::from_str::<CuraExtruderV2>(&contents) {
                            Ok(cfg) => {
                                let key = name.replace(".def.json","");
                                cura_extruder_entries.insert(key, cfg);
                            }
                            Err(e1) => {
                                println!("{:?}: {:?}", entry.file_name(), e1)
                            }
                        },
                    }
                }
            }
        }
    }
    Ok((gridapps_entries, cura_entries, cura_extruder_entries))
}
