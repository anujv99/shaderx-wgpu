
use wasm_bindgen::prelude::wasm_bindgen;

mod app;

#[wasm_bindgen(start)]
pub async fn init() {
  cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
      std::panic::set_hook(Box::new(console_error_panic_hook::hook));
      console_log::init_with_level(log::Level::Debug).expect("[lib] failed to initialize logger");
    } else {
      env_logger::init();
    }
  }
}

#[wasm_bindgen(js_name = getMaxDimension2D)]
pub fn get_max_dimension_2d() -> u32 {
  let limits = {
    #[cfg(not(target_arch = "wasm32"))] {
      wgpu::Limits::default()
    }
    #[cfg(target_arch = "wasm32")] {
      wgpu::Limits::downlevel_webgl2_defaults()
    }
  };

  limits.max_texture_dimension_2d
}

