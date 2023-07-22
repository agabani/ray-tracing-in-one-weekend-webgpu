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

fn length_squared(e: vec3<f32>) -> f32 {
    return e.x * e.x + e.y * e.y + e.z * e.z;
}

struct HitRecord {
    some: bool,
    point: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    front_face: bool,
}

fn hit_record_set_face_normal(hit_record: HitRecord, ray: Ray, outward_normal: vec3<f32>) -> HitRecord {
    let front_face = dot(ray.direction,outward_normal) < 0.0;
    var normal: vec3<f32>;
    if front_face {
        normal = outward_normal;
    } else {
        normal = -outward_normal;
    }
    return HitRecord(hit_record.some, hit_record.point, normal, hit_record.t, front_face);
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

fn ray_color(ray: Ray, world: World) -> vec3<f32> {
    let hit_record = world_hit(world, ray, 0.0, 10000.0);
    if hit_record.some {
        let color = 0.5 * (hit_record.normal + vec3<f32>(1.0, 1.0, 1.0));
        return color;
    }

    let unit_direction = normalize(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    let color = (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);
    return color;
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
}

fn sphere_hit(sphere: Sphere, ray: Ray, t_min: f32, t_max: f32) -> HitRecord {
    let oc = ray.origin - sphere.center;
    let a = length_squared(ray.direction);
    let half_b = dot(oc, ray.direction);
    let c = length_squared(oc) - (sphere.radius * sphere.radius);
    let discriminant = (half_b * half_b) - (a * c);

    if discriminant < 0.0 {
        return HitRecord(false, vec3(0.0), vec3(0.0), 0.0, false);
    }

    let sqrtd = sqrt(discriminant);
    var root = (-half_b - sqrtd) / a;
    if root < t_min || t_max < root {
        root = (-half_b + sqrtd) / a;
        if root < t_min || t_max < root {
            return HitRecord(false, vec3(0.0), vec3(0.0), 0.0, false);
        }
    }

    let rec_t = root;
    let rec_p = ray_at(ray, rec_t);
    let outward_normal = (rec_p - sphere.center) / sphere.radius;
    var hit_record = HitRecord(true, rec_p, vec3(0.0), rec_t, false);
    hit_record = hit_record_set_face_normal(hit_record, ray, outward_normal);
    return hit_record;
}

struct World {
    objects: array<Sphere, 2>,
}

fn world_hit(world: World, ray: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var hit_record = HitRecord(false, vec3(0.0), vec3(0.0), 0.0, false);
    var closest_so_far = t_max;

    // TODO: turn into a loop...
    let h0 = sphere_hit(world.objects[0], ray, t_min, closest_so_far);
    if h0.some {
        hit_record = h0;
        closest_so_far = h0.t;
    }

    // TODO: turn into a loop...
    let h1 = sphere_hit(world.objects[1], ray, t_min, closest_so_far);
    if h1.some {
        hit_record = h1;
        closest_so_far = h1.t;
    }

    return hit_record;
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

    // World
    var world = World();
    world.objects[0] = Sphere(vec3<f32>(0.0, 0.0, -1.0), 0.5);
    world.objects[1] = Sphere(vec3<f32>(0.0, -100.5, -1.0), 100.0);

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
    let pixel_color = ray_color(ray, world);

    // Save
    out.pixel[index] = write_color(pixel_color);
}
