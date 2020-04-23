mod button;
mod correct_resp;

use bj_core::basicstrategy::rules;
use bj_core::basicstrategy::BasicStrategy;
use bj_core::deck::{Card, Rank, Suit};
use bj_core::hand::Hand;
use bj_core::playstats::PlayStats;
use bj_core::rendertable::{HTMLTableRenderer, HTMLTableRendererOpts};
use bj_core::resp::Resp;
use bj_core::table::Table;
use bj_core::utils::{playstats_table, rand_next_hand, uniform_rand_2card_hand};
use bj_web_core::bs_data;
use bj_web_core::localstorage::{lskeys, LSVal};
use button::Button;
use console_error_panic_hook;
use correct_resp::is_correct_resp_button;
use std::default::Default;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
#[macro_use]
extern crate lazy_static;

const LS_KEY_TABLE_PLAYSTATS: &str = "bj-table-playstats";
const LS_KEY_STREAK: &str = "bj-streak";
const LS_KEY_EXISTING_HAND: &str = "bj-hand";

#[derive(Clone, Copy, Debug)]
enum RandHandType {
    // each card is drawn from the top of an shuffled inifinite deck
    Card,
    // a random cell is chosen from a basic strategy table, and a random hand constructed to fit
    // that cell
    Cell,
}

