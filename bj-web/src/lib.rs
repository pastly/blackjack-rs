mod localstorage;

use bj_core::deck::{Card, Deck, Rank, Suit};
use bj_core::hand::Hand;
use bj_core::playstats::PlayStats;
use bj_core::rendertable::{HTMLTableRenderer, TableRenderer};
use bj_core::resp::{resps_from_buf, Resp};
use bj_core::table::Table;
use bj_core::utils::rand_next_hand;
use console_error_panic_hook;
use lazy_static::lazy_static;
use localstorage::LSVal;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

const LS_KEY_TABLE_PLAYSTATS: &str = "bj-table-playstats";
const LS_KEY_STREAK: &str = "bj-streak";
const LS_KEY_EXISTING_HAND: &str = "bj-hand";

lazy_static! {
    static ref TABLE: Mutex<Table<Resp>> =
        Mutex::new(Table::new(resps_from_buf(T1_TXT).unwrap()).unwrap());
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
H  H  H  H  H  H  H  H  H  H
H  H  H  H  H  H  H  H  H  H
H  H  H  H  H  H  H  H  H  H
H  H  H  H  H  H  H  H  H  H
H  Dh Dh Dh Dh H  H  H  H  H
Dh Dh Dh Dh Dh Dh Dh Dh H  H
Dh Dh Dh Dh Dh Dh Dh Dh Dh Dh
H  H  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
# soft hands: player value 13-21 (row) and dealer show 2-A (col)
H  H  H  Dh Dh H  H  H  H  H
H  H  H  Dh Dh H  H  H  H  H
H  H  Dh Dh Dh H  H  H  H  H
H  H  Dh Dh Dh H  H  H  H  H
H  Dh Dh Dh Dh H  H  H  H  H
Ds Ds Ds Ds Ds S  S  H  H  H
S  S  S  S  Ds S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
# pair hands: player value 4, 6, ... (row) and dealer show 2-A (col)
P  P  P  P  P  P  H  H  H  H
P  P  P  P  P  P  H  H  H  H
H  H  H  P  P  H  H  H  H  H
Dh Dh Dh Dh Dh Dh Dh Dh H  H
P  P  P  P  P  H  H  H  H  H
P  P  P  P  P  P  H  H  H  H
P  P  P  P  P  P  P  P  P  P
P  P  P  P  P  S  P  P  S  S
S  S  S  S  S  S  S  S  S  S
P  P  P  P  P  P  P  P  P  P
";

fn new_play_stats() -> Table<PlayStats> {
    Table::new(std::iter::repeat(PlayStats::new()).take(360)).unwrap()
}

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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let stats = LSVal::from_ls_or_default(LS_KEY_TABLE_PLAYSTATS, new_play_stats());
    let (player_hand, dealer_card) =
        &*LSVal::from_ls_or_default(LS_KEY_EXISTING_HAND, rand_next_hand(&stats));
    let stat_table = LSVal::from_ls_or_default(LS_KEY_TABLE_PLAYSTATS, new_play_stats());
    let streak = LSVal::from_ls_or_default(LS_KEY_STREAK, 0);
    output_hand(player_hand, *dealer_card);
    output_stats((player_hand, *dealer_card), &(*stat_table), *streak);
    output_resp_table();
    Ok(())
}

