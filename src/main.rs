use shaderx_wgpu::init;
mod app;

fn main() {
  pollster::block_on(init());
  pollster::block_on(app::App::new());
}
