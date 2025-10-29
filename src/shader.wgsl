
struct CameraUniform {
    view_proj: mat4x4f,
};
@group(1)@binding(0)
var<uniform> carmera:CameraUniform;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_uv: vec2f,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_uv: vec2f,
}
struct FragmentInput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_uv: vec2f,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = carmera.view_proj * vec4<f32>(vertex.position, 1.0);
    out.tex_uv = vertex.tex_uv;
    return out;
}


@group(0) @binding(0)
var the_texture:texture_2d<f32>;
@group(0) @binding(1)
var the_sampler:sampler;

@fragment
fn fs_main(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return textureSample(the_texture, the_sampler, fragment_in.tex_uv);
}