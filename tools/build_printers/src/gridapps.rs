use serde::Deserialize;

use crate::traits::*;

#[derive(Deserialize, Debug)]
struct Extruder01 {
    nozzle: f32,
    filament: f32,
    offset_x: f32,
    offset_y: f32,
    select: Option<Vec<String>>,
    deselect: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Command01 {
    fan_power: String,
    progress: Option<String>,
    layer: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Settings01 {
    //nozzle_size: Option<f32>,
    origin_center: bool,
    bed_width: u16,
    bed_depth: u16,
    build_height: u16,
    bed_circle: Option<bool>,
    bed_belt: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct GridApps01 {
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
    extSelect: Option<Vec<String>>,
    extDeselect: Option<Vec<String>>,
    extOffsetX: f32,
    extOffsetY: f32,
}

#[derive(Deserialize, Debug)]
struct Settings02 {}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GridApps02 {
    //deviceName: String,
    bedWidth: u16,
    bedDepth: u16,
    maxHeight: f32,
    bedRound: bool,
    bedBelt: bool,
    //deviceZMax: Option<i16>,
    originCenter: bool,
    //extrudeAbs: Option<bool>,
    gcodePre: Vec<String>,
    gcodePost: Vec<String>,
    gcodeLayer: Vec<String>,
    gcodeTrack: Vec<String>,
    gcodeFan: Vec<String>,
    extruders: Vec<Extruder02>,
    //settings: Option<Settings02>,
}

impl GenericExtruder for Extruder01 {
    fn get_nozzle(&self) -> f32 {
        self.nozzle
    }
    fn get_filament(&self) -> f32 {
        self.filament
    }
    fn get_offset_x(&self) -> f32 {
        self.offset_x
    }
    fn get_offset_y(&self) -> f32 {
        self.offset_y
    }
    fn get_select_gcode(&self) -> String {
        self.select
            .as_ref()
            .map(|x| x.join("\n"))
            .unwrap_or_else(|| "".to_string())
    }
    fn get_deselect_gcode(&self) -> String {
        self.deselect
            .as_ref()
            .map(|x| x.join("\n"))
            .unwrap_or_else(|| "".to_string())
    }
}
impl GenericExtruder for Extruder02 {
    fn get_nozzle(&self) -> f32 {
        self.extNozzle
    }
    fn get_filament(&self) -> f32 {
        self.extFilament
    }
    fn get_offset_x(&self) -> f32 {
        self.extOffsetX
    }
    fn get_offset_y(&self) -> f32 {
        self.extOffsetY
    }
    fn get_select_gcode(&self) -> String {
        self.extSelect
            .as_ref()
            .map(|x| x.join("\n"))
            .unwrap_or_else(|| "".to_string())
    }
    fn get_deselect_gcode(&self) -> String {
        self.extDeselect
            .as_ref()
            .map(|x| x.join("\n"))
            .unwrap_or_else(|| "".to_string())
    }
}
impl GenericPrinter for GridApps01 {
    fn get_license(&self) -> String {
        "MIT".to_string()
    }
    fn get_source(&self) -> String {
        "GridApps01".to_string()
    }
    fn get_origin_center(&self) -> Option<bool> {
        Some(self.settings.origin_center)
    }
    fn get_bed_circle(&self) -> Option<bool> {
        Some(self.settings.bed_circle.unwrap_or(false))
    }
    fn get_bed_width(&self) -> Option<u16> {
        Some(self.settings.bed_width)
    }
    fn get_bed_depth(&self) -> Option<u16> {
        Some(self.settings.bed_depth)
    }
    fn get_build_height(&self) -> Option<u16> {
        Some(self.settings.build_height)
    }
    fn get_pre_gcode(&self) -> Option<Vec<String>> {
        Some(self.pre.iter().cloned().collect::<Vec<String>>())
    }
    fn get_post_gcode(&self) -> Option<Vec<String>> {
        Some(self.post.iter().cloned().collect::<Vec<String>>())
    }
    fn get_fan_gcode(&self) -> Option<String> {
        Some(self.cmd.fan_power.clone())
    }
    fn get_progress_gcode(&self) -> Option<String> {
        Some(
            self.cmd
                .progress
                .as_ref()
                .unwrap_or(&"".to_string())
                .clone(),
        )
    }
    fn get_layer_gcode(&self) -> Option<String> {
        Some(
            self.cmd
                .layer
                .as_ref()
                .map(|x| x.join("\n"))
                .unwrap_or_else(|| "".to_string()),
        )
    }
    fn get_bed_belt(&self) -> Option<bool> {
        Some(self.settings.bed_belt.unwrap_or(false))
    }
    fn get_extruders(&self) -> Option<Vec<&dyn GenericExtruder>> {
        if let Some(extruders) = self.extruders.as_ref() {
            return Some(
                extruders
                    .iter()
                    .map(|e| e as &dyn GenericExtruder)
                    .collect::<Vec<_>>(),
            );
        }
        Some(vec![])
    }
    fn get_speed_travel(&self) -> Option<f32> {
        None
    }
}
impl GenericPrinter for GridApps02 {
    fn get_license(&self) -> String {
        "MIT".to_string()
    }
    fn get_source(&self) -> String {
        "GridApps02".to_string()
    }
    fn get_origin_center(&self) -> Option<bool> {
        Some(self.originCenter)
    }
    fn get_bed_circle(&self) -> Option<bool> {
        Some(self.bedRound)
    }
    fn get_bed_width(&self) -> Option<u16> {
        Some(self.bedWidth)
    }
    fn get_bed_depth(&self) -> Option<u16> {
        Some(self.bedDepth)
    }
    fn get_build_height(&self) -> Option<u16> {
        Some(self.maxHeight as u16)
    }
    fn get_pre_gcode(&self) -> Option<Vec<String>> {
        Some(self.gcodePre.iter().cloned().collect::<Vec<String>>())
    }
    fn get_post_gcode(&self) -> Option<Vec<String>> {
        Some(self.gcodePost.iter().cloned().collect::<Vec<String>>())
    }
    fn get_fan_gcode(&self) -> Option<String> {
        Some(self.gcodeFan.join("\n"))
    }
    fn get_progress_gcode(&self) -> Option<String> {
        Some(self.gcodeTrack.join("\n"))
    }
    fn get_layer_gcode(&self) -> Option<String> {
        Some(self.gcodeLayer.join("\n"))
    }
    fn get_bed_belt(&self) -> Option<bool> {
        Some(self.bedBelt)
    }
    fn get_extruders(&self) -> Option<Vec<&dyn GenericExtruder>> {
        Some(
            self.extruders
                .iter()
                .map(|e| e as &dyn GenericExtruder)
                .collect::<Vec<_>>(),
        )
    }
    fn get_speed_travel(&self) -> Option<f32> {
        None
    }
}
