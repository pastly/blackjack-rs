use bj_core::deck::{Card, Deck, Rank, Suit};
use bj_core::hand::Hand;
use bj_core::table::{resps_from_buf, Resp, Table};
use console_error_panic_hook;
use lazy_static::lazy_static;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

lazy_static! {
    static ref TABLE: Mutex<Table<Resp>> = Mutex::new(Table::new(resps_from_buf(T1_TXT)).unwrap());
}
lazy_static! {
    static ref DECK: Mutex<Deck> = Mutex::new(Deck::new_infinite());
}

const T1_TXT: &[u8] = b"
## Table
##     Decks: 4+
##     Soft 17: dealer hit
##     Double after split: allowed
##     Surrender: not allowed
##     Dealer peek for BJ: yes
## https://wizardofodds.com/games/blackjack/strategy/calculator/
# hard hands: player value 5-21 (row) and dealer show 2-A (col)
HHHHHHHHHH
HHHHHHHHHH
HHHHHHHHHH
HHHHHHHHHH
HDDDDHHHHH
DDDDDDDDHH
DDDDDDDDDD
HHSSSHHHHH
SSSSSHHHHH
SSSSSHHHHH
SSSSSHHHHH
SSSSSHHHHH
SSSSSSSSSS
SSSSSSSSSS
SSSSSSSSSS
SSSSSSSSSS
SSSSSSSSSS
# soft hands: player value 13-21 (row) and dealer show 2-A (col)
HHHDDHHHHH
HHHDDHHHHH
HHDDDHHHHH
HHDDDHHHHH
HDDDDHHHHH
DDDDDSSHHH
SSSSDSSSSS
SSSSSSSSSS
SSSSSSSSSS
# pair hands: player value 4, 6, ... (row) and dealer show 2-A (col)
PPPPPPHHHH
PPPPPPHHHH
HHHPPHHHHH
DDDDDDDDHH
PPPPPHHHHH
PPPPPPHHHH
PPPPPPPPPP
PPPPPSPPSS
SSSSSSSSSS
PPPPPPPPPP
";

fn card_char(card: Card) -> char {
    // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode#Block
    let base: u32 = match card.suit() {
        Suit::Spade => 0x1F0A0,
        Suit::Heart => 0x1F0B0,
        Suit::Diamond => 0x1F0C0,
        Suit::Club => 0x1F0D0,
    };
    let val = base
        + match card.rank() {
            Rank::RA => 1,
            Rank::R2 => 2,
            Rank::R3 => 3,
            Rank::R4 => 4,
            Rank::R5 => 5,
            Rank::R6 => 6,
            Rank::R7 => 7,
            Rank::R8 => 8,
            Rank::R9 => 9,
            Rank::RT => 10,
            Rank::RJ => 11,
            // Unicode includes Knight here. Weird. Skip 12.
            Rank::RQ => 13,
            Rank::RK => 14,
        };
    // Safety: Value will always be a valid char thanks to match statements and enums on card
    // suits and ranks.
    unsafe { std::char::from_u32_unchecked(val) }
}

fn char_card(ch: char) -> Option<Card> {
    let suit = match ch as u32 {
        0x1F0A1..=0x1F0AE => Suit::Spade,
        0x1F0B1..=0x1F0BE => Suit::Heart,
        0x1F0C1..=0x1F0CE => Suit::Diamond,
        0x1F0D1..=0x1F0DE => Suit::Club,
        _ => panic!("Cannot determine suit for card character"),
    };
    let rank = match ((ch as u32) - 0x1F0A0) % 16 {
        1 => Rank::RA,
        2 => Rank::R2,
        3 => Rank::R3,
        4 => Rank::R4,
        5 => Rank::R5,
        6 => Rank::R6,
        7 => Rank::R7,
        8 => Rank::R8,
        9 => Rank::R9,
        10 => Rank::RT,
        11 => Rank::RJ,
        13 => Rank::RQ,
        14 => Rank::RK,
        0 | 12 | 15 | _ => panic!("Cannot determine rank for card character"),
    };
    Some(Card::new(rank, suit))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    output_new_hand();
    Ok(())
}