fn output_resp_table() {
    let t = Table::new(resps_from_buf(T1_TXT).unwrap()).unwrap();
    let mut fd: Vec<u8> = vec![];
    HTMLTableRenderer::render(&mut fd, t).unwrap();
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    doc.get_element_by_id("strat_html")
        .expect("should exist strat_html")
        .dyn_ref::<HtmlElement>()
        .expect("strat_html should be HtmlElement")
        .set_inner_html(&String::from_utf8(fd).unwrap());
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

fn output_stats(current_hand: (&Hand, Card), stat_table: &Table<PlayStats>, streak: u32) {
    let (correct, seen) = stat_table
        .values()
        .fold((0, 0), |(acc_correct, acc_seen), stat| {
            (acc_correct + stat.correct(), acc_seen + stat.seen())
        });
    set_stat(Stat::Correct, correct.into());
    set_stat(Stat::Seen, seen.into());
    set_stat(Stat::Streak, streak);
    let stat = stat_table.get(current_hand.0, current_hand.1).unwrap();
    set_stat(Stat::HandCorrect, stat.correct().into());
    set_stat(Stat::HandSeen, stat.seen().into());

    enum Stat {
        Correct,
        Seen,
        HandCorrect,
        HandSeen,
        Streak,
    }

    fn set_stat(stat: Stat, val: u32) {
        let win = web_sys::window().expect("should have a window in this context");
        let doc = win.document().expect("window should have a document");
        let id = match stat {
            Stat::Correct => "num_correct",
            Stat::Seen => "num_seen",
            Stat::HandCorrect => "hand_num_correct",
            Stat::HandSeen => "hand_num_seen",
            Stat::Streak => "num_streak",
        };
        doc.get_element_by_id(id)
            .expect("should exist stat")
            .dyn_ref::<HtmlElement>()
            .expect("stat should be HtmlElement")
            .set_inner_text(&val.to_string())
    }
}

fn update_stats(old_hand: (&Hand, Card), old_was_correct: bool) {
    let mut stat_table = LSVal::from_ls_or_default(LS_KEY_TABLE_PLAYSTATS, new_play_stats());
    let mut streak = LSVal::from_ls_or_default(LS_KEY_STREAK, 0);
    let mut old_stat = stat_table.get(&old_hand.0, old_hand.1).unwrap();
    old_stat.inc(old_was_correct);
    stat_table
        .update(&old_hand.0, old_hand.1, old_stat)
        .unwrap();
    *streak = if old_was_correct { *streak + 1 } else { 0 };
}

fn handle_button(resp: Resp) {
    // the (player_hand, dealer_card) currently on the screen
    let mut hand: LSVal<(Hand, Card)> = LSVal::from_ls(LS_KEY_EXISTING_HAND).unwrap();
    // the correct response to this (player_hand, dealer_card)
    let correct = TABLE.lock().unwrap().get(&hand.0, hand.1).unwrap();
    // grab a copy of what the user's existing streak is. If they get the hand wrong, we will want
    // to display this to them and we will soon be clearing out the localstorage copy of their
    // streak
    let old_streak = *LSVal::from_ls_or_default(LS_KEY_STREAK, 0);
    // update localstorage state for player statistics
    update_stats((&hand.0, hand.1), resp == correct);
    // display the "hint": player got it right, or they got it wrong and ___ is correct and ___ was
    // their streak
    set_hint(resp, correct, (&hand.0, hand.1), old_streak);
    // get a LSVal-wrapped ref to their stats so that we can (1) generate an appropriate next hand
    // for them and (2) tell them what their stats are for the new hand
    let stats: LSVal<Table<PlayStats>> = LSVal::from_ls(LS_KEY_TABLE_PLAYSTATS).unwrap();
    let _ = hand.swap(rand_next_hand(&*stats)); // (1)
    output_hand(&hand.0, hand.1);
    // update_stats() will have either incremented their streak or reset it to zero, so we need tor
    // refetch their streak from localstorage
    let new_streak = *LSVal::from_ls_or_default(LS_KEY_STREAK, 0);
    output_stats((&hand.0, hand.1), &(*stats), new_streak); // (2)

    fn set_hint(given: Resp, correct: Resp, hand: (&Hand, Card), streak: u32) {
        let win = web_sys::window().expect("should have a window in this context");
        let doc = win.document().expect("window should have a document");
        let s = if given == correct {
            format!("{} correct", given)
        } else {
            format!(
                "{} wrong. Should {} {} vs {}. Streak was {}",
                given, correct, hand.0, hand.1, streak
            )
        };
        doc.get_element_by_id("hint")
            .expect("should exist hint")
            .dyn_ref::<HtmlElement>()
            .expect("hint should be HtmlElement")
            .set_inner_text(&s)
    }
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
    // Double doesn't exist, so just do DoubleElseHit
    handle_button(Resp::DoubleElseHit)
}

#[wasm_bindgen]
pub fn on_button_split() {
    handle_button(Resp::Split)
}

#[wasm_bindgen]
pub fn on_button_clear_stats() {
    let mut stat_table = LSVal::from_ls_or_default(LS_KEY_TABLE_PLAYSTATS, new_play_stats());
    let mut streak = LSVal::from_ls_or_default(LS_KEY_STREAK, 0);
    for v in stat_table.values_mut() {
        *v = PlayStats::new();
    }
    *streak = 0;
    let (player, dealer) = &*LSVal::from_ls(LS_KEY_EXISTING_HAND).unwrap();
    output_stats((&player, *dealer), &(*stat_table), *streak);
}
