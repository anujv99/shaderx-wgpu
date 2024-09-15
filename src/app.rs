
use std::{borrow::Borrow, sync::{Arc, Mutex, MutexGuard}};

use wasm_bindgen::prelude::*;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{self, ControlFlow, EventLoop, EventLoopProxy}, window::{Window, WindowId}};

#[path = "./types.rs"] pub mod types;
use crate::gfx::gfx_state::GfxState;

enum UserEvents {
  CreateInstance((String, Option<js_sys::Function>)),
  DestroyInstance(WindowId),
  UpdateShader((String, js_sys::Function, WindowId)),
  CompileShader((String, js_sys::Function, WindowId)),
}

struct AppInstance {
  window: Arc<Window>,
  gfx: GfxState,
  pub handle: types::InstanceHandle,
}

#[derive(Default)]
struct App {
  instances: Arc<Mutex<Vec<Arc<Mutex<AppInstance>>>>>,
}

#[wasm_bindgen]
pub struct EventHandler {
  event_loop: EventLoopProxy<UserEvents>,
}

impl AppInstance {
  async fn update_shader(instance: Arc<Mutex<AppInstance>>, shader_source: String, callback: js_sys::Function) {
    let mut instance = instance.lock().expect("[app] failed to lock instance");

    let result = instance.gfx.compile_shader(&shader_source).await;
    if result.messages.is_empty() {
      instance.gfx.update_shader(&shader_source);
    }

    let result: types::ShaderCompilationInfo = result.into();
    let _ = callback.call1(&JsValue::NULL, &JsValue::from(result));
  }

  async fn compile_shader(instance: Arc<Mutex<AppInstance>>, shader_source: String, callback: js_sys::Function) {
    let instance = instance.lock().expect("[app] failed to lock instance");

    let result = instance.gfx.compile_shader(&shader_source).await;
    let result: types::ShaderCompilationInfo = result.into();
    let _ = callback.call1(&JsValue::NULL, &JsValue::from(result));
  }

  #[cfg(not(target_arch = "wasm32"))]
  async fn create_instance(window: Arc<Window>, instances: Arc<Mutex<Vec<Arc<Mutex<AppInstance>>>>>) {
    let gfx = GfxState::new(window.clone()).await;

    let mut instances = instances.lock().expect("[app] failed to lock instances");

    let handle = types::InstanceHandle {
      window_id: window.id(),
    };

    instances.push(Arc::new(Mutex::new(AppInstance {
      window: window.clone(),
      gfx,
      handle,
    })));
  }

  #[cfg(target_arch = "wasm32")]
  async fn create_instance(
    window: Arc<Window>,
    instances: Arc<Mutex<Vec<Arc<Mutex<AppInstance>>>>>,
    container_id: String,
    callback: Option<js_sys::Function>,
  ) {
    use std::rc::Rc;

    use winit::platform::web::WindowExtWebSys;
    use winit::platform::web::EventLoopExtWebSys;

    web_sys::window()
      .and_then(|win| win.document())
      .and_then(|doc| {
        let dst = doc.get_element_by_id(&container_id).expect("[app] failed to get canvas container");
        let canvas = web_sys::Element::from(window.canvas()?);
        canvas.set_attribute("width", "100%").ok()?;
        canvas.set_attribute("height", "100%").ok()?;
        canvas.set_attribute("style", "width: 100%; height: 100%;").ok()?;
        dst.append_child(&canvas).ok()?;
        Some(())
      })
    .expect("[app] failed to append canvas");

    let gfx = GfxState::new(window.clone()).await;
    let mut instances = instances.lock().expect("[app] failed to lock instances");

    let handle = types::InstanceHandle {
      window_id: window.id(),
    };

    instances.push(Arc::new(Mutex::new(AppInstance {
      window: window.clone(),
      gfx,
      handle,
    })));

    if let Some(callback) = callback {
      let _ = callback.call1(&JsValue::NULL, &JsValue::from(handle));
    }
  }
}

