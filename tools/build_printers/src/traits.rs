
pub trait GenericExtruder {
    fn get_nozzle(&self) -> f32;
    fn get_filament(&self) -> f32;
    fn get_offset_x(&self) -> f32;
    fn get_offset_y(&self) -> f32;
    fn get_select_gcode(&self) -> String;
    fn get_deselect_gcode(&self) -> String;
}

pub trait GenericPrinter {
    fn get_license(&self) -> String;
    fn get_source(&self) -> String;
    fn get_origin_center(&self) -> bool;
    fn get_bed_circle(&self) -> bool;
    fn get_bed_width(&self) -> u16;
    fn get_bed_depth(&self) -> u16;
    fn get_build_height(&self) -> u16;
    fn get_pre_gcode(&self) -> Vec<String>;
    fn get_post_gcode(&self) -> Vec<String>;
    fn get_fan_gcode(&self) -> String;
    fn get_progress_gcode(&self) -> String;
    fn get_layer_gcode(&self) -> String;
    fn get_bed_belt(&self) -> bool;
    fn get_extruders(&self) -> Vec<&dyn GenericExtruder>;
}