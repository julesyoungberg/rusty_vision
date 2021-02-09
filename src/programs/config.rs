use nannou::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub vert: String,
    pub frag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramDefaults {
    pub audio_feature_smoothing: Option<f32>,
    pub camera_position: Option<Vector3<f32>>,
    pub camera_target: Option<Vector3<f32>>,
    pub camera_up: Option<Vector3<f32>>,
    pub color_mode: Option<u32>,
    pub shape_rotation: Option<Vector3<f32>>,
    pub image1: Option<String>,
    pub image2: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramConfig {
    pub pipeline: PipelineConfig,
    pub uniforms: Vec<String>,
    pub defaults: Option<ProgramDefaults>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default: String,
    pub programs: HashMap<String, ProgramConfig>,
}

pub fn get_config() -> Config {
    let json_string = fs::read_to_string("./config.json").expect("Error reading config.json");
    let config: Config =
        serde_json::from_str(json_string.as_str()).expect("Error parsing config.json");
    config
}
