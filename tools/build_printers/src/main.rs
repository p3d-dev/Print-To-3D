use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Error, Read};
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
}

fn main() -> Result<(), Error> {
    let mut entries: HashMap<String, P3dPrinter> = HashMap::new();

    let (gridapps_entries, cura_entries) = load_and_parse()?;

    for (name, ga) in gridapps_entries {
        let p3d = P3dPrinter::from_gridapps(name, ga);
        entries.insert(p3d.name.clone(), p3d);
    }
    for cura in cura_entries.values() {
        if let Some(p3d) = P3dPrinter::from_cura(&cura_entries, &cura) {
            entries.insert(p3d.name.clone(), p3d);
        }
        else {
            println!("Incomplete definition: {}", cura.printer_name());
        }
    }

    for (name, e) in entries {
//        println!("{}: {:#?}", name, e);
    }

    Ok(())
}

fn load_and_parse() -> Result<(HashMap<String, Box<dyn GridApps>>, HashMap<String, CuraV2>), Error>
{
    let mut gridapps_entries: HashMap<String, Box<dyn GridApps>> = HashMap::new();
    let mut cura_entries: HashMap<String, CuraV2> = HashMap::new();

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

        for entry in std::fs::read_dir(cfg_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                println!("skip sub directory {:?}", entry.path());
            } else {
                match (&source, entry.path().extension().and_then(|x| x.to_str())) {
                    (Source::Gridapps, Some(_)) => (),
                    (Source::Cura, Some(_)) => (),
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
                    }
                }
            }
        }
    }
    Ok((gridapps_entries, cura_entries))
}
