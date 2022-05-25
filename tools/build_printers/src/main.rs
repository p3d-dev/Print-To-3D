use std::env;
use std::io::{Error, Read};
use std::process::Command;
use std::fs::{File,DirEntry};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize,Debug)]
struct Extruder01 {
    nozzle: f32,
    filament: f32,
    offset_x: i16,
    offset_y: i16,
    select: Vec<String>,
}

#[derive(Deserialize,Debug)]
struct Command01 {
    fan_power: String,
    progress: Option<String>,
    layer: Option<Vec<String>>,
}

#[derive(Deserialize,Debug)]
struct Settings01 {
    nozzle_size: Option<f32>,
    origin_center: bool,
    bed_width: u16,
    bed_depth: u16,
    build_height: u16,
    bed_circle: Option<bool>,
}

#[derive(Deserialize,Debug)]
struct GridApps01 {
    pre: Vec<String>,
    post: Vec<String>,
    cmd: Command01,
    extruders: Option<Vec<Extruder01>>,
    settings: Settings01
}

#[derive(Deserialize,Debug)]
struct Extruder02 {
    extFilament: f32,
    extNozzle: f32,
    extSelect: Vec<String>,
    extDeselect: Vec<String>,
    extOffsetX: i16,
    extOffsetY: i16,
}

#[derive(Deserialize,Debug)]
struct Settings02 {
    need: bool
}

#[derive(Deserialize,Debug)]
struct GridApps02 {
    deviceName: String,
    bedWidth: i16,
    bedDepth: i16,
    maxHeight: i16,
    bedRound: bool,
    bedBelt: bool,
    deviceZMax: Option<i16>,
    originCenter: bool,
    extrudeAbs: Option<bool>,
    gcodePre: Vec<String>,
    gcodePost: Vec<String>,
    gcodeLayer: Vec<String>,
    gcodeTrack: Vec<String>,
    gcodeFan: Vec<String>,
    extruders: Vec<Extruder02>,
    settings: Option<Settings02>
}


fn main() -> Result<(), Error> {
    let repos: Vec<(&'static str, &'static str, &'static str)> = vec![(
        "https://github.com/GridSpace/grid-apps.git",
        "grid-apps",
        "grid-apps/src/kiri-dev/fdm",
    )];

    let temp_dir = env::temp_dir();
    println!("Temporary directory: {}", temp_dir.display());

    for (repo, repo_name, dir) in repos {
        println!("Process {}", repo);
        let output = Command::new("git")
                     .arg("clone")
                     .arg(repo)
                     .arg(temp_dir.join(repo_name))
                     .status()?;
        println!("{:?}", output);

        let cfg_dir = temp_dir.join(dir);
        
        for entry in std::fs::read_dir(cfg_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                println!("skip sub directory {:?}",entry.path());
            }
            else {
                //println!("process {:?}",entry.path());
                let mut file = File::open(entry.path())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                match serde_json::from_str::<GridApps01>(&contents) {
                    Ok(cfg) => (),
                    Err(e1) => {
                match serde_json::from_str::<GridApps02>(&contents) {
                    Ok(cfg) => (),
                    Err(e2) => println!("{:?}: {:?}i {:?}", entry.path(), e1,e2)
                }
                    }
                }
            }
       }
    }
    Ok(())
}
