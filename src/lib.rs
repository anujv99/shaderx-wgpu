mod app;

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
