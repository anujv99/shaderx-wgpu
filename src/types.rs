
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IAppParams: &'static str = r#"
interface IAppParams {
  containerId: string;
}
"#;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(typescript_type = "IAppParams")]
  pub type IAppParams;
}
