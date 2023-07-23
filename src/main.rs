use rand::{rngs::ThreadRng, Rng};
use ray_tracing_in_one_weekend_webgpu::{gpu, shaders::ray_tracer};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let gpu = gpu::GPU::new().await.unwrap();

    let shader = ray_tracer::Shader::new(gpu);

    let input = ray_tracer::InputType {
        samples_per_pixel: 200,
        screen_size: glam::UVec2 { x: 400, y: 224 },
        view_box_position: glam::UVec2 { x: 0, y: 0 },
        view_box_size: glam::UVec2 { x: 400, y: 224 },
        spheres: random_scene(),
    };

    println!("executing");
    let output = shader.execute_in_chunks(&input).await;
    println!("executed");

    println!("saving");
    let mut file = tokio::fs::File::create("image.ppm").await.unwrap();
    file.write_all(
        format!(
            "P3\n{} {}\n255\n",
            input.view_box_size.x, input.view_box_size.y
        )
        .as_bytes(),
    )
    .await
    .unwrap();
    for y in (0..input.view_box_size.y).rev() {
        for x in 0..input.view_box_size.x {
            let index = y * input.view_box_size.x + x;
            let pixel = output.pixels[index as usize];
            file.write_all(format!("{} {} {}\n", pixel.x, pixel.y, pixel.z).as_bytes())
                .await
                .unwrap();
        }
    }
    file.flush().await.unwrap();
    println!("saved");
}

fn random_scene() -> Vec<ray_tracer::InputTypeSphere> {
    let mut rng = rand::thread_rng();

    let mut spheres = Vec::new();

    // ground
    spheres.push(ray_tracer::InputTypeSphere {
        center: glam::Vec3 {
            x: 0.0,
            y: -1000.0,
            z: 0.0,
        },
        radius: 1000.0,
        material: ray_tracer::InputTypeMaterial::new_lambertian(glam::Vec3 {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        }),
    });

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f32 = rng.gen();
            let center = glam::Vec3 {
                x: a as f32 + 0.9 * rng.gen::<f32>(),
                y: 0.2,
                z: b as f32 + 0.9 * rng.gen::<f32>(),
            };

            if (center
                - glam::Vec3 {
                    x: 4.0,
                    y: 0.2,
                    z: 0.0,
                })
            .length()
                > 0.9
            {
                let material = if choose_mat < 0.8 {
                    let albedo = random_vec3(&mut rng) * random_vec3(&mut rng);
                    ray_tracer::InputTypeMaterial::new_lambertian(albedo)
                } else if choose_mat < 0.95 {
                    let albedo = random_vec3(&mut rng);
                    let fuzz = rng.gen();
                    ray_tracer::InputTypeMaterial::new_metal(albedo, fuzz)
                } else {
                    ray_tracer::InputTypeMaterial::new_dielectric(1.5)
                };

                spheres.push(ray_tracer::InputTypeSphere {
                    center,
                    radius: 0.2,
                    material,
                })
            }
        }
    }

    spheres.push(ray_tracer::InputTypeSphere {
        center: glam::Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        radius: 1.0,
        material: ray_tracer::InputTypeMaterial::new_dielectric(1.5),
    });

    spheres.push(ray_tracer::InputTypeSphere {
        center: glam::Vec3 {
            x: -4.0,
            y: 1.0,
            z: 0.0,
        },
        radius: 1.0,
        material: ray_tracer::InputTypeMaterial::new_lambertian(glam::Vec3 {
            x: 0.4,
            y: 0.2,
            z: 0.1,
        }),
    });

    spheres.push(ray_tracer::InputTypeSphere {
        center: glam::Vec3 {
            x: 4.0,
            y: 1.0,
            z: 0.0,
        },
        radius: 1.0,
        material: ray_tracer::InputTypeMaterial::new_metal(
            glam::Vec3 {
                x: 0.7,
                y: 0.6,
                z: 0.5,
            },
            0.0,
        ),
    });

    spheres
}

fn random_vec3(rng: &mut ThreadRng) -> glam::Vec3 {
    glam::Vec3 {
        x: rng.gen(),
        y: rng.gen(),
        z: rng.gen(),
    }
}
