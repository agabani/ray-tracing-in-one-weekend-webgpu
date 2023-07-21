use ray_tracing_in_one_weekend_webgpu::{gpu, shaders::ray_tracer};

#[tokio::main]
async fn main() {
    let gpu = gpu::GPU::new().await.unwrap();

    let shader = ray_tracer::Shader::new(gpu);

    let input = ray_tracer::InputType {
        screen_size: glam::UVec2 { x: 256, y: 256 },
    };

    let output = shader.execute(&input).await;

    println!("P3");
    println!("{} {}", input.screen_size.x, input.screen_size.y);
    println!("255");
    for y in (0..input.screen_size.y).rev() {
        for x in 0..input.screen_size.x {
            let index = y * input.screen_size.x + x;
            let pixel = output.pixels[index as usize];
            println!("{} {} {}", pixel.x, pixel.y, pixel.z);
        }
    }
}
