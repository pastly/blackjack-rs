use bj_core::count::{StatefulHiLo, DECK_LEN};
use bj_core::deck::{Card, Deck};
use bj_web_core::card_char;
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
}

impl Default for State {
    fn default() -> Self {
        Self {
            deck: Deck::with_length(1),
            count: StatefulHiLo::new(1),
            total_cards: DECK_LEN,
            seen_cards: 0,
            cards_at_a_time: 1,
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
    if state.seen_cards >= state.total_cards {
        log("No next card");
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
