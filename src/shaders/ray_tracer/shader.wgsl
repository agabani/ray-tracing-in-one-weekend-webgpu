/*
 * ============================================================================
 * Input Storage Buffer
 * ============================================================================
 */
struct InputType {
    screen_size: vec2<u32>,
    view_box_position: vec2<u32>,
    view_box_size: vec2<u32>,
    spheres: array<Sphere>,
}

@group(0) @binding(0)
var<storage> in: InputType;

/*
 * ============================================================================
 * Output Storage Buffer
 * ============================================================================
 */
struct OutputType {
    pixel_length: u32,
    pixel: array<vec3<u32>>,
}

@group(0) @binding(1)
var<storage, read_write> out: OutputType;

/*
 * ============================================================================
 * Random Storage Buffer
 * ============================================================================
 */
struct RandomType {
    values: array<f32>,
}

@group(0) @binding(2)
var<storage> random_type: RandomType;

/*
 * ============================================================================
 * Mathematical Functions
 * ============================================================================
 */
fn length_squared(e: vec3<f32>) -> f32 {
    return e.x * e.x + e.y * e.y + e.z * e.z;
}

fn near_zero(e: vec3<f32>) -> bool {
    let s = 0.00000001;
    let a = abs(e);
    return a.x < s && a.y < s && a.z < s;
}

