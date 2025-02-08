use std::cmp::*;
use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::io::{Error, Read};
use std::path::PathBuf;

use merge::Merge;
use serde::Deserialize;

use crate::traits::*;

#[derive(Deserialize, Debug)]
pub struct OrcaNameSubpath {
    name: String,
    sub_path: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct OrcaV1Header {
    name: String,
    machine_model_list: Option<Vec<OrcaNameSubpath>>,
    process_list: Vec<OrcaNameSubpath>,
    filament_list: Vec<OrcaNameSubpath>,
    machine_list: Vec<OrcaNameSubpath>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct OrcaMachine {
    #[serde(rename = "type")]
    machine_type: String,
    name: String,
    nozzle_diameter: String,
    family: String,
    machine_tech: String,
    model_id: String,
    default_materials: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct OrcaNozzle {
    #[serde(rename = "type")]
    machine_type: String,
    name: String,
    nozzle_diameter: Vec<String>,
    printer_model: String,
    max_layer_height: Option<Vec<String>>,
    min_layer_height: Option<Vec<String>>,
    default_print_profile: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Merge, Default)]
pub struct OrcaFdmMachine {
    #[serde(rename = "type")]
    #[merge(skip)]
    machine_type: String,
    #[merge(skip)]
    name: String,
    #[merge(skip)]
    instantiation: String, // "true" if a real printer
    printer_model: Option<String>,
    default_print_profile: Option<String>,
    inherits: Option<String>,
    pub printable_area: Option<Vec<String>>,
    bed_exclude_area: Option<Vec<String>>,
    pub printable_height: Option<String>,
    nozzle_diameter: Option<Vec<String>>,
    nozzle_type: Option<String>,
    support_chamber_temp_control: Option<String>,
    enable_long_retraction_when_cut: Option<String>,
    gcode_flavor: Option<String>,
    machine_start_gcode: Option<String>,
    machine_end_gcode: Option<String>,
    change_filament_gcode: Option<String>,
    before_layer_change_gcode: Option<String>,
    layer_change_gcode: Option<String>,
    machine_pause_gcode: Option<String>,

    machine_max_acceleration_extruding: Option<Vec<String>>,
    machine_max_acceleration_retracting: Option<Vec<String>>,
    machine_max_acceleration_travel: Option<Vec<String>>,
    machine_max_acceleration_x: Option<Vec<String>>,
    machine_max_acceleration_y: Option<Vec<String>>,
    machine_max_acceleration_z: Option<Vec<String>>,
    machine_max_speed_e: Option<Vec<String>>,
    machine_max_speed_x: Option<Vec<String>>,
    machine_max_speed_y: Option<Vec<String>>,
    machine_max_speed_z: Option<Vec<String>>,
    machine_max_jerk_e: Option<Vec<String>>,
    machine_max_jerk_x: Option<Vec<String>>,
    machine_max_jerk_y: Option<Vec<String>>,
    machine_max_jerk_z: Option<Vec<String>>,
    max_layer_height: Option<Vec<String>>,
    min_layer_height: Option<Vec<String>>,
    retraction_minimum_travel: Option<Vec<String>>,
    retract_before_wipe: Option<Vec<String>>,
    retract_length_toolchange: Option<Vec<String>>,
    retraction_speed: Option<Vec<String>>,
    deretraction_speed: Option<Vec<String>>,
    single_extruder_multi_material: Option<String>,
    default_filament_profile: Option<Vec<String>>,
}
#[derive(Default, Debug)]
pub struct BuildVolume {
    pub round: bool,
    pub height: u32,
    pub center: bool,
    pub width: Option<u32>,
    pub depth: Option<u32>,
    pub diameter: Option<u32>,
}
impl OrcaFdmMachine {
    pub fn get_build_volume(&self) -> Option<BuildVolume> {
        let z = self.printable_height.as_ref()?.parse::<f32>().ok()?;
        let pa = self.printable_area.as_ref()?;
        let mut max_r = 0;
        let mut min_x = 0;
        let mut max_x = 0;
        let mut min_y = 0;
        let mut max_y = 0;
        for s in pa {
            let mut v = s.split("x");
            let x = v.next()?.parse::<f32>().ok()?;
            let y = v.next()?.parse::<f32>().ok()?;
            let x_int = x.round() as i32;
            let y_int = y.round() as i32;

            min_x = min(min_x, x_int);
            max_x = max(max_x, x_int);
            min_y = min(min_y, y_int);
            max_y = max(max_y, y_int);
            let r = (x * x + y * y).sqrt().round() as i32;
            max_r = max(max_r, r);
        }
        println!("x={}..{} y={}..{} r={}", min_x, max_x, min_y, max_y, max_r);
        let mut bv = BuildVolume::default();
        bv.center = (min_x + max_x == 0) && (min_y + max_y == 0);
        bv.round = bv.center && (max_r == max_x);
        bv.height = z.round() as u32;
        if bv.round {
            bv.diameter = Some(max_r as u32);
        } else {
            bv.width = Some((max_x - min_x) as u32);
            bv.depth = Some((max_y - min_y) as u32);
            if !bv.center && ((min_x != 0) || (min_y != 0)) {
                return None;
            }
        }
        Some(bv)
    }
}

impl OrcaV1Header {
    // OrcaSlicer is structured in dirs
    // There is top level json-file, which directs which file in subpath are what.
    // Inheritance is only within one subdirectory.
    // During read all inherits can be resolved
    pub fn read(&self, cfg_dir: PathBuf) -> Result<Vec<OrcaV1>, Error> {
        if false {
            for ns in self.machine_model_list.as_ref().unwrap() {
                let mut path = cfg_dir.clone();
                path.push(ns.sub_path.clone());
                let mut file = File::open(path.clone())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let machine = serde_json::from_str::<OrcaMachine>(&contents)?;
                println!("{}: {:?}", ns.name, machine);
                // split nozzle_diameter on ";"
                let nd = machine.nozzle_diameter.split(";");
                path.set_extension("");
                let base = path.file_name().unwrap().to_str().unwrap();
                let machine_dir = path.parent().unwrap().to_path_buf();
                let mut path = machine_dir.clone();
                path.push("fdm_machine_common.json");
                if path.exists() {
                    let mut file = File::open(path.clone())?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    let fdm = serde_json::from_str::<OrcaFdmMachine>(&contents)?;
                    println!("{:?}: {:?}", path.file_name().unwrap(), fdm);
                } else {
                    warn!("No fdm_machine_common.json found for {:?}", path);
                }

                for n in nd {
                    let fname = format!("{} {} nozzle.json", base, n);
                    let mut path = machine_dir.clone();
                    path.push(fname);
                    if path.exists() {
                        let mut file = File::open(path.clone())?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;
                        let nozzle = serde_json::from_str::<OrcaNozzle>(&contents)?;
                        println!("{}: {:?}", ns.name, nozzle);
                    } else {
                        println!("No nozzle found for {}: {:?}", ns.name, n);
                    }
                }
            }
        }
        let mut name2machine = HashMap::new();
        for ns in self.machine_list.iter() {
            let mut path = cfg_dir.clone();
            path.push(ns.sub_path.clone());
            let mut file = File::open(path.clone())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            match serde_json::from_str::<OrcaFdmMachine>(&contents) {
                Ok(machine) => {
                    println!("{}: {:?}", ns.name, machine);
                    name2machine.insert(ns.name.clone(), machine);
                }
                Err(e) => {
                    warn!("{}: {:?}", ns.name, e);
                }
            }
        }
        for m in name2machine.keys() {
            println!("{}", m);
        }
        loop {
            info!("LOOP");
            let mut need_inherit = false;
            let keys: Vec<String> = name2machine.keys().map(|k| k.clone()).collect();
            for k in keys.into_iter() {
                let mut machine: OrcaFdmMachine = name2machine.remove(&k).unwrap();
                let mut master_machine: Option<OrcaFdmMachine> = None;
                if let Some(master) = machine.inherits.as_ref() {
                    if let Some(mm) = name2machine.get(master) {
                        master_machine = Some(mm.clone());
                    } else {
                        warn!(
                            "{} inherits from {}: not found",
                            machine.name,
                            machine.inherits.as_ref().unwrap()
                        );
                    }
                }
                if let Some(master_machine) = master_machine {
                    machine.merge(master_machine.clone());
                    println!(
                        "{} inherits from {}",
                        machine.name,
                        machine.inherits.as_ref().unwrap()
                    );
                    machine.inherits = master_machine.inherits.as_ref().cloned();
                    if machine.inherits.is_some() {
                        need_inherit = true;
                    }
                }
                name2machine.insert(k, machine);
            }
            if !need_inherit {
                break;
            }
        }
        let mut ret = Vec::new();
        for (_, fdm) in name2machine.into_iter() {
            if fdm.instantiation == "true" {
                let name = fdm.name.clone();
                match OrcaV1::init(name.clone(), fdm) {
                    Some(v1) => ret.push(v1),
                    None => {
                        warn!("{}: no build volume", name);
                    }
                }
            }
        }
        Ok(ret)
    }
}

#[derive(Debug)]
pub struct OrcaV1 {
    pub name: String,
    pub machine: OrcaFdmMachine,
    pub build_volume: BuildVolume,
}
impl OrcaV1 {
    pub fn init(name: String, machine: OrcaFdmMachine) -> Option<Self> {
        let bv = machine.get_build_volume()?;
        Some(OrcaV1 {
            name,
            machine,
            build_volume: bv,
        })
    }
    pub fn printer_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_build_size(&self) -> Option<(f32, f32, f32, bool)> {
        if let Some(exclude) = self.machine.bed_exclude_area.as_ref() {
            if exclude.len() != 1 {
                return None;
            }
            if exclude[0] != "0x0" {
                return None;
            }
        }
        None
    }
}
impl GenericPrinter for OrcaV1 {
    fn get_license(&self) -> String {
        "AGPL-3.0".to_string()
    }
    fn get_source(&self) -> String {
        "OrcaSlicerV1".to_string()
    }
    fn get_origin_center(&self) -> Option<bool> {
        None
    }
    fn get_bed_circle(&self) -> Option<bool> {
        None
    }
    fn get_bed_width(&self) -> Option<u16> {
        None
    }
    fn get_bed_depth(&self) -> Option<u16> {
        None
    }
    fn get_build_height(&self) -> Option<u16> {
        None
    }
    fn get_pre_gcode(&self) -> Option<Vec<String>> {
        None
    }
    fn get_post_gcode(&self) -> Option<Vec<String>> {
        None
    }
    fn get_fan_gcode(&self) -> Option<String> {
        None
    }
    fn get_progress_gcode(&self) -> Option<String> {
        None
    }
    fn get_layer_gcode(&self) -> Option<String> {
        None
    }
    fn get_bed_belt(&self) -> Option<bool> {
        Some(false)
    }
    fn get_extruders(&self) -> Option<Vec<&dyn GenericExtruder>> {
        None
    }
}
