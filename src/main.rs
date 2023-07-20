use ray_tracing_in_one_weekend_webgpu::{gpu, shaders::ray_tracer};

#[tokio::main]
async fn main() {
    let gpu = gpu::GPU::new().await.unwrap();

    let shader = ray_tracer::Shader::new(gpu);

    let output = shader.execute().await;

    println!("P3");
    println!("256 256");
    println!("255");
    for i in output.arr {
        println!("{} {} {}", i.x, i.y, i.z);
    }
}
