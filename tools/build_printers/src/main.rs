use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Error, Read};
use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;
#[macro_use]
extern crate log;
extern crate mowl;

mod cura;
mod gridapps;
pub mod orca;
mod p3d;
mod traits;

use cura::*;
use gridapps::*;
use orca::*;
use p3d::*;
use traits::*;

enum Source {
    Gridapps,
    Cura,
    CuraExtruder,
    Orca,
}

#[derive(Serialize, Debug)]
pub struct Printers {
    printers: BTreeMap<String, String>,
}

fn generalized(name: &str) -> String {
    name.replace('/', "")
        .replace('_', "")
        .replace(' ', "")
        .replace('(', "")
        .replace(')', ")")
        .replace('+', "plus")
        .replace('.', "")
        .to_lowercase()
}

fn main() -> Result<(), Error> {
    mowl::init().unwrap();

    let mut entries: HashMap<String, P3dPrinter> = HashMap::new();

    let lapr = load_and_parse()?;

    for (name, ga) in lapr.gridapps_entries {
        if let Some(p3d) = P3dPrinter::from_gridapps(name.clone(), ga) {
            let key = generalized(&p3d.name);
            entries.insert(key, p3d);
        } else {
            println!("Incomplete gridapps definition: {}", name);
        }
    }
    let mut collision = 0;
    for cura in lapr.cura_entries.values() {
        if let Some(p3d) = P3dPrinter::from_cura(&lapr.cura_entries, &lapr.cura_extruders, cura) {
            let p3d_name = p3d.name.clone();
            let key = generalized(&p3d_name);
            if let Some(ga) = entries.remove(&key) {
                collision += 1;
                if let Some(p3d_new) = P3dPrinter::resolve_conflict(ga, p3d) {
                    println!(
                        "{}: resolved collision {} into {}",
                        collision, key, p3d_new.name
                    );
                    entries.insert(key, p3d_new);
                } else {
                    println!("{}: UNRESOLVED collision {}", collision, key);
                }
            } else {
                entries.insert(key, p3d);
            }
        } else {
            println!("Incomplete cura definition: {}", cura.printer_name());
        }
    }
    for orca in lapr.orca_entries.values() {
        if let Some(p3d) = P3dPrinter::from_orca(orca) {
            let p3d_name = p3d.name.clone();
            let key = generalized(&p3d_name);
            if let Some(ga) = entries.remove(&key) {
                collision += 1;
                if let Some(p3d_new) = P3dPrinter::resolve_conflict(ga, p3d) {
                    println!(
                        "{}: resolved collision {} into {}",
                        collision, key, p3d_new.name
                    );
                    entries.insert(key, p3d_new);
                } else {
                    println!("{}: UNRESOLVED collision {}", collision, key);
                }
            } else {
                entries.insert(key, p3d);
            }
        } else {
            println!("Incomplete orca definition: {}", orca.printer_name());
        }
    }

    let mut printers = Printers {
        printers: BTreeMap::new(),
    };

    for (_name, e) in entries {
        let mut fname = env::current_dir()?;
        fname.pop();
        fname.pop();
        fname.push("printer");
        let printer_name = &e.name;
        let xname = printer_name
            .replace('/', "_")
            .replace(' ', "_")
            .replace('(', "_")
            .replace(')', ")")
            .replace('+', "_plus");
        fname.push(format!("{}.json", xname));
        //        println!("{:?}",fname);
        let file = File::create(fname)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &e)?;
        //        println!("{}: {:#?}", name, e);

        printers.printers.insert(e.name, xname);
    }

    // Create a printers.json file
    let mut fname = env::current_dir()?;
    fname.pop();
    fname.pop();
    fname.push("printer");
    fname.push("printer.json");
    let file = File::create(fname)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &printers)?;

    Ok(())
}

