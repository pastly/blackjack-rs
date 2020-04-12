use bj_core::basicstrategy::BasicStrategy;
use bj_core::rendertable::{HTMLTableRenderer, HTMLTableRendererOpts};
use bj_web_core::bs_data;
use bj_web_core::localstorage::{lskeys, LSVal};
use console_error_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn render_bs_card(bs: &BasicStrategy) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let mut buf = vec![];
    let opts = HTMLTableRendererOpts {
        incl_bs_rules: false,
    };
    HTMLTableRenderer::render(&mut buf, bs, opts).unwrap();
    doc.get_element_by_id("bscard")
        .expect("should have bscard")
        .dyn_ref::<HtmlElement>()
        .expect("bscard should be HtmlElement")
        .set_inner_html(&String::from_utf8(buf).unwrap());
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let def: BasicStrategy = serde_json::from_reader(bs_data::T1_JSON).unwrap();
    let bs: LSVal<BasicStrategy> = LSVal::from_ls_or_default(lskeys::LS_KEY_BS_CARD, def);
    log("Helo there");
    log(lskeys::LS_KEY_BS_CARD);
    log(&format!("{:?}", *bs));
    render_bs_card(&*bs);
    Ok(())
}
