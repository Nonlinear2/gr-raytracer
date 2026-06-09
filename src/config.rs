use serde::Deserialize;
use std::{error::Error, fs};

#[derive(Debug, Deserialize)]
pub struct Config {
	pub image_height: u32,
	pub aspect_ratio: f32,
	pub samples_per_pixel: u32,
	pub rng_seed: u64,
	pub max_integration_steps: u32,
	pub max_bounces: u32,
}

impl Config {
	pub fn load() -> Result<Self, Box<dyn Error>> {
		let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/config.toml");
		let contents = fs::read_to_string(path)?;
		Ok(toml::from_str(&contents)?)
	}
}