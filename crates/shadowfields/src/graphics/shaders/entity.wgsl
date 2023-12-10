// import "globals.wgsl"

struct VertexInput {
    // See wgpu::VertexBufferLayout in vertex.rs
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
    // unused:
    @location(3) lm_coords: vec2f,
    @location(4) tangent_u: vec3f,
    @location(5) tangent_v: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) normal: vec3f,
    @location(2) world_position: vec3f,
    @location(3) lightbox: vec4f, // xyz: diffuse color, w: sun factor
};

// arguments from canvas.rs: render_pass.set_vertex_buffer(0, ...), set_vertex_buffer(1, ...)
@vertex
fn vs_main(model: VertexInput, @builtin(instance_index) inst_idx: u32) -> VertexOutput {
    let model_matrix = instance_data[inst_idx].model_matrix;

    var out: VertexOutput;

    let world_position = model_matrix * vec4(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = globals.view_proj * world_position;

    out.tex_coords = model.tex_coords;
    out.normal = normalize((model_matrix * vec4(model.normal, 0.0)).xyz);

    out.lightbox = lightbox_sample(inst_idx, world_position.xyz);

    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let base_color = textureSample(t_diffuse, s_diffuse, in.tex_coords).xyz;
    return lightbox_shader(in.lightbox, base_color, in.world_position, in.normal);
}