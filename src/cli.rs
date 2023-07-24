use std::path::PathBuf;

use clap::Parser;

#[allow(clippy::module_name_repetitions)]
#[derive(Parser, Debug)]
#[command(about, version)]
pub struct CliArgs {
    /// chunk size (width:height)
    #[arg(long, default_value = "64:64")]
    pub chunk_size: String,

    /// output
    #[arg(long, default_value = "image.ppm", value_hint = clap::ValueHint::DirPath)]
    pub output: PathBuf,

    /// samples per pixel
    #[arg(long, default_value = "500")]
    pub samples_per_pixel: u32,

    /// screen size (width:height)
    #[arg(long, default_value = "1920:1080")]
    pub screen_size: String,

    /// view box position (x_offset:y_offset)
    #[arg(long)]
    pub view_box_position: Option<String>,

    /// view box size (width:height)
    #[arg(long)]
    pub view_box_size: Option<String>,
}

#[must_use]
pub fn parse() -> CliArgs {
    CliArgs::parse()
}

/// # Panics
///
/// Panics if value is not in format `u32:u32`.
#[must_use]
pub fn str_to_vec2(value: &str) -> glam::UVec2 {
    let (x, y) = value.split_once(':').unwrap();
    glam::UVec2 {
        x: x.parse().unwrap(),
        y: y.parse().unwrap(),
    }
}
