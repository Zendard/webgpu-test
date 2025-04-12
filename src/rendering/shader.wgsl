struct Camera {
  view_pos: vec4<f32>,
  view_proj: mat4x4<f32>
};
@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<uniform> chunk: vec2<i32>;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  //@location(0) tex_coords: vec2<f32>
  @location(0) color: u32,
};

struct InstanceInput  {
  @location(5) raw_description: u32,
};

const FACE_TO_SIN_COS_X = array<f32, 12>(
  0, 1,
  0, 1,
 -1, 0,
  1, 0,
  0, 1,
  0, 1,
);

const FACE_TO_SIN_COS_Y = array<f32, 12>(
 -1, 0,
  1, 0,
  0, 1,
  0, 1,
  0,-1,
  0, 1,
);

const TOP_FACE_TRANSLATION = mat4x4<f32>(
  1, 0, 0, 0,
  0, 1, 0, 0,
  0, 0, 1, 0,
  0, 0, 0, 1,
);

const FRONT_FACE_TRANSLATION = mat4x4<f32>(
  1, 0, 0, 0,
  0, 1, 0, 0,
  0, 0, 1, 0,
  -1, -1, 0, 1,
);

const BACK_FACE_TRANSLATION = mat4x4<f32>(
  1, 0, 0, 0,
  0, 1, 0, 0,
  0, 0, 1, 0,
  0, 0, 0, 1,
);

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
  let data = instance.raw_description;
  let face = (data >> 18) & 7;
  let position_x = (data >> 12) & 63;
  let position_y = (data >> 6 ) & 63;
  let position_z = (data) & 63;
              
  let sin_x = FACE_TO_SIN_COS_X[face*2];
  let cos_x = FACE_TO_SIN_COS_X[face*2 + 1];
  let sin_y = FACE_TO_SIN_COS_Y[face*2];
  let cos_y = FACE_TO_SIN_COS_Y[face*2 + 1];

  let rotation_matrix_x = mat4x4<f32>(
    1, 0      , 0     , 0,
    0, cos_x  , sin_x , 0,
    0, -sin_x , cos_x , 0,
    0, 0      , 0     , 1,
  );

  let rotation_matrix_y = mat4x4<f32>(
    cos_y , 0, -sin_y , 0,
    0     , 1, 0      , 0,
    sin_y , 0, cos_y  , 0,
    0     , 0, 0      , 1,
  );

  let translation_matrix = mat4x4<f32>(
    1              , 0              , 0              , 0,
    0              , 1              , 0              , 0,
    0              , 0              , 1              , 0,
    f32(position_x), f32(position_y), f32(position_z), 1,
  );

  let chunk_translation_matrix = mat4x4<f32>(
    1           , 0, 0           , 0,
    0           , 1, 0           , 0,
    0           , 0, 1           , 0,
    f32(chunk.x) * 32, 0, f32(chunk.y) * 32, 1,
  );


  var tranform_matrix = chunk_translation_matrix * translation_matrix * rotation_matrix_x * rotation_matrix_y;

  // Top face needs to be moved up
  if face == 2 {
    tranform_matrix *= TOP_FACE_TRANSLATION;
  // Front face need to be moved down
  } else if face == 4 && position_y > 0 {
    tranform_matrix *= FRONT_FACE_TRANSLATION;
  // Back face needs to be moved back
  } else if face == 5 {
    tranform_matrix *= BACK_FACE_TRANSLATION;
  }

  let world_position = tranform_matrix * vec4<f32>(model.position, 1.); 

  var out: VertexOutput;
  out.clip_position = camera.view_proj * world_position;
  //out.tex_coords = model.tex_coords;
  out.color = face;
  return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let face = in.color;
  if face == 0 {
    return vec4<f32>(0,0,0,1);
  } else if face == 1 {
    return vec4<f32>(0.5,0,0,1);
  } else if face == 2 {
    return vec4<f32>(0,0.5,0,1);
  } else if face == 3 {
    return vec4<f32>(0,0,0.5,1);
  } else if face == 4 {
    return vec4<f32>(0.5,0.5,0,1);
  } else if face == 5 {
    return vec4<f32>(0,0.5,0.5,1);
  } else {
    return vec4<f32>(0.5,0.5,0.5,1);
  } 
   //return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

// 0 -> left   -> black
// 1 -> right  -> red
// 2 -> top    -> green
// 3 -> bottom -> blue
// 4 -> front  -> yellow
// 5 -> back   -> light blue

