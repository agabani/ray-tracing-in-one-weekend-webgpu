use encase::ShaderType;
use rand::Rng;
use wgpu::util::DeviceExt;

use crate::gpu::GPU;

#[derive(Debug, Default, encase::ShaderType)]
pub struct InputType {
    pub screen_size: glam::UVec2,

    #[size(runtime)]
    pub spheres: Vec<InputTypeSphere>,
}

#[derive(Debug, Default, encase::ShaderType)]
pub struct InputTypeSphere {
    pub center: glam::Vec3,
    pub radius: f32,
    pub material: InputTypeMaterial,
}

#[derive(Debug, Default, encase::ShaderType)]
pub struct InputTypeMaterial {
    albedo: glam::Vec3,
    // 0. background
    // 1. lambertian
    // 2. metal
    // 3. dielectric
    type_: u32,
    fuzz: f32,
    index_of_refraction: f32,
}

impl InputTypeMaterial {
    #[must_use]
    pub fn new_lambertian(albedo: glam::Vec3) -> Self {
        Self {
            albedo,
            type_: 1,
            fuzz: 0.0,
            index_of_refraction: 0.0,
        }
    }

    #[must_use]
    pub fn new_metal(albedo: glam::Vec3, fuzz: f32) -> Self {
        Self {
            albedo,
            type_: 2,
            fuzz,
            index_of_refraction: 0.0,
        }
    }

    #[must_use]
    pub fn new_dielectric(index_of_refraction: f32) -> Self {
        Self {
            albedo: glam::Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            type_: 3,
            fuzz: 0.0,
            index_of_refraction,
        }
    }
}

#[derive(Debug, Default, encase::ShaderType)]
pub struct OutputType {
    pub pixel_length: encase::ArrayLength,
    #[size(runtime)]
    pub pixels: Vec<glam::UVec3>,
}

#[derive(Debug, Default, encase::ShaderType)]
struct RandomType {
    #[size(runtime)]
    values: Vec<f32>,
}

pub struct Shader {
    bind_group_layout: wgpu::BindGroupLayout,
    gpu: GPU,
    pipeline: wgpu::ComputePipeline,
    workgroup_size: glam::UVec3,
}

impl Shader {
    #[must_use]
    pub fn new(gpu: GPU) -> Self {
        // create the shader
        let workgroup_size = glam::UVec3::new(8, 8, 1);

        let shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shader.wgsl")
                        .replace(
                            "@workgroup_size(1)",
                            &format!(
                                "@workgroup_size({}, {}, {})",
                                workgroup_size.x, workgroup_size.y, workgroup_size.z
                            ),
                        )
                        .into(),
                ),
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
                                min_binding_size: Some(InputType::min_size()),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: Some(OutputType::min_size()),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: Some(RandomType::min_size()),
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
            workgroup_size,
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::missing_panics_doc)]
    pub async fn execute(&self, in_value: &InputType) -> OutputType {
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
            size: u64::from(OutputType::min_size())
                * u64::from(in_value.screen_size.x)
                * u64::from(in_value.screen_size.y),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // create a buffer for the shader random
        let mut rng = rand::thread_rng();
        let random_value = RandomType {
            values: (0..1_000_000).map(|_| rng.gen()).collect(),
        };

        let mut random_byte_buffer = Vec::new();
        let mut random_buffer = encase::StorageBuffer::new(&mut random_byte_buffer);

        random_buffer.write(&random_value).unwrap();

        let random_buffer =
            self.gpu
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Random Buffer"),
                    contents: &random_byte_buffer,
                    usage: wgpu::BufferUsages::STORAGE,
                });

        // create a buffer for the result
        let mapping_buffer = self.gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mapping Buffer"),
            size: u64::from(OutputType::min_size())
                * u64::from(in_value.screen_size.x)
                * u64::from(in_value.screen_size.y),
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: random_buffer.as_entire_binding(),
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

            let screen_size = in_value.screen_size.extend(1);
            let mut workgroups = screen_size / self.workgroup_size;
            if screen_size.x % self.workgroup_size.x > 0 {
                workgroups.x += 1;
            }
            if screen_size.y % self.workgroup_size.y > 0 {
                workgroups.y += 1;
            }
            if screen_size.z % self.workgroup_size.z > 0 {
                workgroups.z += 1;
            }
            pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
        }

        // create the command for the output gpu buffer to be copied to the output cpu buffer
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &mapping_buffer,
            0,
            u64::from(OutputType::min_size())
                * u64::from(in_value.screen_size.x)
                * u64::from(in_value.screen_size.y),
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
    use crate::{gpu::GPU, shaders::ray_tracer};

    #[tokio::test]
    async fn test() {
        let gpu = GPU::new().await.unwrap();

        let shader = ray_tracer::Shader::new(gpu);

        let output = shader
            .execute(&ray_tracer::InputType {
                screen_size: glam::UVec2 { x: 256, y: 256 },
                spheres: Vec::new(),
            })
            .await;

        println!("{:?}", output);
    }
}
