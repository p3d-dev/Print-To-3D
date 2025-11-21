use std::cmp::*;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fs::File;
use std::io::{Error, Read};
use std::path::PathBuf;

use merge::Merge;
use regex::Regex;
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
    pub height: u16,
    pub center: bool,
    pub width: Option<u16>,
    pub depth: Option<u16>,
    pub diameter: Option<u16>,
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
            let x_int = x.round() as i16;
            let y_int = y.round() as i16;

            min_x = min(min_x, x_int);
            max_x = max(max_x, x_int);
            min_y = min(min_y, y_int);
            max_y = max(max_y, y_int);
            let r = (x * x + y * y).sqrt().round() as i16;
            max_r = max(max_r, r);
        }
        println!(
            "{}: x={}..{} y={}..{} r={}",
            self.name, min_x, max_x, min_y, max_y, max_r
        );
        let mut bv = BuildVolume::default();
        bv.center = (min_x + max_x == 0) && (min_y + max_y == 0);
        bv.round = bv.center && (max_r == max_x);
        bv.height = z.round() as u16;
        if bv.round {
            bv.diameter = Some(max_r as u16);
        } else {
            bv.width = Some((max_x - min_x) as u16);
            bv.depth = Some((max_y - min_y) as u16);
            if !bv.center && ((min_x != 0) || (min_y != 0)) {
                warn!("not centered");
                return None;
            }
        }
        Some(bv)
    }
    pub fn get_extruders(&self) -> Option<Vec<OrcaExtruder>> {
        let nozzles = self.nozzle_diameter.as_ref()?;
        if nozzles.len() != 1 {
            info!("{}: {:?} nozzles", self.name, nozzles);
        }
        let nd = nozzles.first()?.parse::<f32>().ok()?;
        if nd < 0.1 || nd > 1.0 {
            warn!("{}: {:?} nozzles", self.name, nozzles);
            return None;
        }
        let extruder = OrcaExtruder { diameter: nd };
        Some(vec![extruder])
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
                // println!("{}: {:?}", ns.name, machine);
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
                        warn!("{}: Cannot create orca printer", name);
                    }
                }
            }
        }
        Ok(ret)
    }
}

#[derive(Debug)]
pub struct OrcaExtruder {
    diameter: f32,
}
impl GenericExtruder for OrcaExtruder {
    fn get_nozzle(&self) -> f32 {
        self.diameter
    }
    fn get_filament(&self) -> f32 {
        1.75
    }
    fn get_offset_x(&self) -> f32 {
        0.0
    }
    fn get_offset_y(&self) -> f32 {
        0.0
    }
    fn get_select_gcode(&self) -> String {
        "".to_string()
    }
    fn get_deselect_gcode(&self) -> String {
        "".to_string()
    }
}