enum Stat {
    Correct,
    Seen,
    Streak,
}

fn get_stat(stat: Stat) -> u32 {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let id = match stat {
        Stat::Correct => "num_correct",
        Stat::Seen => "num_seen",
        Stat::Streak => "num_streak",
    };
    doc.get_element_by_id(id)
        .expect("should exist stat")
        .dyn_ref::<HtmlElement>()
        .expect("stat should be HtmlElement")
        .inner_text()
        .parse::<u32>()
        .unwrap()
}

fn set_stat(stat: Stat, val: u32) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let id = match stat {
        Stat::Correct => "num_correct",
        Stat::Seen => "num_seen",
        Stat::Streak => "num_streak",
    };
    doc.get_element_by_id(id)
        .expect("should exist stat")
        .dyn_ref::<HtmlElement>()
        .expect("stat should be HtmlElement")
        .set_inner_text(&val.to_string())
}

fn set_hint(given: Resp, correct: Resp, streak: u32) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let s = if given == correct {
        format!("{} correct", given)
    } else {
        format!("{} wrong. Should {}. Streak was {}", given, correct, streak)
    };
    doc.get_element_by_id("hint")
        .expect("should exist hint")
        .dyn_ref::<HtmlElement>()
        .expect("hint should be HtmlElement")
        .set_inner_text(&s)
}

fn output_new_hand() {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let mut deck = DECK.lock().unwrap();
    //let table = TABLE.lock().unwrap();
    let player = Hand::new(&[deck.draw().unwrap(), deck.draw().unwrap()]);
    let dealer = deck.draw().unwrap();
    doc.get_element_by_id("player_cards")
        .expect("should exist player_cards")
        .dyn_ref::<HtmlElement>()
        .expect("player_cards should be HtmlElement")
        .set_inner_text(&player.into_cards().map(card_char).collect::<String>());
    doc.get_element_by_id("dealer_cards")
        .expect("should exist dealer_cards")
        .dyn_ref::<HtmlElement>()
        .expect("dealer_cards should be HtmlElement")
        .set_inner_text(&format!("{}", card_char(dealer)));
}

fn existing_cards(is_player: bool) -> Vec<Card> {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let id = if is_player {
        "player_cards"
    } else {
        "dealer_cards"
    };
    doc.get_element_by_id(id)
        .expect("should exist player_cards/dealer_cards")
        .dyn_ref::<HtmlElement>()
        .expect("player_cards/dealer_cards should be HtmlElement")
        .inner_text()
        .chars()
        .map(|c| char_card(c).unwrap())
        .collect()
}

fn handle_button(resp: Resp) {
    //log(&format!("got a {}", resp));
    let player_cards = existing_cards(true);
    let dealer_cards = existing_cards(false);
    //log(&format!("player had {:?}", player_cards));
    //log(&format!("dealer had {:?}", dealer_cards));
    assert_eq!(player_cards.len(), 2);
    assert_eq!(dealer_cards.len(), 1);
    let player = Hand::new(&player_cards);
    let dealer = dealer_cards[0];
    let table = TABLE.lock().unwrap();
    let correct = table.get(&player, dealer).unwrap();
    //log(&format!("correct is {}", correct));
    set_hint(resp, correct, get_stat(Stat::Streak));
    if resp == correct {
        set_stat(Stat::Correct, get_stat(Stat::Correct) + 1);
        set_stat(Stat::Streak, get_stat(Stat::Streak) + 1);
    } else {
        set_stat(Stat::Streak, 0);
    }
    set_stat(Stat::Seen, get_stat(Stat::Seen) + 1);
    output_new_hand();
}

#[wasm_bindgen]
pub fn on_button_hit() {
    handle_button(Resp::Hit)
}

#[wasm_bindgen]
pub fn on_button_stand() {
    handle_button(Resp::Stand)
}

#[wasm_bindgen]
pub fn on_button_double() {
    handle_button(Resp::Double)
}

#[wasm_bindgen]
pub fn on_button_split() {
    handle_button(Resp::Split)
}
