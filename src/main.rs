use shaderx_wgpu::init;
mod app;
mod gfx;

fn main() {
  pollster::block_on(init());
  pollster::block_on(app::App::new());
}
