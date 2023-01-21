struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) v_color: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
  // var pos = array<vec2<f32>, 3>(
  //   // vertex position
  //   vec2<f32>(0.0, 0.5),
  //   vec2<f32>(-0.5, -0.5),
  //   vec2<f32>(0.5, -0.5),
  // );
  // var color = array<vec3<f32>, 3>(
  //   // rgb color
  //   vec3<f32>(1.0, 0.0, 0.0),
  //   vec3<f32>(0.0, 1.0, 0.0),
  //   vec3<f32>(0.0, 0.0, 1.0),
  // );

  var out: VertexOutput;
  // out.position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
  // out.v_color = vec4<f32>(color[in_vertex_index],1.0);
  // return out;
  var pos = array<vec2<f32>,6>(
        vec2<f32>(-0.5,  0.7),
        vec2<f32>( 0.3,  0.6),
        vec2<f32>( 0.5,  0.3),
        vec2<f32>( 0.4, -0.5),
        vec2<f32>(-0.4, -0.4),
        vec2<f32>(-0.3,  0.2)
    );
    out.position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
   return vec4<f32>(1.0, 1.0, 0.0, 1.0);
  // return in.v_color;
}