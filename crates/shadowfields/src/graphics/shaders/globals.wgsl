// Common functionality.
// Concatenated with some WGSL files (think "#import") by shader_pack.rs.

struct Globals {
    view_proj: mat4x4f,
    cam_position: vec3f,
    sun_dir: vec3f,
    sun_color: vec3f,
};

struct InstanceData {
    bounding_box_size: vec3f,
    _padding2: f32,
    bounding_box_bottom: vec3f,
    _padding3: f32,
    lightbox: array<array<array<vec4f, 2>,2>,2>,
    model_matrix: mat4x4f,
    time: f32,
    unused: f32,
    _padding: vec2f,
}

@group(1) @binding(0)
var<uniform> globals: Globals;

@group(2) @binding(0)
var<storage, read> instance_data: array<InstanceData>;


fn lightbox_shader(lightbox: vec4f, base_color: vec3f, world_position: vec3f, normal: vec3f) -> vec4f {
    let sun_mask = lightbox.w;

    let cos_theta_sun = max(0.0, -dot(globals.sun_dir, normal));
    let sun_diffuse = (sun_mask * cos_theta_sun) * globals.sun_color;

    let sun_specular = (sun_mask * specular(globals.sun_dir, world_position, normal)) * vec3(globals.sun_color);

	// Add a little fake directionality to ambient,
    // else object in the shadows appear extremely flat.
    let base_ambient = lightbox.xzy;
    let fake_ambient_dir = vec3f(0.3, 0.9, 0.0);
    let ambient = base_ambient * 0.5*(1.0 + dot(normal, fake_ambient_dir));

    let color = (sun_diffuse + ambient) * base_color + sun_specular;

    return vec4(color, 1.0);
}


fn specular(light_dir: vec3f, world_position: vec3f, normal: vec3f) -> f32 {
	// sun specular contribution
    let view_dir = normalize(globals.cam_position - world_position);
    let refl_dir = reflect(light_dir, normal);
    let exponent = 8.0; // TODO: variable
    return pow(max(0.0, dot(view_dir, refl_dir)), exponent);
}

// vertex light = interpolation between the lightbox vertices.
// See light_box.rs
// https://en.wikipedia.org/wiki/Trilinear_interpolation
fn lightbox_sample(inst_idx: u32, world_position: vec3f) -> vec4f {

    let lightbox_position = world_position.xyz - instance_data[inst_idx].bounding_box_bottom; // << TODO: should be center

    var uvw = (lightbox_position / instance_data[inst_idx].bounding_box_size) + vec3(0.5, 0.0, 0.5); // << wrt bottom << TODO: should be -0.5

    uvw.x = clamp(uvw.x, 0.0, 1.0);
    uvw.y = clamp(uvw.y, 0.0, 1.0);
    uvw.z = clamp(uvw.z, 0.0, 1.0);

	// Debug: show cleary when uvw is out of bounds: red: too high, blue: too low
    // TODO: remove for production: replace by clamp
    // if uvw.x > 1.0 || uvw.y > 1.0 || uvw.z > 1.0 {
    //     return vec3f(1.0, 0.0, 0.0);
    // }
    // if uvw.x < 0.0 || uvw.y < 0.0 || uvw.z < 0.0 {
    //     return vec3f(0.0, 0.0, 1.0);
    // }



    let c = instance_data[inst_idx].lightbox;
    let c00 = mix(c[0][0][0], c[0][0][1], uvw.x);
    let c01 = mix(c[0][1][0], c[0][1][1], uvw.x);
    let c10 = mix(c[1][0][0], c[1][0][1], uvw.x);
    let c11 = mix(c[1][1][0], c[1][1][1], uvw.x);
    let c0 = mix(c00, c01, uvw.y);
    let c1 = mix(c10, c11, uvw.y);
    return mix(c0, c1, uvw.z);
}
