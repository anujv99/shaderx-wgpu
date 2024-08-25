
struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) vert_position: vec4<f32>,
};

@vertex
fn vs_main(
  @builtin(vertex_index) index: u32,
) -> VertexOutput {
  // https://randallr.wordpress.com/2014/06/14/rendering-a-screen-covering-triangle-in-opengl/
  var output: VertexOutput;
  let x = -1.0f + f32((index & 1u) << 2u);
  let y = -1.0f + f32((index & 2u) << 1u);
  output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
  output.vert_position = output.clip_position;
  return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let color = 0.5f + in.vert_position.xyz * 0.5f;
  return vec4<f32>(color, 1.0);
}