impl ApplicationHandler<UserEvents> for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    log::warn!("[app] event: resumed");

    let window = Arc::new(event_loop.create_window(Window::default_attributes()).expect("[app] failed to create window"));

    #[cfg(not(target_arch = "wasm32"))]
    pollster::block_on(AppInstance::create_instance(window.clone(), self.instances.clone()));
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    let instances = self.instances.lock().expect("[app] failed to lock instances");
    let instance = instances.iter().find(|instance| {
      let instance = instance.lock().expect("[app] failed to lock instance");
      instance.window.id() == window_id
    });

    let mut instance = match instance {
      Some(instance) => instance.lock().expect("[app] failed to lock instance"),
      None => return,
    };

    match event {
      WindowEvent::CloseRequested => {
        log::warn!("[app] event: close_requested");
        event_loop.exit();
      },
      WindowEvent::RedrawRequested => {
        instance.window.request_redraw();
        let _ = instance.gfx.render();
      },
      WindowEvent::Resized(size) => {
        log::warn!("[app] event: resized: {:?}", size);
        instance.gfx.resize(size);
      },
      WindowEvent::MouseInput { device_id, state, button } => {
        log::warn!("[app] event: mouse_input: {:?}, {:?}, {:?}", device_id, state, button);
      },
      _ => {},
    }
  }

  fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvents) {
    match event {
      UserEvents::CreateInstance((container_id, callback)) => {
        log::warn!("[app] event: create_instance: {}", container_id);
        
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).expect("[app] failed to create window"));

        #[cfg(not(target_arch = "wasm32"))]
        pollster::block_on(AppInstance::create_instance(window.clone(), self.instances.clone()));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(AppInstance::create_instance(window.clone(), self.instances.clone(), container_id, callback));
      },
      UserEvents::UpdateShader((shader_source, callback, window_id)) => {
        log::warn!("[app] event: update_shader: {:?}", window_id);
        let instances = self.instances.lock().expect("[app] failed to lock instances");

        for instance in instances.iter() {
          {
            let instance = instance.lock().expect("[app] failed to lock instance");
            if instance.window.id() != window_id {
              continue;
            }
          }

          wasm_bindgen_futures::spawn_local(AppInstance::update_shader(instance.clone(), shader_source.clone(), callback.clone()));
        }
      },
      UserEvents::CompileShader((shader_source, callback, window_id)) => {
        log::warn!("[app] event: compile_shader: {:?}", window_id);
        let instances = self.instances.lock().expect("[app] failed to lock instances");

        for instance in instances.iter() {
          {
            let instance = instance.lock().expect("[app] failed to lock instance");
            if instance.window.id() != window_id {
              continue;
            }
          }

          wasm_bindgen_futures::spawn_local(AppInstance::compile_shader(instance.clone(), shader_source.clone(), callback.clone()));
        }
      },
      UserEvents::DestroyInstance(window_id) => {
        log::warn!("[app] event: destroy_instance: {:?}", window_id);
        let mut instances = self.instances.lock().expect("[app] failed to lock instances");
        instances.retain(|instance| {
          let instance = instance.lock().expect("[app] failed to lock instance");
          instance.window.id() != window_id
        });
      },
    }
  }
}

