const image_width : u32 = 256u;
const image_height : u32 = 256u;

struct OutputType {
    array_length: u32,
    arr: array<vec3<u32>>,
}

@group(0) @binding(0)
var<storage, read_write> out: OutputType;

@compute @workgroup_size(1, 1, 1)
fn main() {
    out.array_length = image_width * image_height;
    out.arr[0] = vec3<u32>(0u, 0u, 0u);
    out.arr[1] = vec3<u32>(10u, 10u, 10u);

    for (var j = 0u; j < image_height; j = j + 1u) {
        for (var i = 0u; i < image_width; i = i + 1u) {
            let r : f32 = f32(i) / f32(image_width - 1u);
            let g : f32 = f32(j) / f32(image_height - 1u);
            let b : f32 = 0.25f;

            let ir : u32 = u32(255.999 * r);
            let ig : u32 = u32(255.999 * g);
            let ib : u32 = u32(255.999 * b);

            let index = image_height * j + i;

            out.arr[index] = vec3<u32>(ir, ig, ib);
        }
    }
}
