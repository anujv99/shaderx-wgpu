
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const CustomTypes: &'static str = r#"
interface IAppParams {
  containerId: string;
}

interface ICompilationMessage {
  message: string;
  type: "error" | "warning" | "info";
  location?: {
    lineNumber: number;
    linePosition: number;
    offset: number;
    length: number;
  };
}

type TShaderCompilationInfoIteractorCallback = (message: ICompilationMessage) => void;
"#;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(typescript_type = "IAppParams")]
  pub type IAppParams;
  #[wasm_bindgen(typescript_type = "TShaderCompilationInfoIteractorCallback")]
  pub type TShaderCompilationInfoIteractorCallback;
}

#[wasm_bindgen]
pub struct ShaderCompilationInfo {
  messages: Vec<wgpu::CompilationMessage>,
}

#[wasm_bindgen]
impl ShaderCompilationInfo {
  #[wasm_bindgen(js_name = forEach)]
  pub fn for_each(&self, callback: TShaderCompilationInfoIteractorCallback) {
    let callback_js: JsValue = callback.into();
    if callback_js.is_undefined() || !callback_js.is_function() {
      return;
    }
    let callback_fn = js_sys::Function::from(callback_js);
    self.messages.iter().for_each(|message| {
      let js_message = compilation_message_to_js_value(message);
      callback_fn.call1(&JsValue::NULL, &js_message).unwrap();
    });
  }
  
  #[wasm_bindgen(js_name = isEmpty)]
  pub fn is_empty(&self) -> bool {
    self.messages.is_empty()
  }
}

impl From<wgpu::CompilationInfo> for ShaderCompilationInfo {
  fn from(info: wgpu::CompilationInfo) -> Self {
    ShaderCompilationInfo {
      messages: info.messages,
    }
  }
}

fn compilation_message_to_js_value(message: &wgpu::CompilationMessage) -> JsValue {
  let obj = js_sys::Object::new();
  let message_type = match message.message_type {
    wgpu::CompilationMessageType::Error => "error",
    wgpu::CompilationMessageType::Warning => "warning",
    wgpu::CompilationMessageType::Info => "info",
  };
  js_sys::Reflect::set(&obj, &JsValue::from_str("message"), &JsValue::from_str(&message.message)).unwrap();
  js_sys::Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str(&message_type)).unwrap();
  if message.location.is_some() {
    let loc = js_sys::Object::new();
    js_sys::Reflect::set(&loc, &JsValue::from_str("lineNumber"), &JsValue::from_f64(message.location.unwrap().line_number as f64)).unwrap();
    js_sys::Reflect::set(&loc, &JsValue::from_str("linePosition"), &JsValue::from_f64(message.location.unwrap().line_position as f64)).unwrap();
    js_sys::Reflect::set(&loc, &JsValue::from_str("offset"), &JsValue::from_f64(message.location.unwrap().offset as f64)).unwrap();
    js_sys::Reflect::set(&loc, &JsValue::from_str("length"), &JsValue::from_f64(message.location.unwrap().length as f64)).unwrap();
    js_sys::Reflect::set(&obj, &JsValue::from_str("location"), &loc).unwrap();
  }
  obj.into()
}

