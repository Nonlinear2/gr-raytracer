use crate::constants::{IntegrationMethod, Pipeline};

pub const RNG_SEED: u64 = 0;

// Pipeline
pub const PIPELINE: Pipeline = Pipeline::CPU;

// Debug
pub const DEBUG: bool = cfg!(debug_assertions);
pub const DEBUG_RAY_INDEX: u32 = 82292;

// Image
pub const IMAGE_HEIGHT: u32 = 40;
pub const IMAGE_WIDTH: u32 = ((IMAGE_HEIGHT as f32) * 16.0 / 9.0) as u32;

pub const SAMPLES_PER_PIXEL: u32 = if DEBUG { 1 } else {
    1
};

pub const MAX_BOUNCES: u32 = 5; // maximum number of light bounces on objects

// Integration
pub const INTEGRATION_METHOD: IntegrationMethod = IntegrationMethod::EULER;
pub const MAX_INTEGRATION_STEPS: u32 = 10000;
pub const INTEGRATION_STEP_SIZE: f32 = 0.003;

// GPU
pub const WORKGROUP_SIZE: u32 = 64;