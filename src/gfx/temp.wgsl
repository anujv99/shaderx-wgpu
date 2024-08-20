
struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
  @builtin(vertex_index) index: u32,
) -> VertexOutput {
  var output: VertexOutput;
  let x = f32(1 - i32(index)) * 0.5;
  let y = f32(i32(index & 1u) * 2 - 1) * 0.5;
  output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
  return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}
