use ray_tracing_in_one_weekend_webgpu::{gpu, shaders::ray_tracer};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let gpu = gpu::GPU::new().await.unwrap();

    let shader = ray_tracer::Shader::new(gpu);

    let input = ray_tracer::InputType {
        screen_size: glam::UVec2 { x: 400, y: 225 },
        spheres: vec![
            ray_tracer::InputTypeSphere {
                center: glam::Vec3 {
                    x: 0.0,
                    y: -100.5,
                    z: -1.0,
                },
                radius: 100.0,
                material: ray_tracer::InputTypeMaterial::new_lambertian(glam::Vec3 {
                    x: 0.8,
                    y: 0.8,
                    z: 0.0,
                }),
            },
            ray_tracer::InputTypeSphere {
                center: glam::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
                radius: 0.5,
                material: ray_tracer::InputTypeMaterial::new_lambertian(glam::Vec3 {
                    x: 0.1,
                    y: 0.2,
                    z: 0.5,
                }),
            },
            ray_tracer::InputTypeSphere {
                center: glam::Vec3 {
                    x: -1.0,
                    y: 0.0,
                    z: -1.0,
                },
                radius: 0.5,
                material: ray_tracer::InputTypeMaterial::new_dielectric(1.5),
            },
            ray_tracer::InputTypeSphere {
                center: glam::Vec3 {
                    x: -1.0,
                    y: 0.0,
                    z: -1.0,
                },
                radius: -0.4,
                material: ray_tracer::InputTypeMaterial::new_dielectric(1.5),
            },
            ray_tracer::InputTypeSphere {
                center: glam::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: -1.0,
                },
                radius: 0.5,
                material: ray_tracer::InputTypeMaterial::new_metal(
                    glam::Vec3 {
                        x: 0.8,
                        y: 0.6,
                        z: 0.2,
                    },
                    0.0,
                ),
            },
        ],
    };

    println!("executing");
    let output = shader.execute(&input).await;
    println!("executed");

    println!("saving");
    let mut file = tokio::fs::File::create("image.ppm").await.unwrap();
    file.write_all(
        format!("P3\n{} {}\n255\n", input.screen_size.x, input.screen_size.y).as_bytes(),
    )
    .await
    .unwrap();
    for y in (0..input.screen_size.y).rev() {
        for x in 0..input.screen_size.x {
            let index = y * input.screen_size.x + x;
            let pixel = output.pixels[index as usize];
            file.write_all(format!("{} {} {}\n", pixel.x, pixel.y, pixel.z).as_bytes())
                .await
                .unwrap();
        }
    }
    file.flush().await.unwrap();
    println!("saved");
}
