use encase::ShaderType;
use wgpu::util::DeviceExt;

use crate::gpu::GPU;

#[derive(Debug, Default, encase::ShaderType)]
pub struct Type {
    pub array_length: encase::ArrayLength,
    pub array_length_call_ret_val: u32,
    pub a: glam::Vec3,
    #[align(16)]
    #[size(runtime)]
    pub arr: Vec<u32>,
}

#[allow(clippy::module_name_repetitions)]
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
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: Some(Type::min_size()),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: Some(Type::min_size()),
                            },
                            count: None,
                        },
                    ],
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
    pub async fn execute(&self, in_value: &Type) -> Type {
        // create a buffer for the shader input
        let mut in_byte_buffer = Vec::new();
        let mut in_buffer = encase::StorageBuffer::new(&mut in_byte_buffer);

        in_buffer.write(in_value).unwrap();

        let input_buffer =
            self.gpu
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Input Buffer"),
                    contents: &in_byte_buffer,
                    usage: wgpu::BufferUsages::STORAGE,
                });

        // create a buffer for the shader output
        let output_buffer = self.gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: in_byte_buffer.len() as _,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // create a buffer for the result
        let mapping_buffer = self.gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mapping Buffer"),
            size: in_byte_buffer.len() as _,
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
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: output_buffer.as_entire_binding(),
                    },
                ],
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
            pass.dispatch_workgroups(1, 1, 1);
        }

        // create the command for the output gpu buffer to be copied to the output cpu buffer
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &mapping_buffer,
            0,
            in_byte_buffer.len() as _,
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
        let mut out_value: Type = Type::default();
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
    use encase::ArrayLength;

    use crate::{gpu::GPU, shaders::example};

    #[tokio::test]
    async fn example() {
        let gpu = GPU::new().await.unwrap();

        let shader = example::Shader::new(gpu);

        let a_i = example::Type {
            array_length: ArrayLength,
            array_length_call_ret_val: 4,
            a: glam::Vec3::new(5.0, 4.0, 6.0),
            arr: vec![45, 46, 47, 48, 49],
        };
        let b_i = example::Type {
            array_length: ArrayLength,
            array_length_call_ret_val: 4,
            a: glam::Vec3::new(5.0, 4.0, 6.0),
            arr: vec![45],
        };

        let a_f = shader.execute(&a_i);
        let b_f = shader.execute(&b_i);

        let b_o = b_f.await;
        let a_o = a_f.await;

        println!("{:?}", a_o);
        println!("{:?}", b_o);
    }
}
