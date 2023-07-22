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

fn hit_sphere(center: vec3<f32>, radius: f32, ray: Ray) -> f32 {
    let oc = ray.origin - center;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return -1.0;
    } else {
        let normal = (-b - sqrt(discriminant)) / (2.0 * a);
        return normal;
    }
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

fn ray_color(ray: Ray) -> vec3<f32> {
    var t = hit_sphere(vec3<f32>(0.0, 0.0, -1.0), 0.5, ray);
    if t > 0.0 {
        let n = normalize(ray_at(ray, t) - vec3<f32>(0.0, 0.0, -1.0));
        let color = 0.5 * (n + vec3<f32>(1.0, 1.0, 1.0));
        return color;
    }

    let unit_direction = normalize(ray.direction);
    t = 0.5 * (unit_direction.y + 1.0);
    let color = (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);
    return color;
}

fn write_color(color: vec3<f32>) -> vec3<u32> {
    let r : u32 = u32(255.999 * color.x);
    let g : u32 = u32(255.999 * color.y);
    let b : u32 = u32(255.999 * color.z);
    return vec3<u32>(r, g, b);
}

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
    let u = f32(i) / f32(image_width - 1u);
    let v = f32(j) / f32(image_height - 1u);
    let ray = Ray(origin, lower_left_corner + u * horizontal + v * vertical - origin);
    let pixel_color = ray_color(ray);

    // Save
    out.pixel[index] = write_color(pixel_color);
}
