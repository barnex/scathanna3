struct Globals {
    view_proj: mat4x4f,
    cam_position: vec3f,
    sun_dir: vec3f,
    sun_color: vec3f,
};

@group(1) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    // unused:
    @location(2) normal: vec3f,
    @location(3) lm_coords: vec2f,
    @location(4) tangent_u: vec3f,
    @location(5) tangent_v: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = globals.view_proj * vec4(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var color: vec4f = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    color.w = 0.25;
    return color;
}