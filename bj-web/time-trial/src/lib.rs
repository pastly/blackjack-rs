mod handresult;

use bj_core::basicstrategy::{rules, BasicStrategy};
use bj_core::deck::{Card, Rank};
use bj_core::hand::Hand;
use bj_core::rendertable::{HTMLTableRenderer, HTMLTableRendererOpts};
use bj_core::resp::Resp;
use bj_core::utils::uniform_rand_2card_hand;
use bj_web_core::bs_data;
use bj_web_core::button::GameButton;
use bj_web_core::card_char;
use bj_web_core::correct_resp::is_correct_resp_button;
use bj_web_core::localstorage::{lskeys, LSVal};
use handresult::HandResult;
use js_sys::Date;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
#[macro_use]
extern crate lazy_static;

#[derive(Debug)]
struct State {
    use_session_storage: bool,
    // results storage, obviously
    results: Vec<HandResult>,
    // stop when results.len() is this
    num_hands: usize,
    // timestamp (in seconds, not ms) of first result
    start_time: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            use_session_storage: true,
            // resized to num_hands capacity in rust_init()
            results: vec![],
            // to be updated on rust_init()
            num_hands: 0,
            // to be updated on first result
            start_time: 0.0,
        }
    }
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(Default::default());
}

fn def_bs_card() -> BasicStrategy {
    serde_json::from_reader(bs_data::T1_JSON).unwrap()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    fn flash_hint_message(s: &str);
    fn set_hint_message(s: &str);
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

#[wasm_bindgen]
pub fn rust_init(num_hands: usize) {
    let mut state = STATE.lock().unwrap();
    {
        state.num_hands = num_hands;
        let cap = state.results.capacity();
        let len = state.results.len();
        if cap < num_hands {
            state.results.reserve(num_hands - len);
        }
        assert!(state.results.capacity() >= num_hands);
    }
    output_resp_table(&*state);
    let hand = &*LSVal::from_ls_or_default(
        state.use_session_storage,
        lskeys::LS_KEY_EXISTING_HAND,
        uniform_rand_2card_hand(),
    );
    output_hand(&hand.0, hand.1);
}

fn output_resp_table(state: &State) {
    let bs_card = LSVal::from_ls_or_default(
        state.use_session_storage,
        lskeys::LS_KEY_BS_CARD,
        def_bs_card(),
    );
    let mut fd: Vec<u8> = vec![];
    let opts = HTMLTableRendererOpts {
        incl_bs_rules: true,
        cell_onclick_cb: None,
    };
    HTMLTableRenderer::render(&mut fd, &*bs_card, opts).unwrap();
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    doc.get_element_by_id("strat_html")
        .expect("should exist strat_html")
        .dyn_ref::<HtmlElement>()
        .expect("strat_html should be HtmlElement")
        .set_inner_html(&String::from_utf8(fd).unwrap());
}

fn is_legal_resp(btn: GameButton, hand: (&Hand, Card), surrender_rule: rules::Surrender) -> bool {
    let (player, dealer) = hand;
    match btn {
        GameButton::Hit | GameButton::Stand => true,
        GameButton::Double => player.can_double(),
        GameButton::Split => player.can_split(),
        GameButton::Surrender => player.can_surrender(surrender_rule, dealer),
    }
}

#[wasm_bindgen]
pub fn on_button_hit() {
    let mut state = STATE.lock().unwrap();
    handle_button(&mut *state, GameButton::Hit);
}

#[wasm_bindgen]
pub fn on_button_stand() {
    let mut state = STATE.lock().unwrap();
    handle_button(&mut *state, GameButton::Stand);
}

#[wasm_bindgen]
pub fn on_button_double() {
    let mut state = STATE.lock().unwrap();
    handle_button(&mut *state, GameButton::Double);
}

#[wasm_bindgen]
pub fn on_button_split() {
    let mut state = STATE.lock().unwrap();
    handle_button(&mut *state, GameButton::Split);
}

#[wasm_bindgen]
pub fn on_button_surrender() {
    let mut state = STATE.lock().unwrap();
    handle_button(&mut *state, GameButton::Surrender);
}

fn handle_button(state: &mut State, btn: GameButton) {
    // don't do anything if game over
    if state.num_hands <= state.results.len() {
        return;
    }
    // the (player_hand, dealer_card) currently on the screen
    let mut hand: LSVal<(Hand, Card)> =
        LSVal::from_ls(state.use_session_storage, lskeys::LS_KEY_EXISTING_HAND).unwrap();
    let bs_card = LSVal::from_ls_or_default(
        state.use_session_storage,
        lskeys::LS_KEY_BS_CARD,
        def_bs_card(),
    );
    let surrender_rule = match &bs_card.rules {
        None => rules::Surrender::Yes,
        Some(rules) => rules.surrender,
    };
    // return early if user didn't even give legal response to this hand
    if !is_legal_resp(btn, (&hand.0, hand.1), surrender_rule) {
        log(&format!(
            "{} is not a legal response to {}/{}",
            btn, &hand.0, hand.1
        ));
        return;
    }
    let now = Date::now() / 1000.0; // convert fro ms to s
    if state.results.is_empty() {
        state.start_time = now;
    }
    // the correct response to this (player_hand, dealer_card). We store the bool is_correct as
    // well because whether or not the response is correct is more complex than button == resp: if
    // the correct Resp is DoubleElseHit (or its cousins) then it is not enough to simply check if
    // the Double button was pressed.
    let correct: Resp = bs_card.table.get(&hand.0, hand.1).unwrap();
    let is_correct = is_correct_resp_button(btn, correct, (&hand.0, hand.1), surrender_rule);
    // store the result for this hand
    state.results.push(HandResult {
        player: hand.0.clone(),
        dealer: hand.1,
        correct: is_correct,
        time: now - state.start_time,
    });
    set_hint(
        btn,
        correct,
        (&hand.0, hand.1),
        is_correct,
        state.num_hands - state.results.len(),
    );
    // generate a new hand
    let _ = hand.swap(uniform_rand_2card_hand());
    output_hand(&hand.0, hand.1);
    update_buttons((&hand.0, hand.1), &bs_card.rules);
    // consider ending the game
    if state.results.len() == state.num_hands {
        // game over
        assert!(!state.results.is_empty());
        let dur = state.results[state.results.len() - 1].time;
        let num_correct = state
            .results
            .iter()
            .fold(0, |acc, res| acc + if res.correct { 1 } else { 0 });
        set_hint_message(&format!(
            "Done! Did {}/{} hands correctly in {} seconds",
            num_correct, state.num_hands, dur,
        ));
        set_hint_message(&handresult::to_string(&state.results));
    }
}

fn output_hand(player: &Hand, dealer: Card) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    doc.get_element_by_id("player_cards")
        .expect("should exist player_cards")
        .dyn_ref::<HtmlElement>()
        .expect("player_cards should be HtmlElement")
        .set_inner_text(&player.cards().map(|&c| card_char(c)).collect::<String>());
    doc.get_element_by_id("dealer_cards")
        .expect("should exist dealer_cards")
        .dyn_ref::<HtmlElement>()
        .expect("dealer_cards should be HtmlElement")
        .set_inner_text(&format!("{}", card_char(dealer)));
}

