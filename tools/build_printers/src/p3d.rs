use std::collections::HashMap;

use lazy_static::*;
use serde::Serialize;
use regex::Regex;

use crate::cura::*;
use crate::gridapps::*;

#[derive(Serialize, Debug)]
pub struct P3dExtruder {
    nozzle: f32,
    filament: f32,
    offset_x: f32,
    offset_y: f32,
    select: String,
    deselect: String,
}
impl P3dExtruder {
    fn standard() -> Self {
        P3dExtruder {
            nozzle: 0.4,
            filament: 1.75,
            offset_x: 0.0,
            offset_y: 0.0,
            select: "".to_string(),
            deselect: "".to_string(),
        }
    }
    pub fn from_gridapps(cfg: &dyn GridAppsExtruder) -> Option<Self> {
        let nozzle = cfg.get_nozzle();
        let filament = cfg.get_filament();
        let offset_x = cfg.get_offset_x();
        let offset_y = cfg.get_offset_y();
        let select = cfg.get_select_gcode();
        let deselect = cfg.get_deselect_gcode();
        Some(P3dExtruder {
            nozzle,
            filament,
            offset_x,
            offset_y,
            select,
            deselect,
        })
    }
    pub fn from_cura(all: &HashMap<String, CuraExtruderV2>, cfg_name: &String) -> Option<Self> {
        let cfg = all.get(cfg_name)?;
        let nozzle = cfg.get_nozzle(all)?;
        let filament = cfg.get_filament(all)?;
        let offset_x = cfg.get_offset_x(all)?;
        let offset_y = cfg.get_offset_y(all)?;
        let select = cfg.get_select_gcode(all)?;
        let deselect = cfg.get_deselect_gcode(all)?;

        if nozzle.abs() < 0.001 {
            println!("nozzle: {} {:?} nozzle is zero", cfg_name, cfg);
            return None;
        }
        if filament.abs() < 0.001 {
            println!("nozzle: {} {:?} filament is zero", cfg_name, cfg);
            return None;
        }

        //        println!("nozzle: {} {:?}", cfg_name, cfg);
        Some(P3dExtruder {
            nozzle,
            filament,
            offset_x,
            offset_y,
            select,
            deselect,
        })
    }
}

#[derive(Serialize, Debug)]
pub struct P3dPrinter {
    pub name: String,
    machine_name: String,
    source: String,
    origin_center: bool,
    bed_elliptic: bool,
    bed_belt: bool,
    extruders: Vec<P3dExtruder>,
    build_size: (u16, u16, u16),
    pre_gcode: Vec<String>,
    post_gcode: Vec<String>,
    fan_gcode: String,
    progress_gcode: String,
    layer_gcode: String,
    heated_bed: bool,
}

impl P3dPrinter {
    pub fn from_gridapps(name: String, cfg: Box<dyn GridApps>) -> Option<Self> {
        let source = cfg.get_source();
        let origin_center = cfg.get_origin_center();
        let bed_elliptic = cfg.get_bed_circle();
        let bed_width = cfg.get_bed_width();
        let bed_depth = cfg.get_bed_depth();
        let build_height = cfg.get_build_height();
        let build_size = (bed_width, bed_depth, build_height);
        let pre_gcode = cfg.get_pre_gcode();
        let post_gcode = cfg.get_post_gcode();
        let fan_gcode = cfg.get_fan_gcode();
        let progress_gcode = cfg.get_progress_gcode();
        let layer_gcode = cfg.get_layer_gcode();
        let bed_belt = cfg.get_bed_belt();
        let ga_extruders = cfg.get_extruders();
        let mut extruders: Vec<P3dExtruder> = vec![];
        if ga_extruders.is_empty() {
            println!("use standard nozzle for {}", name);
            extruders.push(P3dExtruder::standard());
        } else {
            for e in ga_extruders {
                match P3dExtruder::from_gridapps(e) {
                    Some(p3dextruder) => extruders.push(p3dextruder),
                    None => return None,
                }
            }
        }
        let machine_name = name.clone();
        let p3d = P3dPrinter {
            source,
            name,
            machine_name,
            origin_center,
            bed_elliptic,
            build_size,
            pre_gcode,
            post_gcode,
            fan_gcode,
            progress_gcode,
            layer_gcode,
            bed_belt,
            extruders,
            heated_bed: true,
        };
        p3d.checked()
    }
    pub fn from_cura(
        all: &HashMap<String, CuraV2>,
        all_extruders: &HashMap<String, CuraExtruderV2>,
        cfg: &CuraV2,
    ) -> Option<Self> {
        let source = cfg.get_source();
        let name = cfg.printer_name();
        let machine_name = cfg.get_machine_name(all).unwrap_or_else(|| name.clone());
        let origin_center = cfg.get_origin_center(all)?;
        let bed_elliptic = cfg.get_bed_elliptic(all)?;
        let bed_width = cfg.get_bed_width(all)?;
        let bed_depth = cfg.get_bed_depth(all)?;
        let build_height = cfg.get_build_height(all)?;
        let build_size = (bed_width, bed_depth, build_height);
        let heated_bed = cfg.get_heated_bed(all)?;
        let pre_gcode = cfg
            .get_pre_gcode(all)?
            .split("\n")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let post_gcode = cfg
            .get_post_gcode(all)?
            .split("\n")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let cura_extruders = cfg.get_extruders(all);
        let mut extruders: Vec<P3dExtruder> = vec![];
        for e in cura_extruders {
            match P3dExtruder::from_cura(all_extruders, &e) {
                Some(p3dextruder) => extruders.push(p3dextruder),
                None => return None,
            }
        }
        if extruders.is_empty() {
            return None;
        }
        if build_size.0 == 0 || build_size.1 == 0 || build_size.2 == 0 {
            return None;
        }
        let p3d = P3dPrinter {
            source,
            name,
            machine_name,
            origin_center,
            bed_elliptic,
            build_size,
            pre_gcode,
            post_gcode,
            fan_gcode: "".to_string(),
            progress_gcode: "".to_string(),
            layer_gcode: "".to_string(),
            bed_belt: false,
            extruders,
            heated_bed,
        };
        p3d.checked()
    }
    pub fn resolve_conflict(mut this: Self, other: Self) -> Option<Self> {
        if this.machine_name == *"Unknown" {
            if other.machine_name == *"Unknown" {
                return None;
            }
            return Some(other);
        }
        if other.machine_name == *"Unknown" {
            return Some(this);
        }
        if this.origin_center != other.origin_center {
            println!("mismatch on origin_center");
            println!("{:#?}", this);
            println!("{:#?}", other);
            return None;
        }
        if this.build_size != other.build_size {
            println!(
                "mismatch on build size: {:?} vs {:?}",
                this.build_size, other.build_size
            );
            this.build_size = (
                this.build_size.0.min(other.build_size.0),
                this.build_size.1.min(other.build_size.1),
                this.build_size.2.min(other.build_size.2),
            );
            println!("resolved to {:?}", this.build_size);
        }
        this.source = format!("{}+{}", this.source, other.source);
        Some(this)
    }
}

