// import "globals.wgsl"

struct VertexInput {
    // See wgpu::VertexBufferLayout in vertex_kf.rs
    @location(0) tex_coords: vec2f,
    @location(1) position1: vec3f,
    @location(2) position2: vec3f,
    @location(3) normal1: vec3f,
    @location(4) normal2: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) normal: vec3f,
    @location(2) world_position: vec3f,
    @location(3) lightbox: vec4f,
};

// arguments from canvas.rs: render_pass.set_vertex_buffer(0, ...), set_vertex_buffer(1, ...)
@vertex
fn vs_main(model: VertexInput, @builtin(instance_index) inst_idx: u32) -> VertexOutput {
    let model_matrix = instance_data[inst_idx].model_matrix;

    let t = instance_data[inst_idx].time;
    let position = (1.0 - t) * model.position1 + t * model.position2;
    let normal = normalize((1.0 - t) * model.normal1 + t * model.normal2);

    var out: VertexOutput;

    let world_position = model_matrix * vec4(position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = globals.view_proj * world_position;

    out.tex_coords = model.tex_coords;
    out.normal = normalize((model_matrix * vec4(normal, 0.0)).xyz);

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