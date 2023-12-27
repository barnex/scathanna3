// import "globals.wgsl"

struct VertexInput {
    // See wgpu::VertexBufferLayout in vertex.rs
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) velocity: vec3f, // stored as normal
    @location(3) t_range: vec2f, // stored as lm_coords
    // unused:
    @location(4) tangent_u: vec3f,
    @location(5) tangent_v: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) alpha: f32,
};

// arguments from canvas.rs: render_pass.set_vertex_buffer(0, ...), set_vertex_buffer(1, ...)
@vertex
fn vs_main(model: VertexInput, @builtin(instance_index) inst_idx: u32) -> VertexOutput {
    let model_matrix = instance_data[inst_idx].model_matrix;

    let t = instance_data[inst_idx].time;
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    var world_pos = (model_matrix * vec4(model.position + t * model.velocity, 1.0));
    world_pos.y -= 2.0 * t * t; // hard-coded gravity
    out.clip_position = globals.view_proj * world_pos;

	// hack to discard triangle
    if t > model.t_range.y{
        out.clip_position = vec4f(0.0, 0.0, 0.0, 0.0);
    }

    //out.alpha = 1.0 - t;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    //let extra_bright = 6.0; // pump up the brightness
    //color.w *= extra_bright * in.alpha;
    //if color.w == 0.0 {
    //    discard;
    //}
    return color;
}