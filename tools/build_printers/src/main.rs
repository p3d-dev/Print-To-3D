use std::env;
use std::fs::{DirEntry, File};
use std::io::{Error, Read};
use std::collections::HashMap;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

trait GridApps {
}

#[derive(Deserialize, Debug)]
struct Extruder01 {
    nozzle: f32,
    filament: f32,
    offset_x: i16,
    offset_y: i16,
    select: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Command01 {
    fan_power: String,
    progress: Option<String>,
    layer: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Settings01 {
    nozzle_size: Option<f32>,
    origin_center: bool,
    bed_width: u16,
    bed_depth: u16,
    build_height: u16,
    bed_circle: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct GridApps01 {
    pre: Vec<String>,
    post: Vec<String>,
    cmd: Command01,
    extruders: Option<Vec<Extruder01>>,
    settings: Settings01,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Extruder02 {
    extFilament: f32,
    extNozzle: f32,
    extSelect: Vec<String>,
    extDeselect: Vec<String>,
    extOffsetX: i16,
    extOffsetY: i16,
}

#[derive(Deserialize, Debug)]
struct Settings02 {
    need: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
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
    settings: Option<Settings02>,
}

#[derive(Deserialize, Debug)]
struct Default<T> {
    default_value: Option<T>,
    value: Option<T>,
}

#[derive(Deserialize, Debug)]
struct OverridesV2 {
    machine_name: Option<Default<String>>,
    machine_heated_bed: Option<Default<bool>>,
    machine_nozzle_size: Option<Default<f32>>,
    machine_width: Option<Default<f32>>,
    machine_height: Option<Default<f32>>,
    machine_depth: Option<Default<f32>>,
    machine_center_is_zero: Option<Default<bool>>,
    machine_start_gcode: Option<Default<String>>,
    machine_end_gcode: Option<Default<String>>,
    machine_shape: Option<Default<String>>,
    material_diameter: Option<Default<f32>>,
}

#[derive(Deserialize, Debug)]
struct LongSetting<T> {
    default_value: Option<T>,
}

#[derive(Deserialize, Debug)]
struct DetailedSettingsV2 {
    machine_name: Option<LongSetting<String>>,
    machine_heated_bed: Option<LongSetting<bool>>,
    machine_nozzle_size: Option<LongSetting<f32>>,
    machine_width: Option<LongSetting<f32>>,
    machine_height: Option<LongSetting<f32>>,
    machine_depth: Option<LongSetting<f32>>,
    machine_center_is_zero: Option<LongSetting<bool>>,
    machine_start_gcode: Option<LongSetting<String>>,
    machine_end_gcode: Option<LongSetting<String>>,
    machine_shape: Option<LongSetting<String>>,
    material_diameter: Option<LongSetting<f32>>,
}

#[derive(Deserialize, Debug)]
struct MachineSettingsV2 {
    children: DetailedSettingsV2,
}

#[derive(Deserialize, Debug)]
struct SettingsV2 {
    machine_settings: MachineSettingsV2,
}

#[derive(Deserialize, Debug)]
struct CuraV2 {
    name: String,
    inherits: Option<String>,
    overrides: Option<OverridesV2>,
    settings: Option<SettingsV2>,
}

enum Source {
    Gridapps,
    Cura,
}

fn main() -> Result<(), Error> {
    loadAndParse()
}

fn loadAndParse() -> Result<(), Error> {
    let mut gridapps_entries: HashMap<String,Box<dyn GridApps>> = HashMap::new(); 
    let mut cura_entries: HashMap<String,CuraV2> = HashMap::new(); 

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
                    (Source::Gridapps,Some(_)) => (),
                    (Source::Cura,Some(_)) => (),
                    (_,_) => {
                        println!("skip file due to extension {:?}", entry.path().file_name());
                        continue
                    }
                }

                //println!("process {:?}",entry.path());
                let mut file = File::open(entry.path())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                match source {
                    Source::Gridapps => match serde_json::from_str::<GridApps01>(&contents) {
                        Ok(cfg) => (),
                        Err(e1) => match serde_json::from_str::<GridApps02>(&contents) {
                            Ok(cfg) => (),
                            Err(e2) => println!("{:?}: {:?} {:?}", entry.file_name(), e1, e2),
                        },
                    },
                    Source::Cura => match serde_json::from_str::<CuraV2>(&contents) {
                        Ok(cfg) => (),
                        Err(e1) => {
                            println!("{:?}: {:?}", entry.file_name(), e1)
                        }
                    },
                }
            }
        }
    }
    Ok(())
}
