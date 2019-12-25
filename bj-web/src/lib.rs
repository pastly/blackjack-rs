use wasm_bindgen::prelude::*;
use std::fmt;
use console_error_panic_hook;
use bj_core::deck::{Card, Deck, Rank, Suit};
use bj_core::hand::{Hand};
use bj_core::table::{resps_from_buf, Resp, Table};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref TABLE: Mutex<Table<Resp>> = Mutex::new(Table::new(resps_from_buf(
            T1_TXT)).unwrap());
}
lazy_static! {
    static ref DECK: Mutex<Deck> = Mutex::new(Deck::new_infinite());
}

const T1_TXT: &[u8] =
b"
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

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmCard(Card);

impl From<Card> for WasmCard {
    fn from(c: Card) -> Self {
        Self{ 0: c }
    }
}

impl WasmCard {
    pub fn as_char(self) -> char {
        // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode#Block
        let base: u32 = match self.0.suit() {
            Suit::Spade => 0x1F0A0,
            Suit::Heart => 0x1F0B0,
            Suit::Diamond => 0x1F0C0,
            Suit::Club => 0x1F0D0,
        };
        let val = base + match self.0.rank() {
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
}

/// A simplier representation of bj_core::hand::Hand that is easier to work with in wasm, but that
/// I think will have a harder time with invariants
#[wasm_bindgen]
pub struct WasmHand(Vec<WasmCard>);

impl WasmHand {
    fn cards(&self) -> Vec<WasmCard> {
        self.0.clone()
    }
}

impl From<Hand> for WasmHand {
    fn from(h: Hand) -> Self {
        assert!(h.cards().count() >= 2);
        Self { 0: h.into_cards().map(|c| c.into()).collect() }
    }
}

impl fmt::Display for WasmHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.cards()
                .iter()
                .map(|c| c.as_char())
                .collect::<String>()
        )
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn gen_hand() -> WasmHand {
    let mut deck = DECK.lock().unwrap();
    let h = Hand::new(&[deck.draw().unwrap(), deck.draw().unwrap()]).into();
    log(&format!("{}", h));
    h
}

#[wasm_bindgen]
pub fn gen_card() -> WasmCard {
    let mut deck = DECK.lock().unwrap();
    let table = TABLE.lock().unwrap();
    let player = Hand::new(&[deck.draw().unwrap(), deck.draw().unwrap()]);
    let dealer = deck.draw().unwrap();
    let resp = table.get(&player, dealer).unwrap();
    log(&format!("{} {} {}", player, dealer, resp));
    let c = Deck::new().draw().unwrap();
    //log(&format!("generated {:?}", c));
    c.into()
}
