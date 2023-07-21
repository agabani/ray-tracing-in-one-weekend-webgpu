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
    // exit if pixel is outside of screen space
    if global_id.x >= in.screen_size.x || global_id.y >= in.screen_size.y || global_id.z >= 1u {
        return;
    }

    // declare pixel space
    if global_id.x == 0u && global_id.y == 0u && global_id.z == 0u {
        out.pixel_length = in.screen_size.y * in.screen_size.x;
    }

    // declare constants
    let image_width = in.screen_size.x;
    let image_height = in.screen_size.y;
    let i = global_id.x;
    let j = global_id.y;
    let index = image_width * j + i;

    // calculate pixel color
    let r : f32 = f32(i) / f32(image_width - 1u);
    let g : f32 = f32(j) / f32(image_height - 1u);
    let b : f32 = 0.25f;

    let ir : u32 = u32(255.999 * r);
    let ig : u32 = u32(255.999 * g);
    let ib : u32 = u32(255.999 * b);

    out.pixel[index] = vec3<u32>(ir, ig, ib);
}