fn update_buttons(hand: (&Hand, Card), rules: &Option<rules::Rules>) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let hit_enabled = true;
    let stand_enabled = true;
    let double_enabled = hand.0.can_double();
    let split_enabled = hand.0.can_split();
    let surrender_enabled = if let Some(rules) = rules {
        match rules.surrender {
            rules::Surrender::No => false,
            rules::Surrender::Yes => true,
            rules::Surrender::NotAce => hand.1.rank() != Rank::RA,
        }
    } else {
        true
    };
    for (id, enabled) in [
        ("button_hit", hit_enabled),
        ("button_stand", stand_enabled),
        ("button_double", double_enabled),
        ("button_split", split_enabled),
        ("button_surrender", surrender_enabled),
    ]
    .iter()
    {
        let class_list = doc
            .get_element_by_id(id)
            .expect("should exist button")
            .dyn_ref::<Element>()
            .expect("button should be Element")
            .class_list();
        if *enabled {
            class_list
                .remove_1("hide")
                .expect("Unable to add hide class");
        } else {
            class_list
                .add_1("hide")
                .expect("Unable to remove hide class");
        }
    }
}

fn set_hint(
    given: GameButton,
    correct: Resp,
    hand: (&Hand, Card),
    is_correct: bool,
    remaining: usize,
) {
    let s = if is_correct {
        format!(
            "{} correct. {} hand{} to go.",
            given,
            remaining,
            if remaining == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "{} wrong. Should {} {} vs {}. {} hand{} to go.",
            given,
            correct,
            hand.0,
            hand.1,
            remaining,
            if remaining == 1 { "" } else { "s" }
        )
    };
    flash_hint_message(&s);
}

#[wasm_bindgen]
pub fn results_from_state() -> String {
    let state = STATE.lock().unwrap();
    handresult::to_string(&state.results)
}
