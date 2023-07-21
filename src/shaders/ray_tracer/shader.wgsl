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

    // calculate pixel color
    let j = global_id.x;
    let i = global_id.y;

    let r : f32 = f32(i) / f32(in.screen_size.x - 1u);
    let g : f32 = f32(j) / f32(in.screen_size.y - 1u);
    let b : f32 = 0.25f;

    let ir : u32 = u32(255.999 * r);
    let ig : u32 = u32(255.999 * g);
    let ib : u32 = u32(255.999 * b);

    let index = in.screen_size.y * j + i;

    out.pixel[index] = vec3<u32>(ir, ig, ib);
}