struct LoadAndParseResult {
    gridapps_entries: HashMap<String, Box<dyn GenericPrinter>>,
    cura_entries: HashMap<String, CuraV2>,
    cura_extruders: HashMap<String, Box<dyn GenericExtruder>>,
    orca_entries: HashMap<String, OrcaV1>,
}

fn load_and_parse() -> Result<LoadAndParseResult, Error> {
    let mut gridapps_entries: HashMap<String, Box<dyn GenericPrinter>> = HashMap::new();
    let mut cura_entries: HashMap<String, CuraV2> = HashMap::new();
    let mut cura_extruder_entries: HashMap<String, CuraExtruderV2> = HashMap::new();
    let mut orca_entries: HashMap<String, OrcaV1> = HashMap::new();

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
        (
            "https://github.com/p3d-dev/OrcaSlicer.git",
            "OrcaSlicer",
            "OrcaSlicer/resources/profiles",
            Source::Orca,
        ),
    ];

    // let temp_dir = env::temp_dir();
    let temp_dir: PathBuf = PathBuf::from("tmp");
    println!("Temporary directory: {}", temp_dir.display());

    for (repo, repo_name, dir, source) in repos {
        println!("Process {}", repo);
        let repo_dir = temp_dir.join(repo_name);
        if !repo_dir.exists() {
            let output = Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg(repo)
                .arg(repo_dir)
                .status()?;
            println!("{:?}", output);
        }
	else {
            let output = Command::new("git")
                .arg("-C")
		.arg(repo_dir)
                .arg("pull")
                .status()?;
            println!("{:?}", output);
	}

        let cfg_dir = temp_dir.join(dir);

        if let Source::CuraExtruder = source {
            // the file fdmextruder is at a weird place
            let fname = temp_dir.join("Cura/resources/definitions/fdmextruder.def.json");
            let mut file = File::open(fname)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            match serde_json::from_str::<CuraExtruderV2>(&contents) {
                Ok(cfg) => {
                    cura_extruder_entries.insert("fdmextruder".to_string(), cfg);
                }
                Err(e1) => {
                    println!("fdm_extruder: {:?}", e1)
                }
            }
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
                    (Source::Orca, Some("json")) => (),
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
                                let key = name.replace(".def.json", "");
                                cura_entries.insert(key, cfg);
                            }
                            Err(e1) => {
                                println!("cura {:?}: {:?}", entry.file_name(), e1)
                            }
                        },
                        Source::CuraExtruder => {
                            match serde_json::from_str::<CuraExtruderV2>(&contents) {
                                Ok(cfg) => {
                                    let key = name.replace(".def.json", "");
                                    cura_extruder_entries.insert(key, cfg);
                                }
                                Err(e1) => {
                                    println!("{:?}: {:?}", entry.file_name(), e1)
                                }
                            }
                        }
                        Source::Orca => match serde_json::from_str::<OrcaV1Header>(&contents) {
                            Ok(header) => {
                                let key = name.replace(".json", "");
                                let cfg_dir = temp_dir.join(dir).join(&key);
                                match header.read(cfg_dir) {
                                    Ok(printers) => {
                                        for orca in printers.into_iter() {
                                            println!("Orca {}", orca.printer_name());
                                            let name = orca.printer_name();
                                            orca_entries.insert(name, orca);
                                        }
                                    }
                                    Err(e1) => {
                                        println!("{:?}: {:?}", entry.file_name(), e1)
                                    }
                                }
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
    let mut cura_extruders: HashMap<String, Box<dyn GenericExtruder>> = HashMap::new();
    for (name, cfg) in cura_extruder_entries.iter() {
        if let Some(standard) = CuraExtruder::new(&cfg, &cura_extruder_entries) {
            let e = Box::new(standard);
            cura_extruders.insert(name.clone(), e);
        }
    }
    Ok(LoadAndParseResult {
        gridapps_entries,
        cura_entries,
        cura_extruders,
        orca_entries,
    })
}
