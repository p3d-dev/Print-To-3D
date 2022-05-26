use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Default<T> {
    default_value: Option<T>,
    value: Option<T>,
}

#[derive(Deserialize, Debug)]
struct LongSetting<T> {
    default_value: Option<T>,
}

#[derive(Deserialize, Debug)]
struct MetaDataExtruderV2 {
    //machine: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OverridesExtruderV2 {
    machine_nozzle_size: Option<Default<f32>>,
    machine_nozzle_offset_x: Option<Default<f32>>,
    machine_nozzle_offset_y: Option<Default<f32>>,
    material_diameter: Option<Default<f32>>,
    machine_extruder_start_code: Option<Default<String>>,
    machine_extruder_end_code: Option<Default<String>>,
}

#[derive(Deserialize, Debug)]
struct DetailedSettingsExtruderV2 {
    machine_nozzle_size: Option<LongSetting<f32>>,
    machine_nozzle_offset_x: Option<LongSetting<f32>>,
    machine_nozzle_offset_y: Option<LongSetting<f32>>,
    material_diameter: Option<LongSetting<f32>>,
    machine_extruder_start_code: Option<LongSetting<String>>,
    machine_extruder_end_code: Option<LongSetting<String>>,
}

#[derive(Deserialize, Debug)]
struct MachineSettingsExtruderV2 {
    children: DetailedSettingsExtruderV2,
}

#[derive(Deserialize, Debug)]
struct SettingsExtruderV2 {
    machine_settings: MachineSettingsExtruderV2,
}

