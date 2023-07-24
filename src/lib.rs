#![warn(clippy::pedantic)]

pub mod cli;
pub mod gpu;
pub mod shaders;

#[derive(Debug)]
pub enum Error {
    Wgpu(wgpu::Error),
    WgpuDeviceNotFound,
    WgpuRequestDeviceError(wgpu::RequestDeviceError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