impl Default for RandHandType {
    fn default() -> Self {
        Self::Card
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct State {
    use_session_storage: bool,
    rand_hand_type: RandHandType,
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(Default::default());
}

fn new_play_stats() -> Table<PlayStats> {
    Table::new(std::iter::repeat(PlayStats::new()).take(360)).unwrap()
}

fn def_bs_card() -> BasicStrategy {
    serde_json::from_reader(bs_data::T1_JSON).unwrap()
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

fn debug_log(s: &str) {
    #[cfg(debug_assertions)]
    log(s)
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

fn set_state(new_state: State) {
    debug_log(&format!("Setting state {:?}", new_state));
    let mut old_state = STATE.lock().unwrap();
    *old_state = new_state;
}

#[wasm_bindgen]
pub fn rust_init(use_session_storage: bool, rand_hand_type: u8) {
    let state = State {
        use_session_storage,
        rand_hand_type: match rand_hand_type {
            0 => RandHandType::Card,
            1 => RandHandType::Cell,
            // purposefully vague
            _ => panic!("Invalid option specified"),
        },
        //..Default::default()
    };
    set_state(state);
    let stats = LSVal::from_ls_or_default(
        state.use_session_storage,
        LS_KEY_TABLE_PLAYSTATS,
        new_play_stats(),
    );
    let (player_hand, dealer_card) = &*LSVal::from_ls_or_default(
        state.use_session_storage,
        LS_KEY_EXISTING_HAND,
        rand_next_hand(&stats),
    );
    let stat_table = LSVal::from_ls_or_default(
        state.use_session_storage,
        LS_KEY_TABLE_PLAYSTATS,
        new_play_stats(),
    );
    let streak = LSVal::from_ls_or_default(state.use_session_storage, LS_KEY_STREAK, 0);
    output_hand(player_hand, *dealer_card);
    {
        let bs_card = LSVal::from_ls_or_default(
            state.use_session_storage,
            lskeys::LS_KEY_BS_CARD,
            def_bs_card(),
        );
        update_buttons((player_hand, *dealer_card), &bs_card.rules);
    }
    output_stats((player_hand, *dealer_card), &(*stat_table), *streak);
    output_resp_table(state);
}

fn output_resp_table(state: State) {
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

fn update_stats(state: State, old_hand: (&Hand, Card), old_was_correct: bool) {
    let mut stat_table = LSVal::from_ls_or_default(
        state.use_session_storage,
        LS_KEY_TABLE_PLAYSTATS,
        new_play_stats(),
    );
    let mut streak = LSVal::from_ls_or_default(state.use_session_storage, LS_KEY_STREAK, 0);
    let mut old_stat = stat_table.get(&old_hand.0, old_hand.1).unwrap();
    old_stat.inc(old_was_correct);
    stat_table
        .update(&old_hand.0, old_hand.1, old_stat)
        .unwrap();
    *streak = if old_was_correct { *streak + 1 } else { 0 };
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

fn handle_button(state: State, btn: Button) {
    // the (player_hand, dealer_card) currently on the screen
    let mut hand: LSVal<(Hand, Card)> =
        LSVal::from_ls(state.use_session_storage, LS_KEY_EXISTING_HAND).unwrap();
    // the correct response to this (player_hand, dealer_card). We store the bool is_correct as
    // well because whether or not the response is correct is more complex than button == resp: if
    // the correct Resp is DoubleElseHit (or its cousins) then it is not enough to simply check if
    // the Double button was pressed.
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
    let correct: Resp = bs_card.table.get(&hand.0, hand.1).unwrap();
    let is_correct = is_correct_resp_button(btn, correct, (&hand.0, hand.1), surrender_rule);
    // grab a copy of what the user's existing streak is. If they get the hand wrong, we will want
    // to display this to them and we will soon be clearing out the localstorage copy of their
    // streak
    let old_streak = *LSVal::from_ls_or_default(state.use_session_storage, LS_KEY_STREAK, 0);
    // update localstorage state for player statistics
    update_stats(state, (&hand.0, hand.1), is_correct);
    // display the "hint": player got it right, or they got it wrong and ___ is correct and ___ was
    // their streak
    set_hint(btn, correct, (&hand.0, hand.1), is_correct, old_streak);
    // get a LSVal-wrapped ref to their stats so that we can (1) generate an appropriate next hand
    // for them and (2) tell them what their stats are for the new hand
    let stats: LSVal<Table<PlayStats>> =
        LSVal::from_ls(state.use_session_storage, LS_KEY_TABLE_PLAYSTATS).unwrap();
    let _ = hand.swap(match state.rand_hand_type {
        RandHandType::Card => uniform_rand_2card_hand(),
        RandHandType::Cell => rand_next_hand(&*stats),
    });
    output_hand(&hand.0, hand.1);
    update_buttons((&hand.0, hand.1), &bs_card.rules);
    // update_stats() will have either incremented their streak or reset it to zero, so we need tor
    // refetch their streak from localstorage
    let new_streak = *LSVal::from_ls_or_default(state.use_session_storage, LS_KEY_STREAK, 0);
    output_stats((&hand.0, hand.1), &(*stats), new_streak);

    fn set_hint(given: Button, correct: Resp, hand: (&Hand, Card), is_correct: bool, streak: u32) {
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

fn is_legal_resp(btn: Button, hand: (&Hand, Card), surrender_rule: rules::Surrender) -> bool {
    let (player, dealer) = hand;
    match btn {
        Button::Hit | Button::Stand => true,
        Button::Double => player.can_double(),
        Button::Split => player.can_split(),
        Button::Surrender => player.can_surrender(surrender_rule, dealer),
    }
}

#[wasm_bindgen]
pub fn on_button_hit() {
    let state = STATE.lock().unwrap();
    handle_button(*state, Button::Hit)
}

#[wasm_bindgen]
pub fn on_button_stand() {
    let state = STATE.lock().unwrap();
    handle_button(*state, Button::Stand)
}

#[wasm_bindgen]
pub fn on_button_double() {
    let state = STATE.lock().unwrap();
    handle_button(*state, Button::Double)
}

#[wasm_bindgen]
pub fn on_button_split() {
    let state = STATE.lock().unwrap();
    handle_button(*state, Button::Split)
}

#[wasm_bindgen]
pub fn on_button_surrender() {
    let state = STATE.lock().unwrap();
    handle_button(*state, Button::Surrender)
}

#[wasm_bindgen]
pub fn on_button_clear_stats() {
    let state = STATE.lock().unwrap();
    let mut stat_table = LSVal::from_ls_or_default(
        state.use_session_storage,
        LS_KEY_TABLE_PLAYSTATS,
        new_play_stats(),
    );
    let mut streak = LSVal::from_ls_or_default(state.use_session_storage, LS_KEY_STREAK, 0);
    for v in stat_table.values_mut() {
        *v = PlayStats::new();
    }
    *streak = 0;
    let (player, dealer) =
        &*LSVal::from_ls(state.use_session_storage, LS_KEY_EXISTING_HAND).unwrap();
    output_stats((&player, *dealer), &(*stat_table), *streak);
}

#[wasm_bindgen]
pub fn playstats_as_db_str() -> String {
    let state = STATE.lock().unwrap();
    let table = LSVal::from_ls_or_default(
        state.use_session_storage,
        LS_KEY_TABLE_PLAYSTATS,
        new_play_stats(),
    );
    playstats_table::parse_to_string(&table)
}

#[wasm_bindgen]
pub fn playstats_db_str_into_storage(s: String) {
    let table = match playstats_table::parse_from_string(s) {
        Ok(t) => t,
        Err(e) => {
            debug_log(&format!("Couldn\'t parse string to table: {}", e));
            return;
        }
    };
    debug_log("Storing table in storage");
    let state = STATE.lock().unwrap();
    let mut stat_table = LSVal::from_ls_or_default(
        state.use_session_storage,
        LS_KEY_TABLE_PLAYSTATS,
        new_play_stats(),
    );
    stat_table.swap(table);
}