#[derive(Debug)]
pub struct OrcaV1 {
    pub name: String,
    pub machine: OrcaFdmMachine,
    pub build_volume: BuildVolume,
    pub extruders: Vec<OrcaExtruder>,
}
impl OrcaV1 {
    pub fn init(name: String, machine: OrcaFdmMachine) -> Option<Self> {
        let bv = machine.get_build_volume()?;
        let extruders = machine.get_extruders()?;
        Some(OrcaV1 {
            name,
            machine,
            build_volume: bv,
            extruders,
        })
    }
    pub fn printer_name(&self) -> String {
        self.name.clone()
    }
    pub fn process_variables(&self, line: &str) -> Option<String> {
        // find each {..} in line
        // Find all matches in the text
        let mut replacement: HashMap<&str, &str> = HashMap::new();
        replacement.insert("[printer_model]", &self.name);
        replacement.insert("{first_layer_bed_temperature[0]}", "{bed_temp}");
        replacement.insert("[first_layer_bed_temperature]", "{bed_temp}");
        replacement.insert("[bed_temperature_initial_layer_single]", "{bed_temp}");
        replacement.insert("[bed_temperature_initial_layer]", "{bed_temp}");
        replacement.insert("[hot_plate_temp_initial_layer]", "{bed_temp}");
        replacement.insert("[first_layer_temperature]", "{temp}");
        replacement.insert("{first_layer_temperature[0]}", "{temp}");
        replacement.insert("[first_layer_temperature[initial_tool]]", "{temp}");
        replacement.insert("[nozzle_temperature_initial_layer]", "{temp}");
        replacement.insert(
            "{nozzle_temperature_initial_layer[initial_extruder]}",
            "{temp}",
        );
        replacement.insert(
            "{nozzle_temperature[initial_no_support_extruder]}",
            "{temp}",
        );
        replacement.insert("[layer_num]", "{layer}");
        replacement.insert("[layer_z]", "{z}");
        replacement.insert("[total_layer_count]", "{layers}");
        replacement.insert("{+0.0}", "{+0.0}");
        replacement.insert("{0.0}", "{0.0}");
        // for Anycubic Kobra S1
        replacement.insert("{if max_layer_z < max_print_height-1}G1 Z{z_offset+min(max_layer_z+2, max_print_height)} F900 ; Move print head further up{endif}", "G1 Z{z+2}");
        // Raise3D Pro3 Plus 0.4 nozzle (Right): G1 X170 Y{random(2,8)} E15 F200 : {random(2,8)}
        replacement.insert("{random(2,8)}", "{Math.floor(Math.random() * 6 + 2)}");
        // Others
        replacement.insert(
            "{random(-160,-152)}",
            "{Math.floor(Math.random() * 8 - 160)}",
        );
        // for Elegoo Centauri Carbon: 256*256*256mm
        replacement.insert("{print_bed_max[0]*0.5}", "128");
        replacement.insert("{print_bed_max[0]*0.5-10}", "118");
        replacement.insert("{print_bed_max[0]*0.5-30}", "98");
        replacement.insert("{print_bed_max[0]*0.5-50}", "78");
        replacement.insert("{print_bed_max[0]*0.5+10}", "138");
        replacement.insert("{print_bed_max[0]*0.5+30}", "158");
        replacement.insert("{print_bed_max[0]*0.5+50}", "178");
        // use filament_max_volumetric_speed[initial_no_support_extruder] = 12 mm^3/s (not ok for TPU)
        replacement.insert("{0.2*min(12000, max(1200, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))}", "960");
        replacement.insert("{min(12000, max(1200, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))}", "4800");
        replacement.insert("{min(6000, max(900, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))}", "4800");
        replacement.insert("{min(20000,max(1000,outer_wall_acceleration))}", "6000");
        replacement.insert("{max_layer_z+0.5}", "{z+0.5}");
        replacement.insert("{layer_num + 1}", "{layer + 1}");

        let mut newline = line.to_string();
        for (k, v) in replacement.iter() {
            newline = newline.replace(k, v);
        }

        let mut known: HashSet<&str> = HashSet::new();
        for v in replacement.values() {
            known.insert(v);
        }
        known.insert("{z}");
        known.insert("{layer}");
        known.insert("{temp}");
        known.insert("{bed_temp}");
        known.insert("{z_max+2}");
        known.insert("{z+2}");
        known.insert("{Math.min(z+50, z_max+0.5)}");
        known.insert("{layer + 1}");
        //known.insert("[printer_model]");
        //known.insert("{filament_type[initial_no_support_extruder]}"); // Bambu
        //known.insert("[initial_no_support_extruder]"); // Bambu
        //known.insert("{nozzle_temperature_initial_layer[initial_extruder]}"); // Bambu
        //known.insert("{first_layer_bed_temperature[0]}");
        //known.insert("[bed_temperature_initial_layer]");
        //known.insert("[first_layer_temperature]");
        //known.insert("[first_layer_bed_temperature]");
        //known.insert("[nozzle_temperature_initial_layer]");
        //known.insert("[bed_temperature_initial_layer_single]");
        //known.insert("{+0.0}");
        //known.insert("{0.0}");
        known.insert("{total_layer_count}");

        // Create a regular expression to match "{..}"
        let re = Regex::new(r"\{.*?\}").unwrap();
        for mat in re.captures_iter(&newline) {
            let m = mat.get(0)?.as_str();
            if known.contains(m) {
                continue;
            }
            warn!("{}: {} : {}", self.name, newline, m);
            warn!("Found '{{..}}': {:?}", mat.get(0)?.as_str());
            return None;
        }
        // Create a regular expression to match "[..]"
        let re = Regex::new(r"\[.*?\]").unwrap();
        for mat in re.captures_iter(&newline) {
            let m = mat.get(0)?.as_str();
            if known.contains(m) {
                continue;
            }
            warn!("{}: {} : {}", self.name, newline, m);
            warn!("Found '[..]': {:?}", mat.get(0)?.as_str());
            return None;
        }
        Some(newline)
    }
    pub fn process_gcode(&self, gcode: &str) -> Option<Vec<String>> {
        let mut err = false;
        let mut lines = vec![];
        for line in gcode.split("\n") {
            if line.trim().starts_with(";") {
                if let Some(line) = self.process_variables(line) {
                    lines.push(line);
                } else {
                    lines.push(line.to_string());
                }
            } else {
                if let Some(line) = self.process_variables(line) {
                    lines.push(line);
                } else {
                    err = true;
                }
            }
        }
        if err {
            return None;
        }
        Some(lines)
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
        Some(self.build_volume.center)
    }
    fn get_bed_circle(&self) -> Option<bool> {
        Some(self.build_volume.round)
    }
    fn get_bed_width(&self) -> Option<u16> {
        if self.build_volume.round {
            self.build_volume.diameter
        } else {
            self.build_volume.width
        }
    }
    fn get_bed_depth(&self) -> Option<u16> {
        if self.build_volume.round {
            self.build_volume.diameter
        } else {
            self.build_volume.depth
        }
    }
    fn get_build_height(&self) -> Option<u16> {
        Some(self.build_volume.height)
    }
    fn get_pre_gcode(&self) -> Option<Vec<String>> {
        let gcode = self.machine.machine_start_gcode.as_ref()?;
        self.process_gcode(gcode)
    }
    fn get_post_gcode(&self) -> Option<Vec<String>> {
        let gcode = self.machine.machine_end_gcode.as_ref()?;
        self.process_gcode(gcode)
    }
    fn get_fan_gcode(&self) -> Option<String> {
        Some("M106 S{fan_speed}  ; p3d: add for orca slicer".to_string())
    }
    fn get_progress_gcode(&self) -> Option<String> {
        Some("".to_string())
    }
    fn get_layer_gcode(&self) -> Option<String> {
        if let Some(gcode) = self.machine.layer_change_gcode.as_ref() {
            return self.process_gcode(gcode).map(|s| s.join("\n"));
        }
        Some("".to_string())
    }
    fn get_bed_belt(&self) -> Option<bool> {
        Some(false)
    }
    fn get_extruders(&self) -> Option<Vec<&dyn GenericExtruder>> {
        Some(
            self.extruders
                .iter()
                .map(|e| e as &dyn GenericExtruder)
                .collect(),
        )
    }
    fn get_speed_travel(&self) -> Option<f32> {
        None
    }
}
