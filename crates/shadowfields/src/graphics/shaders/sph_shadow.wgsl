// Vertex shader

// mirrors global_uniforms.rs (host data).
struct Globals {
    view_proj: mat4x4f,
    cam_position: vec3f,
    sun_dir: vec3f,
    sun_color: vec3f,
    shadow_centers: array<vec4f, 4>,
};

@group(1) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
    @location(3) lm_coords: vec2f,
    @location(4) tangent_u: vec3f,
    @location(5) tangent_v: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) lm_coords: vec2f,
    @location(3) normal: vec3f,
    @location(4) tangent_u: vec3f,
    @location(5) tangent_v: vec3f,
    @location(6) world_position: vec3f,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.lm_coords = model.lm_coords;
    out.clip_position = globals.view_proj * vec4(model.position, 1.0);
    out.normal = model.normal;
    out.tangent_u = model.tangent_u;
    out.tangent_v = model.tangent_v;
    out.world_position = model.position;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_base_color: texture_2d<f32>;

@group(0) @binding(1)
var s_base_color: sampler;

@group(0) @binding(2)
var t_sphz: texture_2d<f32>;

@group(0) @binding(3)
var s_sphz: sampler;

@group(0) @binding(4)
var t_normalmap: texture_2d<f32>;

@group(0) @binding(5)
var s_normalmap: sampler;

@group(0) @binding(6)
var t_sun_mask: texture_2d<f32>;

@group(0) @binding(7)
var s_sun_mask: sampler;

@group(0) @binding(8)
var t_sphx: texture_2d<f32>;

@group(0) @binding(9)
var s_sphx: sampler;

@group(0) @binding(10)
var t_sphy: texture_2d<f32>;

@group(0) @binding(11)
var s_sphy: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let base_color_tex = textureSample(t_base_color, s_base_color, in.tex_coords).xyz;
    let nm_tex = textureSample(t_normalmap, s_normalmap, in.tex_coords).xyz;
    let sphx_ = textureSample(t_sphx, s_sphx, in.lm_coords).xyz;
    let sphy_ = textureSample(t_sphy, s_sphy, in.lm_coords).xyz;
    let sphz = textureSample(t_sphz, s_sphz, in.lm_coords).xyz;
    let sun_mask = textureSample(t_sun_mask, s_sun_mask, in.lm_coords).x; // TODO(performance): use monochrome texture.


    // https://en.wikipedia.org/wiki/Normal_mapping#Calculation
    let nm = 2.0 * (nm_tex - 0.5);
    let sphx = 2.0 * (sphx_ - 0.5);
    let sphy = 2.0 * (sphy_ - 0.5);

	// 1-th order sherical harmonic approximation
    // TODO: wrong: need quadratic sum
    // TODO: wrong: need to remap x, y to -1..1
    let sph_r = max(0.0, sphx.r * nm.x + sphy.r * nm.y);
    let sph_g = max(0.0, sphx.g * nm.x + sphy.g * nm.y);
    let sph_b = max(0.0, sphx.b * nm.x + sphy.b * nm.y);
    let area_diffuse = sphz * nm.z + 6.6 * vec3(sph_r, sph_g, sph_b); // 🪲 

	//  https://en.wikipedia.org/wiki/Phong_reflection_model
    //  sun diffuse contribution
    let normal = normalize(nm.x * in.tangent_u + nm.y * in.tangent_v + nm.z * in.normal); // ! world space
    let to_sun = -globals.sun_dir;
    let cos_theta = max(0.0, dot(to_sun, normal));
    let sun_diffuse = globals.sun_color * sun_mask * cos_theta;

	// sun specular contribution
    let view_dir = normalize(globals.cam_position - in.world_position);
    let refl_dir = reflect(globals.sun_dir, normal);
    let reflectivity = 0.4; // TODO: variable
    let exponent = 16.0; // TODO: variable
    let sun_specular = (reflectivity * sun_mask * pow(max(0.0, dot(view_dir, refl_dir)), exponent)) * vec3(globals.sun_color);

    var shadow_factor = 1.0;
    let shadow_radius = 0.3;
    let shadow_radius_sq = shadow_radius * shadow_radius;
    let aspect = 1.0 / 3.5;
    for (var i: i32 = 0; i < 4; i++) {
        let p1 = in.world_position;
        let p2 = globals.shadow_centers[i].xyz;
        let d = p1 - p2;

        let dist_sq = ((d.x * d.x + aspect * (d.y * d.y) + d.z * d.z) - shadow_radius_sq) / shadow_radius_sq;

        var my_occlusion = 1.0 / dist_sq;
        if my_occlusion > 0.7 || my_occlusion < 0.0 {
            my_occlusion = 0.7;
        }
        let my_shadow = 1.0 - my_occlusion;
        shadow_factor = min(shadow_factor, my_shadow);
    }

    return vec4(shadow_factor*(base_color_tex * (area_diffuse + sun_diffuse) + sun_specular), 1.0);
}