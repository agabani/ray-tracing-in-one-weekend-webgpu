struct InputType {
    screen_size: vec2<u32>,
}

struct OutputType {
    pixel_length: u32,
    pixel: array<vec3<u32>>,
}

@group(0) @binding(0)
var<storage> in: InputType;

@group(0) @binding(1)
var<storage, read_write> out: OutputType;

@compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // Exit
    if global_id.x >= in.screen_size.x || global_id.y >= in.screen_size.y || global_id.z >= 1u {
        return;
    }

    // Pixel Space
    if global_id.x == 0u && global_id.y == 0u && global_id.z == 0u {
        out.pixel_length = in.screen_size.y * in.screen_size.x;
    }

    // Invocation
    let i = global_id.x;
    let j = global_id.y;
    let index = in.screen_size.x * j + i;

    // Image
    let image_width = in.screen_size.x;
    let image_height = in.screen_size.y;
    let aspect_ratio = f32(image_width) / f32(image_height);

    // Camera
    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let origin = vec3<f32>(0.0, 0.0, 0.0);
    let horizontal = vec3<f32>(viewport_width, 0.0, 0.0);
    let vertical = vec3<f32>(0.0, viewport_height, 0.0);
    let lower_left_corner = origin - horizontal/2.0 - vertical/2.0 - vec3<f32>(0.0, 0.0, focal_length);

    // Calculate
    let r : f32 = f32(i) / f32(image_width - 1u);
    let g : f32 = f32(j) / f32(image_height - 1u);
    let b : f32 = 0.25f;

    let ir : u32 = u32(255.999 * r);
    let ig : u32 = u32(255.999 * g);
    let ib : u32 = u32(255.999 * b);

    // Save
    out.pixel[index] = vec3<u32>(ir, ig, ib);
}