#[derive(Deserialize, Debug)]
pub struct CuraExtruderV2 {
    //name: String,
    inherits: Option<String>,
    overrides: Option<OverridesExtruderV2>,
    settings: Option<SettingsExtruderV2>,
    //metadata: Option<MetaDataExtruderV2>,
}
impl CuraExtruderV2 {
    pub fn get_nozzle(&self, all: &HashMap<String, CuraExtruderV2>) -> Option<f32> {
        if let Some(or) = self.overrides.as_ref() {
            if let Some(def) = &or.machine_nozzle_size {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_nozzle_size
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_nozzle(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_filament(&self, all: &HashMap<String, CuraExtruderV2>) -> Option<f32> {
        if let Some(or) = self.overrides.as_ref() {
            if let Some(def) = &or.material_diameter {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .material_diameter
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_filament(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_offset_x(&self, all: &HashMap<String, CuraExtruderV2>) -> Option<f32> {
        if let Some(or) = self.overrides.as_ref() {
            if let Some(def) = &or.machine_nozzle_offset_x {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_nozzle_offset_x
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_offset_x(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_offset_y(&self, all: &HashMap<String, CuraExtruderV2>) -> Option<f32> {
        if let Some(or) = self.overrides.as_ref() {
            if let Some(def) = &or.machine_nozzle_offset_y {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_nozzle_offset_y
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_offset_y(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_select_gcode(&self, all: &HashMap<String, CuraExtruderV2>) -> Option<String> {
        if let Some(or) = self.overrides.as_ref() {
            if let Some(def) = &or.machine_extruder_start_code {
                if let Some(v) = def.value.as_ref().or(def.default_value.as_ref()) {
                    return Some(v.to_string());
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_extruder_start_code
                .as_ref()
                .and_then(|x| x.default_value.as_ref())
            {
                return Some(or.to_string());
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_select_gcode(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_deselect_gcode(&self, all: &HashMap<String, CuraExtruderV2>) -> Option<String> {
        if let Some(or) = self.overrides.as_ref() {
            if let Some(def) = &or.machine_extruder_end_code {
                if let Some(v) = def.value.as_ref().or(def.default_value.as_ref()) {
                    return Some(v.to_string());
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_extruder_end_code
                .as_ref()
                .and_then(|x| x.default_value.as_ref())
            {
                return Some(or.to_string());
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_deselect_gcode(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
}

#[derive(Deserialize, Debug)]
struct MetaDataV2 {
    machine_extruder_trains: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
struct OverridesV2 {
    machine_name: Option<Default<String>>,
    machine_heated_bed: Option<Default<bool>>,
    //machine_nozzle_size: Option<Default<f32>>,
    machine_width: Option<Default<f32>>,
    machine_height: Option<Default<f32>>,
    machine_depth: Option<Default<f32>>,
    machine_center_is_zero: Option<Default<bool>>,
    machine_start_gcode: Option<Default<String>>,
    machine_end_gcode: Option<Default<String>>,
    machine_shape: Option<Default<String>>,
    //material_diameter: Option<Default<f32>>,
}

#[derive(Deserialize, Debug)]
struct DetailedSettingsV2 {
    machine_name: Option<LongSetting<String>>,
    machine_heated_bed: Option<LongSetting<bool>>,
    //machine_nozzle_size: Option<LongSetting<f32>>,
    machine_width: Option<LongSetting<f32>>,
    machine_height: Option<LongSetting<f32>>,
    machine_depth: Option<LongSetting<f32>>,
    machine_center_is_zero: Option<LongSetting<bool>>,
    machine_start_gcode: Option<LongSetting<String>>,
    machine_end_gcode: Option<LongSetting<String>>,
    machine_shape: Option<LongSetting<String>>,
    //material_diameter: Option<LongSetting<f32>>,
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
pub struct CuraV2 {
    name: String,
    inherits: Option<String>,
    overrides: Option<OverridesV2>,
    settings: Option<SettingsV2>,
    metadata: Option<MetaDataV2>,
}

impl CuraV2 {
    pub fn printer_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_machine_name(&self, all: &HashMap<String, CuraV2>) -> Option<String> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_name.as_ref() {
                if let Some(v) = def.value.as_ref().or(def.default_value.as_ref()) {
                    return Some(v.clone());
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_name
                .as_ref()
                .and_then(|x| x.default_value.as_ref())
            {
                return Some(or.clone());
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_machine_name(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_source(&self) -> String {
        "CuraV2".to_string()
    }
    pub fn get_origin_center(&self, all: &HashMap<String, CuraV2>) -> Option<bool> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_center_is_zero.as_ref() {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_center_is_zero
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_origin_center(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    fn string_to_shape(shape: &str) -> Option<bool> {
        match shape {
            "Rectangular" => Some(false),
            "rectangular" => Some(false),
            "elliptic" => Some(true),
            _ => {
                println!("Unknown shape: {}", shape);
                None
            }
        }
    }
    pub fn get_bed_elliptic(&self, all: &HashMap<String, CuraV2>) -> Option<bool> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_shape.as_ref() {
                if let Some(v) = def.value.as_ref().or(def.default_value.as_ref()) {
                    return CuraV2::string_to_shape(v);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_shape
                .as_ref()
                .and_then(|x| x.default_value.as_ref())
            {
                return CuraV2::string_to_shape(or);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_bed_elliptic(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_heated_bed(&self, all: &HashMap<String, CuraV2>) -> Option<bool> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_heated_bed.as_ref() {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_heated_bed
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_heated_bed(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_bed_width(&self, all: &HashMap<String, CuraV2>) -> Option<u16> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_width.as_ref() {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v as u16);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_width
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or as u16);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_bed_width(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_bed_depth(&self, all: &HashMap<String, CuraV2>) -> Option<u16> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_depth.as_ref() {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v as u16);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_depth
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or as u16);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_bed_depth(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_build_height(&self, all: &HashMap<String, CuraV2>) -> Option<u16> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_height.as_ref() {
                if let Some(v) = def.value.or(def.default_value) {
                    return Some(v as u16);
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_height
                .as_ref()
                .and_then(|x| x.default_value)
            {
                return Some(or as u16);
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_build_height(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_pre_gcode(&self, all: &HashMap<String, CuraV2>) -> Option<String> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_start_gcode.as_ref() {
                if let Some(v) = def.value.as_ref().or(def.default_value.as_ref()) {
                    return Some(v.clone());
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_start_gcode
                .as_ref()
                .and_then(|x| x.default_value.as_ref())
            {
                return Some(or.clone());
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_pre_gcode(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_post_gcode(&self, all: &HashMap<String, CuraV2>) -> Option<String> {
        if let Some(overrides) = self.overrides.as_ref() {
            if let Some(def) = overrides.machine_end_gcode.as_ref() {
                if let Some(v) = def.value.as_ref().or(def.default_value.as_ref()) {
                    return Some(v.clone());
                }
            }
        }
        if let Some(settings) = self.settings.as_ref().map(|x| &x.machine_settings.children) {
            if let Some(or) = settings
                .machine_end_gcode
                .as_ref()
                .and_then(|x| x.default_value.as_ref())
            {
                return Some(or.clone());
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_post_gcode(all);
            }
            println!("parent not found: {}", inherits);
        }
        None
    }
    pub fn get_extruders(&self, all: &HashMap<String, CuraV2>) -> Vec<String> {
        if let Some(trains) = self
            .metadata
            .as_ref()
            .and_then(|x| x.machine_extruder_trains.as_ref())
        {
            // support only one nozzle for now
            if let Some(nozzle) = trains.get("0") {
                return vec![nozzle.to_string()];
            }
        }
        if let Some(inherits) = self.inherits.as_ref() {
            if let Some(parent) = all.get(inherits) {
                return parent.get_extruders(all);
            }
            println!("parent not found: {}", inherits);
        }
        vec![]
    }
}
