pub mod bs_data;
pub mod button;
pub mod correct_resp;
pub mod localstorage;

use bj_core::deck::{Card, Rank, Suit};

pub fn card_char(card: Card) -> char {
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

pub fn char_card(c: char) -> Option<Card> {
    let c = c as u32;
    let (base, suit): (u32, _) = {
        if c > 0x1F0D0 { (0x1F0D0, Suit::Club) }
        else if c > 0x1F0C0 { (0x1F0C0, Suit::Diamond) }
        else if c > 0x1F0B0 { (0x1F0B0, Suit::Heart) }
        else if c > 0x1F0A0 { (0x1F0A0, Suit::Spade) }
        else { return None }
    };
    let rank = {
        let diff = c - base;
        match diff {
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
            // Unicode includes Knight here. Weird. Skip 12.
            13 => Rank::RQ,
            14 => Rank::RK,
            _ => return None,
        }
    };
    Some(Card::new(rank, suit))
}
