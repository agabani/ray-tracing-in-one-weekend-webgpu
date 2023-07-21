use ray_tracing_in_one_weekend_webgpu::{gpu, shaders::ray_tracer};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let gpu = gpu::GPU::new().await.unwrap();

    let shader = ray_tracer::Shader::new(gpu);

    let input = ray_tracer::InputType {
        screen_size: glam::UVec2 { x: 256, y: 256 },
    };

    println!("executing");
    let output = shader.execute(&input).await;
    println!("executed");

    println!("saving");
    let mut file = tokio::fs::File::create("image.ppm").await.unwrap();
    file.write(format!("P3\n{} {}\n255\n", input.screen_size.x, input.screen_size.y).as_bytes())
        .await
        .unwrap();
    for y in (0..input.screen_size.y).rev() {
        for x in 0..input.screen_size.x {
            let index = y * input.screen_size.x + x;
            let pixel = output.pixels[index as usize];
            file.write(format!("{} {} {}\n", pixel.x, pixel.y, pixel.z).as_bytes())
                .await
                .unwrap();
        }
    }
    file.flush().await.unwrap();
    println!("saved");
}
