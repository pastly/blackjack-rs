use bj_core::basicstrategy::BasicStrategy;
use bj_core::hand::HandType;
use bj_core::rendertable::{HTMLTableRenderer, HTMLTableRendererOpts};
use bj_core::resp::Resp;
use bj_core::table::{dealer_card_from_desc, player_hand_from_desc, GameDesc};
use bj_web_core::bs_data;
use bj_web_core::localstorage::{lskeys, LSVal};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

const LS_KEY_SELECTED_RESP: &str = "bj-custom-card-selected-resp";
const USE_SESSION_STORAGE: bool = false;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn resp_from_str(s: &str) -> Option<Resp> {
    match s {
        "H" => Some(Resp::Hit),
        "S" => Some(Resp::Stand),
        "Dh" => Some(Resp::DoubleElseHit),
        "Ds" => Some(Resp::DoubleElseStand),
        "P" => Some(Resp::Split),
        "Rh" => Some(Resp::SurrenderElseHit),
        "Rs" => Some(Resp::SurrenderElseStand),
        "Rp" => Some(Resp::SurrenderElseSplit),
        _ => None,
    }
}

fn render_bs_card(bs: &BasicStrategy) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let mut buf = vec![];
    let opts = HTMLTableRendererOpts {
        incl_bs_rules: false,
        cell_onclick_cb: Some("onclick_cell".to_string()),
    };
    HTMLTableRenderer::render(&mut buf, bs, opts).unwrap();
    doc.get_element_by_id("bscard")
        .expect("should have bscard")
        .dyn_ref::<HtmlElement>()
        .expect("bscard should be HtmlElement")
        .set_inner_html(&String::from_utf8(buf).unwrap());
}

fn set_border_selected_resp(resp: Resp) {
    let cell_idx = match resp {
        Resp::Hit => 0,
        Resp::Stand => 1,
        Resp::DoubleElseHit => 2,
        Resp::DoubleElseStand => 3,
        Resp::Split => 4,
        Resp::SurrenderElseHit => 5,
        Resp::SurrenderElseStand => 6,
        Resp::SurrenderElseSplit => 7,
    };
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let cells = doc
        .get_element_by_id("cell_color_opts")
        .expect("shoudl have cell_color_opts")
        .dyn_ref::<Element>()
        .expect("cell_color_opts should be HtmlElement")
        .get_elements_by_tag_name("td");
    for i in 0..cells.length() {
        let class_list = cells
            .item(i)
            .unwrap()
            .dyn_ref::<Element>()
            .expect("cell should be Element")
            .class_list();
        if i == cell_idx {
            class_list
                .add_1("selected")
                .expect("unable to add selected class");
        } else {
            class_list
                .remove_1("selected")
                .expect("unable to remove selected class");
        }
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let def: BasicStrategy = serde_json::from_reader(bs_data::T1_JSON).unwrap();
    let bs: LSVal<BasicStrategy> =
        LSVal::from_ls_or_default(USE_SESSION_STORAGE, lskeys::LS_KEY_BS_CARD, def);
    {
        let mut resp = LSVal::from_ls_or_default(USE_SESSION_STORAGE, LS_KEY_SELECTED_RESP, None);
        *resp = Some(Resp::Hit);
        set_border_selected_resp(resp.unwrap());
    }
    render_bs_card(&*bs);
    Ok(())
}

#[wasm_bindgen]
pub fn onclick_cell(tbl: &str, mut player: u8, dealer: u8) {
    let resp: LSVal<Option<Resp>> =
        LSVal::from_ls_or_default(USE_SESSION_STORAGE, LS_KEY_SELECTED_RESP, None);
    if resp.is_none() {
        log("Should have had a selected response at this point, but don't");
        return;
    }
    let new = resp.unwrap();
    let (key_player, key_dealer) = {
        let hand_type = match tbl {
            "hard" => HandType::Hard,
            "soft" => HandType::Soft,
            "pair" => HandType::Pair,
            _ => panic!(format!("Impossible hand type {}", tbl)),
        };
        if tbl == "pair" {
            player *= 2;
        }
        let desc = GameDesc {
            hand: hand_type,
            player,
            dealer,
        };
        (
            player_hand_from_desc(desc).unwrap(),
            dealer_card_from_desc(desc).unwrap(),
        )
    };
    let mut bs = {
        let def: BasicStrategy = serde_json::from_reader(bs_data::T1_JSON).unwrap();
        LSVal::from_ls_or_default(USE_SESSION_STORAGE, lskeys::LS_KEY_BS_CARD, def)
    };
    let old = bs.table.get(&key_player, key_dealer).unwrap();
    log(&format!(
        "Changing {} {}/{} from {} to {}",
        tbl, player, dealer, old, new
    ));
    bs.table.update(&key_player, key_dealer, new).unwrap();
    render_bs_card(&*bs);
}

#[wasm_bindgen]
pub fn onclick_select_resp(resp_str: &str) {
    let mut stored: LSVal<Option<Resp>> =
        LSVal::from_ls_or_default(USE_SESSION_STORAGE, LS_KEY_SELECTED_RESP, None);
    let new = Some(resp_from_str(resp_str).unwrap());
    log(&format!(
        "Changing selected resp from {:?} to {:?}",
        *stored, new,
    ));
    *stored = new;
    set_border_selected_resp(new.unwrap());
}
