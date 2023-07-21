struct InputType {
    data_set_size: vec3<u32>,
}

struct OutputType {
    array_length: u32,
    arr: array<vec3<u32>>,
}

@group(0) @binding(0)
var<storage> in: InputType;

@group(0) @binding(1)
var<storage, read_write> out: OutputType;

@compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    if global_id.x >= in.data_set_size.x || global_id.y >= in.data_set_size.y || global_id.z >= in.data_set_size.z {
        return;
    }

    if global_id.x == 0u && global_id.y == 0u && global_id.z == 0u {
        out.array_length = in.data_set_size.z * in.data_set_size.y * in.data_set_size.x;
    }

    let index =
        global_id.z * in.data_set_size.y * in.data_set_size.x
      + global_id.y                      * in.data_set_size.x
      + global_id.x;

    out.arr[index] = global_id;
    // out.arr[index] = local_id;
}
