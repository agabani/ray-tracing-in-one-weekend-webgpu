use encase::ShaderType;

use crate::gpu::GPU;

#[derive(Debug, Default, encase::ShaderType)]
pub struct OutputType {
    pub array_length: encase::ArrayLength,
    #[size(runtime)]
    pub arr: Vec<glam::UVec3>,
}

pub struct Shader {
    bind_group_layout: wgpu::BindGroupLayout,
    gpu: GPU,
    pipeline: wgpu::ComputePipeline,
}

impl Shader {
    #[must_use]
    pub fn new(gpu: GPU) -> Self {
        // create the shader
        let shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        // create the interface for the shader
        let bind_group_layout =
            gpu.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: Some(OutputType::min_size()),
                        },
                        count: None,
                    }],
                });

        let pipeline_layout =
            gpu.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = gpu
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
            });

        Self {
            bind_group_layout,
            gpu,
            pipeline,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn execute(&self) -> OutputType {
        // create a buffer for the shader output
        let output_buffer = self.gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: u64::from(OutputType::min_size()) * 256 * 256,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // create a buffer for the result
        let mapping_buffer = self.gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mapping Buffer"),
            size: u64::from(OutputType::min_size()) * 256 * 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // bind the resources to the interface
        let bind_group = self
            .gpu
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: output_buffer.as_entire_binding(),
                }],
            });

        // create the command for the graphics card to execute
        let mut encoder = self
            .gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(4, 4, 4);
        }

        // create the command for the output gpu buffer to be copied to the output cpu buffer
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &mapping_buffer,
            0,
            u64::from(OutputType::min_size()) * 256 * 256,
        );

        // submit the command for processing
        let submission_index = self.gpu.queue().submit(core::iter::once(encoder.finish()));

        // create a future which resolves when the gpu buffer to cpu buffer is complete
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let mapping_slice = mapping_buffer.slice(..);
        mapping_slice.map_async(wgpu::MapMode::Read, |v| sender.send(v).unwrap());

        // constantly poll the gpu
        self.gpu.poll(submission_index).await.unwrap();

        // wait for the future to resolve
        receiver.await.unwrap().unwrap();

        // create a view of the cpu buffer
        let mapping_slice_buffer_view = mapping_slice.get_mapped_range();

        // read the result from the view
        let mut out_value = OutputType::default();
        encase::StorageBuffer::new(mapping_slice_buffer_view.as_ref())
            .read(&mut out_value)
            .unwrap();

        // clean up buffer views and cpu buffer
        drop(mapping_slice_buffer_view);
        mapping_buffer.unmap();

        out_value
    }
}

#[cfg(test)]
mod tests {
    use crate::{gpu::GPU, shaders::workgroup};

    #[tokio::test]
    async fn test() {
        let gpu = GPU::new().await.unwrap();

        let shader = workgroup::Shader::new(gpu);

        let output = shader.execute().await;

        println!("{:?}", output);
    }
}
