// Vertex shader

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
var t_area_sphz: texture_2d<f32>;

@group(0) @binding(3)
var s_area_sphz: sampler;

@group(0) @binding(4)
var t_normalmap: texture_2d<f32>;

@group(0) @binding(5)
var s_normalmap: sampler;

@group(0) @binding(6)
var t_sun_mask: texture_2d<f32>;

@group(0) @binding(7)
var s_sun_mask: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {

    let base_color_tex = textureSample(t_base_color, s_base_color, in.tex_coords).xyz;
    let nm_tex = textureSample(t_normalmap, s_normalmap, in.tex_coords).xyz;
    let area_sphz_tex = textureSample(t_area_sphz, s_area_sphz, in.lm_coords).xyz;
    let sun_mask = textureSample(t_sun_mask, s_sun_mask, in.lm_coords).x; // TODO(performance): use monochrome texture.

	// 0-th order sherical harmonic approximation
    let area_diffuse = area_sphz_tex;

    // https://en.wikipedia.org/wiki/Normal_mapping#Calculation
    let nm = 2.0 * (nm_tex - 0.5);
    let normal = normalize(nm.x * in.tangent_u + nm.y * in.tangent_v + nm.z * in.normal);

	//  https://en.wikipedia.org/wiki/Phong_reflection_model
    //  sun diffuse contribution
    let to_sun = -globals.sun_dir;
    let cos_theta = max(0.0, dot(to_sun, normal));
    let sun_diffuse = globals.sun_color * sun_mask * cos_theta;

	// sun specular contribution
    let reflectivity = 0.4; // TODO: variable
    let sun_specular = reflectivity * sun_mask * specular(globals.sun_dir, in.world_position, normal) * vec3(globals.sun_color);

    return vec4(base_color_tex * (area_diffuse + sun_diffuse) + sun_specular, 1.0);
}