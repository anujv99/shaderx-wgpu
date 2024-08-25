
use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use winit::{event::{Event, WindowEvent}, event_loop::{EventLoop, EventLoopWindowTarget}, window::{Window, WindowBuilder}};

#[path = "./types.rs"] mod types;
use crate::gfx::gfx_state::GfxState;

#[derive(Debug)]
#[wasm_bindgen]
pub struct App {
  window: Arc<Window>,
  gfx: Arc<Mutex<GfxState>>,
}

#[wasm_bindgen]
impl App {
  fn event_handler(event: Event<()>, control_flow: &EventLoopWindowTarget<()>, window: Arc<Window>, gfx: Arc<Mutex<GfxState>>) {
    match event {
      Event::Resumed => {
        log::debug!("[app] event: resumed");
      },
      Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
        WindowEvent::CloseRequested => control_flow.exit(),
        WindowEvent::Resized(physical_size) => {
          log::debug!("[app] event: resized {:?}", physical_size);
          let mut gfx = gfx.lock().expect("[app] failed to lock gfx");
          gfx.resize(*physical_size);
        },
        WindowEvent::RedrawRequested => {
          let mut gfx = gfx.lock().expect("[app] failed to lock gfx");
          window.request_redraw();

          match gfx.render() {
            Ok(_) => (),
            Err(e) => log::error!("[app] failed to render: {:?}", e),
          }
        },
        _ => (),
      },
      _ => (),
    }
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub async fn new() -> Self {
    log::info!("[app] creating window");

    let event_loop = EventLoop::new().expect("[app] failed to create event loop");
    let window = Arc::new(WindowBuilder::new().build(&event_loop).expect("[app] failed to create window"));

    let gfx = Arc::new(Mutex::new(GfxState::new(window.clone()).await));

    let app = Self {
      window: window.clone(),
      gfx: gfx.clone(),
    };

    event_loop.run(move |event, control_flow| {
      Self::event_handler(event, control_flow, window.clone(), gfx.clone());
    }).expect("[app] failed to run event loop");

    app
  }

  // FIXME: remove async constructor
  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen(constructor)]
  pub async fn new(ts_params: types::IAppParams) -> Self {
    log::info!("[app] creating window");

    let event_loop = EventLoop::new().expect("[app] failed to create event loop");
    let window = Arc::new(WindowBuilder::new().build(&event_loop).expect("[app] failed to create window"));

    use winit::platform::web::EventLoopExtWebSys;
    use winit::platform::web::WindowExtWebSys;

    // create canvas on wasm
    {
      let params: JsValue = ts_params.into();
      let mut container_id = String::from("canvas-container");

      if params.is_object() {
        if js_sys::Reflect::has(&params, &JsValue::from_str("containerId")).unwrap() {
          container_id = js_sys::Reflect::get(&params, &JsValue::from_str("containerId")).unwrap().as_string().unwrap();
        }
      }

      web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
          let dst = doc.get_element_by_id(&container_id).expect("[app] failed to get canvas container");
          let canvas = web_sys::Element::from(window.clone().canvas()?);
          canvas.set_attribute("width", "100%").ok()?;
          canvas.set_attribute("height", "100%").ok()?;
          canvas.set_attribute("style", "width: 100%; height: 100%;").ok()?;
          dst.append_child(&canvas).ok()?;
          Some(())
        })
        .expect("[app] failed to append canvas");
    }

    let gfx = Arc::new(Mutex::new(GfxState::new(window.clone()).await));

    let app = Self {
      window: window.clone(),
      gfx: gfx.clone(),
    };

    event_loop.spawn(move |event, control_flow| {
      Self::event_handler(event, control_flow, window.clone(), gfx.clone());
    });

    app
  }

  #[wasm_bindgen(js_name = "updateShader")]
  pub async fn update_shader(&self, shader_source: String) -> types::ShaderCompilationInfo {
    let mut gfx = self.gfx.lock().expect("[app] failed to lock gfx");
    let result = gfx.compile_shader(&shader_source).await;
    if result.messages.is_empty() {
      gfx.update_shader(&shader_source);
    }
    result.into()
  }

  #[wasm_bindgen(js_name = "compileShader")]
  pub async fn compile_shader(&self, shader_source: String) -> types::ShaderCompilationInfo {
    let gfx = self.gfx.lock().expect("[app] failed to lock gfx");
    let result = gfx.compile_shader(&shader_source).await;
    result.into()
  }
}
