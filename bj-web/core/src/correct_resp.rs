use crate::button::GameButton;
use bj_core::basicstrategy::rules::Surrender;
use bj_core::deck::Card;
use bj_core::hand::Hand;
use bj_core::resp::Resp;

pub fn is_correct_resp_button(
    btn: GameButton,
    correct: Resp,
    hand: (&Hand, Card),
    surrender_rule: Surrender,
) -> bool {
    let (player, dealer) = hand;
    let can_double = player.can_double();
    let can_surrender = player.can_surrender(surrender_rule, dealer);
    match btn {
        GameButton::Split => correct == Resp::Split,
        GameButton::Hit => {
            correct == Resp::Hit
                || correct == Resp::DoubleElseHit && !can_double
                || correct == Resp::SurrenderElseHit && !can_surrender
        }
        GameButton::Stand => {
            correct == Resp::Stand
                || correct == Resp::DoubleElseStand && !can_double
                || correct == Resp::SurrenderElseStand && !can_surrender
        }
        GameButton::Double => {
            can_double && (correct == Resp::DoubleElseHit || correct == Resp::DoubleElseStand)
        }
        GameButton::Surrender => {
            can_surrender
                && (correct == Resp::SurrenderElseHit
                    || correct == Resp::SurrenderElseStand
                    || correct == Resp::SurrenderElseSplit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bj_core::deck::{rand_card, Rank};
    use rand::prelude::*;

    const NUM_RAND_HANDS: usize = 5000;

    struct RandomHandIter {
        hand_size: usize,
    }

    impl Iterator for RandomHandIter {
        type Item = (Hand, Card);

        fn next(&mut self) -> Option<Self::Item> {
            let mut player_cards = Vec::with_capacity(self.hand_size);
            player_cards.extend(std::iter::repeat_with(rand_card).take(self.hand_size));
            let player = Hand::new(&player_cards);
            let dealer = rand_card();
            Some((player, dealer))
        }
    }

    fn random_hands(hand_size: usize, n: usize) -> impl Iterator<Item = (Hand, Card)> {
        RandomHandIter { hand_size }.into_iter().take(n)
    }

    fn random_surrender_rule() -> Surrender {
        let mut rng = thread_rng();
        *[Surrender::No, Surrender::Yes, Surrender::NotAce]
            .choose(&mut rng)
            .unwrap()
    }

    #[test]
    fn hit_1() {
        // Hit button with Resp::Hit and a random hand should be correct
        let btn = GameButton::Hit;
        let correct = Resp::Hit;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn hit_2() {
        // Hit button with Resp::DoubleElseHit and a 3 card hand should be correct
        let btn = GameButton::Hit;
        let correct = Resp::DoubleElseHit;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn hit_3() {
        // Hit button with Resp::SurrenderElseHit and a 3 card hand should be correct
        let btn = GameButton::Hit;
        let correct = Resp::SurrenderElseHit;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn hit_bad_1() {
        // Hit button with Resp::DoubleElseHit and a 2 card hand should be wrong
        let btn = GameButton::Hit;
        let correct = Resp::DoubleElseHit;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn hit_bad_2() {
        // Hit button with Resp::SurrenderElseHit and a 2 card hand should be wrong
        let btn = GameButton::Hit;
        let correct = Resp::SurrenderElseHit;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                Surrender::Yes
            ));
        }
    }

    #[test]
    fn hit_bad_3() {
        // Hit button with Resp not involving hit are wrong
        let btn = GameButton::Hit;
        for correct in &[
            Resp::Stand,
            Resp::DoubleElseStand,
            Resp::Split,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
        }
    }

    #[test]
    fn stand_1() {
        // Stand button with Resp::Stand and a random hand should be correct
        let btn = GameButton::Stand;
        let correct = Resp::Stand;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn stand_2() {
        // Stand button with Resp::DoubleElseStand and 3 card hand is correct
        let btn = GameButton::Stand;
        let correct = Resp::DoubleElseStand;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn stand_3() {
        // Stand button with Resp::SurrenderElseStand and 3 card hand is correct
        let btn = GameButton::Stand;
        let correct = Resp::SurrenderElseStand;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn stand_bad_1() {
        // Stand button with Resp::DoubleElseStand and 2 card hand is wrong
        let btn = GameButton::Stand;
        let correct = Resp::DoubleElseStand;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                random_surrender_rule()
            ));
        }
    }

    #[test]
    fn stand_bad_2() {
        // Stand button with Resp::SurrenderElseStand and 2 card hand is wrong
        let btn = GameButton::Stand;
        let correct = Resp::SurrenderElseStand;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(
                btn,
                correct,
                (&player, dealer),
                Surrender::Yes
            ));
        }
    }

    #[test]
    fn stand_bad_3() {
        // Stand button with Resp not involving stand are wrong
        let btn = GameButton::Stand;
        for correct in &[
            Resp::Hit,
            Resp::DoubleElseHit,
            Resp::Split,
            Resp::SurrenderElseHit,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
        }
    }

    #[test]
    fn double() {
        // Double button with DoubleElse* and 2 card hand is correct
        let btn = GameButton::Double;
        for correct in &[Resp::DoubleElseHit, Resp::DoubleElseStand] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
        }
    }

    #[test]
    fn double_bad_1() {
        // Double button with DoubleElse* and 3 card hand is wrong
        let btn = GameButton::Double;
        for correct in &[Resp::DoubleElseHit, Resp::DoubleElseStand] {
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
        }
    }

    #[test]
    fn double_bad_2() {
        // Double button with Resp not involving double is wrong
        let btn = GameButton::Double;
        for correct in &[
            Resp::Hit,
            Resp::Stand,
            Resp::Split,
            Resp::SurrenderElseHit,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
        }
    }

    #[test]
    fn surrender_bad_1() {
        // Surrender button with Resp not involve surrender is wrong
        let btn = GameButton::Surrender;
        for correct in &[
            Resp::Hit,
            Resp::Stand,
            Resp::DoubleElseHit,
            Resp::DoubleElseStand,
            Resp::Split,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    random_surrender_rule()
                ));
            }
        }
    }

    #[test]
    fn surrender_no() {
        // Surrender always wrong if rules don't allow surrendering
        let btn = GameButton::Surrender;
        for correct in &[
            Resp::SurrenderElseHit,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    Surrender::No
                ));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    Surrender::No
                ));
            }
        }
    }

    #[test]
    fn surrender_yes() {
        // Surrender right/wrong only depends on number of cards in hand
        let btn = GameButton::Surrender;
        // Never can surrenter 3 card hands
        for correct in &[
            Resp::SurrenderElseHit,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    Surrender::Yes
                ));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    Surrender::Yes
                ));
            }
        }
    }

    #[test]
    fn surrender_notace() {
        // Surrender right/wrong depends on various factors
        let btn = GameButton::Surrender;
        // Never can surrenter 3 card hands
        for correct in &[
            Resp::SurrenderElseHit,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(
                    btn,
                    *correct,
                    (&player, dealer),
                    Surrender::NotAce
                ));
            }
        }
        // 2 card hands can surrender depending only on dealer (non-)ace
        for correct in &[
            Resp::SurrenderElseHit,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                let is_correct = dealer.rank() != Rank::RA;
                assert_eq!(
                    is_correct,
                    is_correct_resp_button(btn, *correct, (&player, dealer), Surrender::NotAce)
                );
            }
        }
    }
}
