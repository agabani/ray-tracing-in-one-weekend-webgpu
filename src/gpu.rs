use std::sync::Arc;

use crate::Error;

#[derive(Clone)]
pub struct GPU {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl GPU {
    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// # Errors
    ///
    /// Will return `Err` if a GPU device cannot be found or a connection cannot be made.
    pub async fn new() -> crate::Result<Self> {
        // create a wgpu instance
        let instance = wgpu::Instance::default();

        // create a handle to the graphics card
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(Error::WgpuDeviceNotFound)?;

        // create a connection to the graphics card
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .map_err(Error::WgpuRequestDeviceError)?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }

    /// # Errors
    ///
    /// Will return `Err` if task failed to execute to completion.
    pub async fn poll(
        &self,
        submission_index: wgpu::SubmissionIndex,
    ) -> Result<(), tokio::task::JoinError> {
        let device = self.device.clone();
        tokio::task::spawn_blocking(move || {
            device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
        })
        .await
    }

    #[must_use]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
