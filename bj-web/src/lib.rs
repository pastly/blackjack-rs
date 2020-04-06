mod bs_data;
mod button;
mod localstorage;
mod correct_resp;

use bj_core::basicstrategy::rules;
use bj_core::basicstrategy::BasicStrategy;
use bj_core::deck::{Card, Deck, Rank, Suit};
use bj_core::hand::Hand;
use bj_core::playstats::PlayStats;
use bj_core::rendertable::{HTMLTableRenderer, TableRenderer};
use bj_core::resp::Resp;
use bj_core::table::Table;
use bj_core::utils::rand_next_hand;
use button::Button;
use console_error_panic_hook;
use correct_resp::is_correct_resp_button;
use lazy_static::lazy_static;
use localstorage::LSVal;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

const LS_KEY_TABLE_PLAYSTATS: &str = "bj-table-playstats";
const LS_KEY_STREAK: &str = "bj-streak";
const LS_KEY_EXISTING_HAND: &str = "bj-hand";

lazy_static! {
    static ref BS_CARD: Mutex<BasicStrategy> =
        Mutex::new(serde_json::from_reader(bs_data::T1_JSON).unwrap());
}
lazy_static! {
    static ref DECK: Mutex<Deck> = Mutex::new(Deck::new_infinite());
}

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
    {
        let bs_card = &*BS_CARD.lock().unwrap();
        update_buttons((player_hand, *dealer_card), &bs_card.rules);
    }
    output_stats((player_hand, *dealer_card), &(*stat_table), *streak);
    output_resp_table();
    Ok(())
}

fn output_resp_table() {
    //let t = Table::new(resps_from_buf(T1_TXT).unwrap()).unwrap();
    let bs_card = &*BS_CARD.lock().unwrap();
    let mut fd: Vec<u8> = vec![];
    HTMLTableRenderer::render(&mut fd, bs_card).unwrap();
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
    set_stat(Stat::Correct, correct);
    set_stat(Stat::Seen, seen);
    set_stat(Stat::Streak, streak);
    let stat = stat_table.get(current_hand.0, current_hand.1).unwrap();
    set_stat(Stat::HandCorrect, stat.correct());
    set_stat(Stat::HandSeen, stat.seen());

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

fn update_buttons(hand: (&Hand, Card), rules: &rules::Rules) {
    let win = web_sys::window().expect("should have a window in this context");
    let doc = win.document().expect("window should have a document");
    let hit_enabled = true;
    let stand_enabled = true;
    let double_enabled = hand.0.can_double();
    let split_enabled = hand.0.can_split();
    let surrender_enabled = match rules.surrender {
        rules::Surrender::No => false,
        rules::Surrender::Yes => true,
        rules::Surrender::NotAce => hand.1.rank() != Rank::RA,
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

fn handle_button(btn: Button) {
    // the (player_hand, dealer_card) currently on the screen
    let mut hand: LSVal<(Hand, Card)> = LSVal::from_ls(LS_KEY_EXISTING_HAND).unwrap();
    // the correct response to this (player_hand, dealer_card). We store the bool is_correct as
    // well because whether or not the response is correct is more complex than button == resp: if
    // the correct Resp is DoubleElseHit (or its cousins) then it is not enough to simply check if
    // the Double button was pressed.
    let bs_card = BS_CARD.lock().unwrap();
    let correct: Resp = bs_card.table.get(&hand.0, hand.1).unwrap();
    let is_correct = is_correct_resp_button(&btn, correct, (&hand.0, hand.1));
    // grab a copy of what the user's existing streak is. If they get the hand wrong, we will want
    // to display this to them and we will soon be clearing out the localstorage copy of their
    // streak
    let old_streak = *LSVal::from_ls_or_default(LS_KEY_STREAK, 0);
    // update localstorage state for player statistics
    update_stats((&hand.0, hand.1), is_correct);
    // display the "hint": player got it right, or they got it wrong and ___ is correct and ___ was
    // their streak
    set_hint(&btn, correct, (&hand.0, hand.1), is_correct, old_streak);
    // get a LSVal-wrapped ref to their stats so that we can (1) generate an appropriate next hand
    // for them and (2) tell them what their stats are for the new hand
    let stats: LSVal<Table<PlayStats>> = LSVal::from_ls(LS_KEY_TABLE_PLAYSTATS).unwrap();
    let _ = hand.swap(rand_next_hand(&*stats)); // (1)
    output_hand(&hand.0, hand.1);
    update_buttons((&hand.0, hand.1), &bs_card.rules);
    // update_stats() will have either incremented their streak or reset it to zero, so we need tor
    // refetch their streak from localstorage
    let new_streak = *LSVal::from_ls_or_default(LS_KEY_STREAK, 0);
    output_stats((&hand.0, hand.1), &(*stats), new_streak); // (2)

    fn set_hint(given: &Button, correct: Resp, hand: (&Hand, Card), is_correct: bool, streak: u32) {
        let win = web_sys::window().expect("should have a window in this context");
        let doc = win.document().expect("window should have a document");
        let s = if is_correct {
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
    handle_button(Button::Hit)
}

#[wasm_bindgen]
pub fn on_button_stand() {
    handle_button(Button::Stand)
}

#[wasm_bindgen]
pub fn on_button_double() {
    handle_button(Button::Double)
}

#[wasm_bindgen]
pub fn on_button_split() {
    handle_button(Button::Split)
}

#[wasm_bindgen]
pub fn on_button_surrender() {
    handle_button(Button::Surrender)
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