lazy_static! {
    static ref XREF: HashMap<&'static str, Option<&'static str>> = [
        ("build_volume_temperature", None),
        ("cool_fan_enabled", None),
        ("cool_fan_speed", None),
        ("date", None),
        ("day", None),
        ("default_material_print_temperature", None),
        ("filament_amount", None),
        ("filament_cost", None),
        ("fill_distance", None),
        ("infill_pattern", None),
        ("infill_sparse_density", None),
        ("initial_extruder_nr", None),
        ("jobname", None),
        ("layer_height", Some("height")),
        ("layer_height_0", Some("height")),
        ("machine_depth", None),
        ("machine_height", Some("z_max")),
        ("machine_name", None),
        ("machine_width", None),
        ("material_bed_temperature", Some("bed_temp")),
        ("material_bed_temperature_layer_0", Some("bed_temp")),
        ("material_brand", None),
        ("material_flow", None),
        ("material_guid", None),
        ("material_id", None),
        ("material_name", None),
        ("material_print_temperature", Some("temp")),
        ("material_print_temperature, 0", Some("temp")),
        ("material_print_temperature, 1", Some("temp")),
        ("material_print_temperature_layer_0", Some("temp")),
        (
            "material_print_temperature_layer_0, initial_extruder_nr",
            None
        ),
        ("material_profile", None),
        ("material_standby_temperature", None),
        ("material_standby_temperature, 0", None),
        ("material_standby_temperature, 1", None),
        ("print_bed_temperature", Some("bed_temp")),
        ("print_speed", None),
        ("print_temperature", Some("temp")),
        ("profile_string", None),
        ("retraction_amount", None),
        ("retraction_retract_speed", None),
        ("retraction_speed", None),
        ("speed_print", None),
        ("speed_topbottom", None),
        ("speed_travel", None),
        ("speed_travel_layer_0", None),
        ("support", None),
        ("support_enable", None),
        ("switch_extruder_retraction_amount", None),
        ("top_bottom_thickness", None),
        ("travel_speed", None),
        ("travel_xy_speed", None),
        ("wall_thickness", None),
    ]
    .into_iter()
    .collect();
}

impl P3dPrinter {
    fn prune(gcode: &mut Vec<String>) {
        let re = Regex::new(r"^[^;]*\{(.*)\}").unwrap();
        for i in 0..gcode.len() {
            if gcode[i].starts_with(";") {
                continue;
            }
            let mut delete = false;
            let mut replace: Option<String> = None;
            for cap in re.captures_iter(&gcode[i]) {
                let m = &cap[1];
                match XREF.get(m) {
                    None => {
                        // keep the variable
                    }
                    Some(None) => {
                        delete = true;
                    }
                    Some(Some(replacement)) => {
                        let modified = gcode[i].replace(m, replacement);
                        replace = Some(modified);
                    } 
                }
            }
            if delete {
                gcode[i] = format!("; PLEASE CHECK: {}", gcode[i]);
            }
            if let Some(replace) = replace {
                gcode[i] = replace;
            }
        }
    }
    fn does_not_contain(&self, pattern: &str) -> bool{
        let re = Regex::new(&format!("^[^;]*{}", pattern)).unwrap();
        for l in self.pre_gcode.iter() {
            if re.is_match(l) {
                return false;
            }
        }
        return true;
    }
    // temp related M codes:
    // M104 set hotend temperature
    // M109 wait for hotend temperature
    // M133 ???
    // M140 set bed temperature
    // M141 set chamber temperature
    // M190 wait for bed temperature
    // M191 wait for chamber temperature
    // M192 wait for probe temperature
    // M901 ???
    fn checked(mut self) -> Option<Self> {
        // check for variables
        P3dPrinter::prune(&mut self.pre_gcode);
        P3dPrinter::prune(&mut self.post_gcode);

        let need_hotend_temp = self.does_not_contain("M104");
        let need_bed_temp = self.heated_bed && self.does_not_contain("M140");

        // Attention: the added commands are here reversed, because inserted at beginning of list
        if need_hotend_temp {
            self.pre_gcode.insert(0,"M109".to_string());
            self.pre_gcode.insert(0,"M104 {temp}".to_string());
        }
        if need_bed_temp {
            self.pre_gcode.insert(0,"M190".to_string());
            self.pre_gcode.insert(0,"M140 {bed_temp}".to_string());
        }

        Some(self)
    }
}