#[wasm_bindgen]
impl EventHandler {
  #[cfg(not(target_arch = "wasm32"))]
  pub fn new() -> Self {
    let event_loop = EventLoop::<UserEvents>::with_user_event().build().expect("[app] failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let event_loop_proxy = event_loop.create_proxy();

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("[app] failed to run event loop");

    return Self {
      event_loop: event_loop_proxy,
    };
  }

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::<UserEvents>::with_user_event().build().expect("[app] failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let event_loop_proxy = event_loop.create_proxy();

    event_loop.spawn_app(App::default());

    return Self {
      event_loop: event_loop_proxy,
    };
  }

  #[wasm_bindgen]
  pub fn create_instance(&self, ts_params: types::IAppParams) {
    let params: JsValue = ts_params.into();
    let mut container_id = String::from("canvas-container");
    let mut callback: Option<js_sys::Function> = None;

    if params.is_object() {
      if js_sys::Reflect::has(&params, &JsValue::from_str("containerId")).unwrap() {
        container_id = js_sys::Reflect::get(&params, &JsValue::from_str("containerId")).unwrap().as_string().unwrap();
      }

      if js_sys::Reflect::has(&params, &JsValue::from_str("callback")).unwrap() {
        callback = Some(js_sys::Reflect::get(&params, &JsValue::from_str("callback")).unwrap().dyn_into().unwrap());
      }
    }

    let _ = self.event_loop.send_event(UserEvents::CreateInstance((container_id, callback)));
  }
  
  #[wasm_bindgen]
  pub fn update_shader(&self, handle: &types::InstanceHandle, ts_params: types::IUpdateShaderParams) {
    let params = ts_params.into();
    let shader_source = js_sys::Reflect::get(&params, &JsValue::from_str("shaderSource")).unwrap().as_string().unwrap();
    let callback = js_sys::Reflect::get(&params, &JsValue::from_str("callback")).unwrap().dyn_into::<js_sys::Function>().unwrap();

    let _ = self.event_loop.send_event(UserEvents::UpdateShader((shader_source, callback, handle.window_id)));
  }

  #[wasm_bindgen]
  pub fn compile_shader(&self, handle: &types::InstanceHandle, ts_params: types::IUpdateShaderParams) {
    let params = ts_params.into();
    let shader_source = js_sys::Reflect::get(&params, &JsValue::from_str("shaderSource")).unwrap().as_string().unwrap();
    let callback = js_sys::Reflect::get(&params, &JsValue::from_str("callback")).unwrap().dyn_into::<js_sys::Function>().unwrap();

    let _ = self.event_loop.send_event(UserEvents::CompileShader((shader_source, callback, handle.window_id)));
  }

  #[wasm_bindgen]
  pub fn destroy_instance(&self, handle: types::InstanceHandle) {
    let _ = self.event_loop.send_event(UserEvents::DestroyInstance(handle.window_id));
  }
}

/*
#[wasm_bindgen]
impl App {
  #[cfg(not(target_arch = "wasm32"))]
  pub async fn new() -> Self {
    log::info!("[app] creating window");

    let event_loop = EventLoop::new().expect("[app] failed to create event loop");
    let window = Arc::new(WindowBuilder::new().build(&event_loop).expect("[app] failed to create window"));

    let gfx = Arc::new(Mutex::new(GfxState::new(window.clone()).await));

    let app = Self {
      window: Some(window.clone()),
      gfx: Some(gfx.clone()),
      container_id: String::from("canvas-container"),
    };

    event_loop.run(move |event, control_flow| {
      Self::event_handler(event, control_flow, window.clone(), gfx.clone());
    }).expect("[app] failed to run event loop");

    app
  }

  #[cfg(target_arch = "wasm32")]
  pub async fn new(ts_params: types::IAppParams) -> Self {
    log::info!("[app] creating window");

    let event_loop = EventLoop::new().expect("[app] failed to create event loop");
    let window = Arc::new(WindowBuilder::new().build(&event_loop).expect("[app] failed to create window"));

    use winit::platform::web::EventLoopExtWebSys;
    use winit::platform::web::WindowExtWebSys;

    let mut container_id = String::from("canvas-container");

    // create canvas on wasm
    {
      let params: JsValue = ts_params.into();

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
      window: Some(window.clone()),
      gfx: Some(gfx.clone()),
      container_id,
    };

    event_loop.spawn(move |event, control_flow| {
      Self::event_handler(event, control_flow, window.clone(), gfx.clone());
    });

    app
  }

  #[wasm_bindgen(js_name = "updateShader")]
  pub async fn update_shader(&self, shader_source: String) -> types::ShaderCompilationInfo {
    types::ShaderCompilationInfo::default()

    /*
    let mut gfx = match self.gfx.as_ref() {
      Some(gfx) => gfx.lock().expect("[app] failed to lock gfx"),
      None => return types::ShaderCompilationInfo::default(),
    };

    let result = gfx.compile_shader(&shader_source).await;
    if result.messages.is_empty() {
      gfx.update_shader(&shader_source);
    }
    result.into()
    */
  }

  #[wasm_bindgen(js_name = "compileShader")]
  pub async fn compile_shader(&self, shader_source: String) -> types::ShaderCompilationInfo {
    types::ShaderCompilationInfo::default()

    /*
    let gfx = match self.gfx.as_ref() {
      Some(gfx) => gfx.lock().expect("[app] failed to lock gfx"),
      None => return types::ShaderCompilationInfo::default(),
    };

    let result = gfx.compile_shader(&shader_source).await;
    result.into()
    */
  }

  #[wasm_bindgen]
  pub async fn destroy(&mut self) {
  }
}
*/
