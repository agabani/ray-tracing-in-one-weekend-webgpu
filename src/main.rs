use ray_tracing_in_one_weekend_webgpu::{gpu, shaders::example};

#[tokio::main]
async fn main() {
    let gpu = gpu::GPU::new().await.unwrap();

    let shader = example::Shader::new(gpu);

    let input = example::Type {
        a: glam::Vec3::new(5.0, 4.0, 6.0),
        arr: vec![45, 46, 47, 48, 49],
        ..Default::default()
    };

    let output = shader.execute(&input).await;

    println!("{:?}", output);
}
