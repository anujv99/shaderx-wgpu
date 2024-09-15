use shaderx_wgpu::init;
mod app;
mod gfx;

fn main() {
  pollster::block_on(init());

  let _event_handler = app::EventHandler::new();
}
