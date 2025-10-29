
struct CameraUniform {
    view_proj: mat4x4f,
};
@group(1)@binding(0)
var<uniform> carmera:CameraUniform;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_uv: vec2f,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4f,
    @location(6) model_matrix_1: vec4f,
    @location(7) model_matrix_2: vec4f,
    @location(8) model_matrix_3: vec4f,
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
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4f(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.pos = carmera.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
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