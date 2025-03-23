struct CameraUniform {
  view_proj: mat4x4<f32>
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
   @location(5) model_matrix_0: vec4<f32>,
   @location(6) model_matrix_1: vec4<f32>,
   @location(7) model_matrix_2: vec4<f32>,
   @location(8) model_matrix_3: vec4<f32>,
 };

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
};



@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

override block_coords =
array<vec3<f32>>(
    vec3(1, 0, 0), vec3(1, 0, 1), vec3(0, 0, 1),
    vec3(0, 0, 1), vec3(0, 0, 0), vec3(1, 0, 0),
    vec3(1, 1, 1), vec3(1, 1, 0), vec3(0, 1, 0),
    vec3(0, 1, 0), vec3(0, 1, 1), vec3(1, 1, 1),
    vec3(0, 0, 0), vec3(0, 1, 0), vec3(1, 1, 0),
    vec3(1, 1, 0), vec3(1, 0, 0), vec3(0, 0, 0),
    vec3(1, 0, 0), vec3(1, 1, 0), vec3(1, 1, 1),
    vec3(1, 1, 1), vec3(1, 0, 1), vec3(1, 0, 0),
    vec3(1, 0, 1), vec3(1, 1, 1), vec3(0, 1, 1),
    vec3(0, 1, 1), vec3(0, 0, 1), vec3(1, 0, 1),
    vec3(0, 0, 1), vec3(0, 1, 1), vec3(0, 1, 0),
    vec3(0, 1, 0), vec3(0, 0, 0), vec3(0, 0, 1),
);

override block_tex_coords= array<vec2<f32>>(
    vec2(1, 0),
    vec2(1, 1),
    vec2(0, 1),
    vec2(0, 1),
    vec2(0, 0),
    vec2(1, 0),
);


@vertex
fn vs_block(@location(0) position: vec3<f32>, @builtin(vertex_index) index: u8) -> VertexOutput {
    let coords = var out = VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(block_coords[index], 1.0)
    out.tex_coords = block_tex_coords[index % 6]
    return out
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
