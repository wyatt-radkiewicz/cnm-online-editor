// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>, 
    @location(1) tex_coords: vec2<f32>, 
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) is_pure_color: u32,
    @location(2) color: vec4<f32>,
};

struct InstanceInput {
    @location(2) pos: vec3<f32>,
    @location(3) size: vec2<f32>,
    @location(4) src: vec4<f32>,
    @location(5) is_pure_color: u32,
    @location(6) color: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    var model_pos: vec3<f32> = model.position;
    model_pos.x *= instance.size.x;
    model_pos.y *= instance.size.y;
    out.clip_position = camera.view_proj * vec4<f32>(model_pos + instance.pos, 1.0);
    out.tex_coords = (model.tex_coords * instance.src.zw) + instance.src.xy;
    out.is_pure_color = instance.is_pure_color;
    out.color = instance.color;
    return out;
}

// // Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32>;
    let dimensions = vec2<f32>(textureDimensions(t_diffuse).xy);
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords / dimensions);

    if in.is_pure_color != 0u {
        color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        if tex_color.x == 0.0 && tex_color.y == 1.0 && tex_color.z == 1.0 {
            discard;
        }
        color = tex_color;
    }

    return color * in.color;
}
