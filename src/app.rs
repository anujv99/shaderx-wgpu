
use std::sync::Arc;

use wasm_bindgen::prelude::*;
use winit::{event::{Event, WindowEvent}, event_loop::{EventLoop, EventLoopWindowTarget}, window::{Window, WindowBuilder}};

#[derive(Debug, Default)]
#[wasm_bindgen]
pub struct App {
  window: Option<Arc<Window>>,
}

impl App {
  fn event_handler(event: Event<()>, control_flow: &EventLoopWindowTarget<()>, window: Arc<Window>) {
    match event {
      Event::Resumed => {
        log::debug!("[app] event: resumed");
      },
      Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
        WindowEvent::CloseRequested => control_flow.exit(),
        _ => (),
      },
      _ => (),
    }
  }

  pub async fn new() -> Self {
    log::info!("[app] creating window");

    let event_loop = EventLoop::new().expect("[app] failed to create event loop");
    let window = Arc::new(WindowBuilder::new().build(&event_loop).expect("[app] failed to create window"));

    // create canvas on wasm
    #[cfg(target_arch = "wasm32")] {
      use winit::platform::web::WindowExtWebSys;
      web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
          let dst = doc.get_element_by_id("wasm-canvas").expect("[app] failed to get canvas container");
          let canvas = web_sys::Element::from(window.clone().canvas()?);
          canvas.set_attribute("width", "100%").ok()?;
          canvas.set_attribute("height", "100%").ok()?;
          canvas.set_attribute("style", "width: 100%; height: 100%;").ok()?;
          dst.append_child(&canvas).ok()?;
          Some(())
        })
        .expect("[app] failed to append canvas");
    }

    let app = Self {
      window: Some(window.clone()),
    };

    event_loop.run(move |event, control_flow| {
      Self::event_handler(event, control_flow, window.clone());
    }).expect("[app] failed to run event loop");

    app
  }
}
