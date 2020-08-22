use bj_core::count::{StatefulHiLo, DECK_LEN};
use bj_core::deck::{Card, Deck};
use bj_web_core::card_char;
use bj_web_core::localstorage::{lskeys, LSVal};
use js_sys::Date;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
#[macro_use]
extern crate lazy_static;

struct State {
    deck: Deck,
    count: StatefulHiLo,
    /// num cards the user asked to see
    total_cards: u16,
    /// num cards the user has seen so far
    seen_cards: u16,
    /// num cards to show at the user at once
    cards_at_a_time: u8,
    /// timestamp (in s, not ms) of when we present the first card
    start_time: f64,
    /// timestamp (in s, not ms) of when we first get asked for another card, but there are no more
    /// cards
    end_time: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            // reset in rust_init()
            deck: Deck::with_length(1),
            // reset in rust_init()
            count: StatefulHiLo::new(1),
            // reset in rust_init()
            total_cards: DECK_LEN,
            // reset in rust_init()
            seen_cards: 0,
            // reset in rust_init()
            cards_at_a_time: 1,
            // updated when we show first card
            start_time: 0.0,
            // updated when we cannot show any more cards
            end_time: 0.0,
        }
    }
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(Default::default());
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

/// Initialize our state.
///
/// num_decks is the, well, number of decks of cards we will generate.  num_cards is the number of
/// cards we will draw from those decks in total, which allows the user to train on less than a
/// whole number of decks, thus ending on a non-zero count (assuming HiLo).
///
/// returns false if there was a problem initing (e.g. impossible request), otherwise true.
#[wasm_bindgen]
pub fn rust_init(num_decks: u8, num_cards: u16, cards_at_a_time: u8) -> bool {
    if num_decks as u16 * DECK_LEN < num_cards {
        log(&format!(
            "{} decks of cards have less than {} cards",
            num_decks, num_cards
        ));
        return false;
    }
    let mut state = STATE.lock().unwrap();
    state.deck = Deck::with_length(num_decks as usize);
    state.count = StatefulHiLo::new(num_decks);
    state.total_cards = num_cards;
    state.seen_cards = 0;
    state.cards_at_a_time = cards_at_a_time;
    state.start_time = 0.0;
    state.end_time = 0.0;
    log(&format!(
        "Init rust state with {} decks and showing {} cards {} at a time",
        num_decks, num_cards, cards_at_a_time
    ));
    true
}

fn output_cards(cards: &[Card]) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    doc.get_element_by_id("cards")
        .expect("should exist cards")
        .dyn_ref::<HtmlElement>()
        .expect("cards should be HtmlElement")
        .set_inner_text(&cards.iter().map(|&c| card_char(c)).collect::<String>());
}

/// Returns true if there was a next card to display, otherwise false
#[wasm_bindgen]
pub fn display_next_card() -> bool {
    let mut state = STATE.lock().unwrap();
    let now = Date::now() / 1000.0; // convert from ms to s
    if state.seen_cards == 0 {
        state.start_time = now;
    }
    if state.seen_cards >= state.total_cards {
        log("No next card");
        if state.end_time == 0.0 {
            state.end_time = now;
        }
        return false;
    }
    let cards = {
        let mut v = vec![];
        while v.len() < state.cards_at_a_time as usize && state.seen_cards < state.total_cards {
            v.push(state.deck.draw().unwrap());
            state.seen_cards += 1;
        }
        v
    };
    state.count.update_many(&cards);
    log(&format!(
        "Next cards are {:?} (count: {})",
        //cards.iter().map(|&c| card_char(c)).collect::<String>(),
        &cards,
        state.count.running_count(),
    ));
    output_cards(&cards);
    true
}

/// Returns the current count
#[wasm_bindgen]
pub fn current_count() -> i16 {
    STATE.lock().unwrap().count.running_count()
}

/// Returns how long it took the player to have us display all cards, or something less than 0 if
/// not all cards have been shown yet
#[wasm_bindgen]
pub fn game_duration() -> f64 {
    // If the game isn't over yet, then end_time will be 0.0 and this will end up negative, which
    // handles the error case by itself
    let state = STATE.lock().unwrap();
    let dur = state.end_time - state.start_time;
    log(&format!("Duration was: {}", dur));
    dur
}

/// Store the given json string representing the preferences for the counting training module in
/// local storage. No verification is done to ensure the string is valid preferences, let alone
/// valid json.
#[wasm_bindgen]
pub fn set_ls_preferences(s: String) {
    let mut prefs =
        LSVal::from_ls_or_default(false, lskeys::LS_KEY_COUNTING_PREFS, "{}".to_string());
    let _ = prefs.swap(s);
}

/// Get the preferences currently in local storage, or just "{}" if none.
#[wasm_bindgen]
pub fn get_ls_preferences() -> String {
    LSVal::from_ls_or_default(false, lskeys::LS_KEY_COUNTING_PREFS, "{}".to_string()).clone()
}
