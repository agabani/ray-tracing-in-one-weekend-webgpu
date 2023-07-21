const workgroup_size_x : u32 = 12u;
const workgroup_size_y : u32 = 12u;
const workgroup_size_z : u32 = 12u;

struct OutputType {
    array_length: u32,
    arr: array<vec3<u32>>,
}

@group(0) @binding(0)
var<storage, read_write> out: OutputType;

@compute @workgroup_size(3, 3, 3)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let index =
        global_id.z * workgroup_size_y * workgroup_size_x
      + global_id.y                    * workgroup_size_x
      + global_id.x;

    out.array_length = workgroup_size_z * workgroup_size_y * workgroup_size_x;
    out.arr[index] = global_id;
    // out.arr[index] = local_id;
}
