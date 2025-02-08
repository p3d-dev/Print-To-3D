use std::fmt::{Debug, Formatter};

pub trait GenericExtruder {
    fn get_nozzle(&self) -> f32;
    fn get_filament(&self) -> f32;
    fn get_offset_x(&self) -> f32;
    fn get_offset_y(&self) -> f32;
    fn get_select_gcode(&self) -> String;
    fn get_deselect_gcode(&self) -> String;
}
impl Debug for dyn GenericExtruder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extruder")
            .field("nozzle", &self.get_nozzle())
            .field("filament", &self.get_filament())
            .field("offset_x", &self.get_offset_x())
            .field("offset_y", &self.get_offset_y())
            .field("select_gcode", &self.get_select_gcode())
            .field("deselect_gcode", &self.get_deselect_gcode())
            .finish()
    }
}

#[allow(dead_code)]
pub trait GenericPrinter {
    fn get_license(&self) -> String;
    fn get_source(&self) -> String;
    fn get_origin_center(&self) -> Option<bool>;
    fn get_bed_circle(&self) -> Option<bool>;
    fn get_bed_width(&self) -> Option<u16>;
    fn get_bed_depth(&self) -> Option<u16>;
    fn get_build_height(&self) -> Option<u16>;
    fn get_pre_gcode(&self) -> Option<Vec<String>>;
    fn get_post_gcode(&self) -> Option<Vec<String>>;
    fn get_fan_gcode(&self) -> Option<String>;
    fn get_progress_gcode(&self) -> Option<String>;
    fn get_layer_gcode(&self) -> Option<String>;
    fn get_bed_belt(&self) -> Option<bool>;
    fn get_extruders(&self) -> Option<Vec<&dyn GenericExtruder>>;
    fn get_speed_travel(&self) -> Option<f32>;
}
