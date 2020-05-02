use bj_core::deck::Card;
use bj_core::hand::Hand;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct HandResult {
    pub player: Hand,
    pub dealer: Card,
    pub correct: bool,
    // duration (in seconds, not ms) since first result at which point this result was recorded
    pub time: f64,
}