fn reflect(v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    return v - 2.0 * dot(v, n) * n;
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = pow((1.0 - ref_idx) / (1.0 + ref_idx), 2.0);
    return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

fn refract(uv: vec3<f32>, n: vec3<f32>, etai_over_etat: f32) -> vec3<f32> {
    let cos_theta = min(dot(-uv, n), 1.0);
    let r_out_perp =  etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -sqrt(abs(1.0 - length_squared(r_out_perp))) * n;
    return r_out_perp + r_out_parallel;
}

/*
 * ============================================================================
 * Random Functions
 * ============================================================================
 */
var<private> random_index : u32 = 0;

fn random() -> f32 {
    if random_index >= arrayLength(&random_type.values) {
        random_index = 0u;
    }
    let value = random_type.values[random_index];
    random_index += 1u;
    return value;
}

fn random_between(min: f32, max: f32) -> f32 {
    return min + (max - min) * random();
}

fn random_init(index: u32) {
    random_index = index;
    random_index = u32(random() * f32(arrayLength(&random_type.values)));
}

fn random_in_unit_disk() -> vec3<f32> {
    var p: vec3<f32>;
    loop {
        p = vec3<f32>(random_between(-1.0, 1.0), random_between(-1.0, 1.0), 0.0);
        if length_squared(p) < 1.0 {
            break;
        }
    }
    return p;
}

fn random_in_unit_sphere() -> vec3<f32> {
    var p: vec3<f32>;
    loop {
        p = random_vec3_between(-1.0, 1.0);
        if length_squared(p) < 1.0 {
            break;
        }
    }
    return p;
}

fn random_unit_vector() -> vec3<f32> {
    return normalize(random_in_unit_sphere());
}

fn random_vec3() -> vec3<f32> {
    return vec3<f32>(random(), random(), random());
}

fn random_vec3_between(min: f32, max: f32) -> vec3<f32> {
    return vec3<f32>(random_between(min, max), random_between(min, max), random_between(min, max));
}

/*
 * ============================================================================
 * Camera
 * ============================================================================
 */
struct Camera {
    origin: vec3<f32>,
    horizontal: vec3<f32>,
    vertical: vec3<f32>,
    lower_left_corner: vec3<f32>,
    u: vec3<f32>,
    v: vec3<f32>,
    w: vec3<f32>,
    lens_radius: f32,
}

fn camera_new(
    lookfrom: vec3<f32>,
    lookat: vec3<f32>,
    vup: vec3<f32>,
    vfov: f32,
    aspect_ratio: f32,
    aperture: f32,
    focus_dist: f32,
) -> Camera {
    let theta = radians(vfov);
    let h = tan(theta / 2.0);
    let viewport_height = 2.0 * h;
    let viewport_width = aspect_ratio * viewport_height;

    let w = normalize(lookfrom - lookat);
    let u = normalize(cross(vup, w));
    let v = cross(w, u);

    let origin = lookfrom;
    let horizontal = focus_dist * viewport_width * u;
    let vertical = focus_dist * viewport_height * v;
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

    let lens_radius = aperture / 2.0;

    return Camera(origin, horizontal, vertical, lower_left_corner, u, v, w, lens_radius);
}

fn camera_get_ray(camera: Camera, s: f32, t: f32) -> Ray {
    let rd = camera.lens_radius * random_in_unit_disk();
    let offset = camera.u * rd.x + camera.v * rd.y;

    return ray_new(
        camera.origin + offset,
        camera.lower_left_corner + s * camera.horizontal + t * camera.vertical - camera.origin - offset
    );
}

/*
 * ============================================================================
 * Hit Record
 * ============================================================================
 */
struct HitRecord {
    some: bool,
    point: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    front_face: bool,
    material: Material,
}

fn hit_record_new_some(point: vec3<f32>, normal: vec3<f32>, t: f32, front_face: bool, material: Material) -> HitRecord {
    return HitRecord(true, point, normal, t, front_face, material);
}

fn hit_record_new_none() -> HitRecord {
    return HitRecord(false, vec3<f32>(), vec3<f32>(), 0.0, false, material_default());
}

fn hit_record_set_face_normal(hit_record: HitRecord, ray: Ray, outward_normal: vec3<f32>) -> HitRecord {
    let front_face = dot(ray.direction,outward_normal) < 0.0;
    var normal: vec3<f32>;
    if front_face {
        normal = outward_normal;
    } else {
        normal = -outward_normal;
    }
    return HitRecord(hit_record.some, hit_record.point, normal, hit_record.t, front_face, hit_record.material);
}

/*
 * ============================================================================
 * Material
 * ============================================================================
 */
struct Material {
    albedo: vec3<f32>,
    // 0. background
    // 1. lambertian
    // 2. metal
    // 3. dielectric
    type_: u32,
    fuzz: f32,
    index_of_refraction: f32,
}

fn material_default() -> Material {
    return Material(vec3<f32>(), 0u, 0.0, 0.0);
}

fn material_new_lambertian(albedo: vec3<f32>) -> Material {
    return Material(albedo, 1u, 0.0, 0.0);
}

fn material_new_metal(albedo: vec3<f32>, fuzz: f32) -> Material {
    return Material(albedo, 2u, fuzz, 0.0);
}

fn material_new_dielectric(index_of_refraction: f32) -> Material {
    return Material(vec3<f32>(1.0, 1.0, 1.0), 3u, 0.0, index_of_refraction);
}

struct MaterialScatterResult {
    some: bool,
    attenuation: vec3<f32>,
    scattered: Ray,
}

fn material_scatter(material: Material, ray_in: Ray, hit_record: HitRecord) -> MaterialScatterResult {
    switch material.type_ {
        case 1u: {
            var scatter_direction = hit_record.normal + random_unit_vector();

            if near_zero(scatter_direction) {
                scatter_direction = hit_record.normal;
            }

            let scattered = ray_new(hit_record.point, scatter_direction);
            return MaterialScatterResult(true, material.albedo, scattered);
        }
        case 2u: {
            let reflected = reflect(normalize(ray_in.direction), hit_record.normal);
            let scattered = ray_new(hit_record.point, reflected + material.fuzz * random_in_unit_sphere());
            let some = dot(scattered.direction, hit_record.normal) >  0.0;
            return MaterialScatterResult(some, material.albedo, scattered);
        }
        case 3u: {
            let attenuation = vec3<f32>(1.0, 1.0, 1.0);
            var refraction_ratio = material.index_of_refraction;
            if hit_record.front_face {
                refraction_ratio = 1.0 / material.index_of_refraction;
            }

            let unit_direction = normalize(ray_in.direction);
            let cos_theta = min(dot(-unit_direction, hit_record.normal), 1.0);
            let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

            let cannot_refract = refraction_ratio * sin_theta > 1.0;
            var direction: vec3<f32>;

            if cannot_refract || reflectance(cos_theta, refraction_ratio) > random() {
                direction = reflect(unit_direction, hit_record.normal);
            } else {
                direction = refract(unit_direction, hit_record.normal, refraction_ratio);
            }

            let scattered = ray_new(hit_record.point, direction);
            return MaterialScatterResult(true, attenuation, scattered);
        }
        default: {
            return MaterialScatterResult(false, vec3<f32>(0.0, 0.0, 0.0), ray_default());
        }
    }
}

/*
 * ============================================================================
 * Ray
 * ============================================================================
 */
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_default() -> Ray {
    return Ray(vec3<f32>(0.0), vec3<f32>(0.0));
}

fn ray_new(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    return Ray(origin, direction);
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

fn ray_color(ray: Ray, world: World) -> vec3<f32> {
    var current_ray = ray;
    var depth = 0i;
    var material_scatter_results = array<MaterialScatterResult, 50>();

    for (; depth < 50i; depth = depth + 1i){
        let hit_record = world_hit(world, current_ray, 0.001, 10000.0);
        if hit_record.some {
            let material_scatter_result = material_scatter(hit_record.material, current_ray, hit_record);
            material_scatter_results[depth] = material_scatter_result;

            if material_scatter_result.some {
                current_ray = material_scatter_result.scattered;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let unit_direction = normalize(current_ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    var color = (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);
    depth -= 1i;

    for (; depth >= 0i; depth = depth - 1i) {
        let material_scatter_results = material_scatter_results[depth];
        color = material_scatter_results.attenuation * color;
    }

    return color;
}

/*
 * ============================================================================
 * Sphere
 * ============================================================================
 */
struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material,
}

fn sphere_new(center: vec3<f32>, radius: f32, material: Material) -> Sphere {
    return Sphere(center, radius, material);
}

fn sphere_hit(sphere: Sphere, ray: Ray, t_min: f32, t_max: f32) -> HitRecord {
    let oc = ray.origin - sphere.center;
    let a = length_squared(ray.direction);
    let half_b = dot(oc, ray.direction);
    let c = length_squared(oc) - (sphere.radius * sphere.radius);
    let discriminant = (half_b * half_b) - (a * c);

    if discriminant < 0.0 {
        return hit_record_new_none();
    }

    let sqrtd = sqrt(discriminant);
    var root = (-half_b - sqrtd) / a;
    if root < t_min || t_max < root {
        root = (-half_b + sqrtd) / a;
        if root < t_min || t_max < root {
            return hit_record_new_none();
        }
    }

    let rec_t = root;
    let rec_p = ray_at(ray, rec_t);
    let outward_normal = (rec_p - sphere.center) / sphere.radius;
    var hit_record = HitRecord(true, rec_p, vec3(0.0), rec_t, false, sphere.material);
    hit_record = hit_record_set_face_normal(hit_record, ray, outward_normal);
    return hit_record;
}

/*
 * ============================================================================
 * World
 * ============================================================================
 */
struct World {
    a: u32,
}

fn world_hit(world: World, ray: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var hit_record = hit_record_new_none();
    var closest_so_far = t_max;

    for (var index = 0u; index < arrayLength(&in.spheres); index = index + 1u) {
        let h = sphere_hit(in.spheres[index], ray, t_min, closest_so_far);
        if h.some {
            hit_record = h;
            closest_so_far = h.t;
        }
    }

    return hit_record;
}

/*
 * ============================================================================
 * Write
 * ============================================================================
 */
fn write_color(color: vec3<f32>, samples_per_pixel: u32) -> vec3<u32> {
    var r = color.x;
    var g = color.y;
    var b = color.z;

    let scale = 1.0 / f32(samples_per_pixel);
    r = sqrt(scale * r);
    g = sqrt(scale * g);
    b = sqrt(scale * b);

    let ir : u32 = u32(255.999 * clamp(r, 0.0, 0.999));
    let ig : u32 = u32(255.999 * clamp(g, 0.0, 0.999));
    let ib : u32 = u32(255.999 * clamp(b, 0.0, 0.999));
    return vec3<u32>(ir, ig, ib);
}

/*
 * ============================================================================
 * Main
 * ============================================================================
 */
@compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // Exit
    if global_id.x >= in.view_box_size.x || global_id.y >= in.view_box_size.y || global_id.z >= 1u {
        return;
    }

    // Pixel Space
    if global_id.x == 0u && global_id.y == 0u && global_id.z == 0u {
        out.pixel_length = in.screen_size.y * in.screen_size.x;
    }

    // Invocation
    let i = in.view_box_position.x + global_id.x;
    let j = in.view_box_position.y + global_id.y;
    let index = in.view_box_size.x * j + i;

    // Initialization
    random_init(index);

    // Image
    let image_width = in.screen_size.x;
    let image_height = in.screen_size.y;
    let aspect_ratio = f32(image_width) / f32(image_height);
    let samples_per_pixel = 200u;

    // World
    var world = World(0u);

    // Camera
    let lookfrom = vec3<f32>(13.0, 2.0, 3.0);
    let lookat = vec3<f32>(0.0, 0.0, 0.0);
    let vup = vec3<f32>(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let camera = camera_new(
        lookfrom,
        lookat,
        vup,
        20.0,
        aspect_ratio,
        aperture,
        dist_to_focus
    );

    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    // Calculate
    var pixel_color = vec3<f32>();

    for (var s = 0u; s < samples_per_pixel; s = s + 1u) {
        let u = (f32(i) + random()) / f32(image_width - 1u);
        let v = (f32(j) + random()) / f32(image_height - 1u);
        let ray = camera_get_ray(camera, u, v);
        pixel_color += ray_color(ray, world);
    }

    // Save
    out.pixel[index] = write_color(pixel_color, samples_per_pixel);
}
