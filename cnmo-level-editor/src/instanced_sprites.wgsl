// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>, 
    @location(1) tex_coords: vec2<f32>, 
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(2) pos: vec3<f32>,
    @location(3) size: vec2<f32>,
    @location(4) src: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    var model_pos: vec3<f32> = model.position;
    model_pos.x *= instance.size.x;
    model_pos.y *= instance.size.y;
    out.clip_position = vec4<f32>(model_pos + instance.pos, 1.0);
    out.tex_coords = (model.tex_coords * instance.src.zw) + instance.src.xy;
    return out;
}

// // Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
