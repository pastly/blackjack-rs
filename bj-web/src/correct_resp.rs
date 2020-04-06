use bj_core::deck::Card;
use bj_core::hand::Hand;
use bj_core::resp::Resp;
use super::button::Button;

pub(crate) fn is_correct_resp_button(btn: &Button, correct: Resp, hand: (&Hand, Card)) -> bool {
    let (player, _dealer) = hand;
    match btn {
        Button::Split => correct == Resp::Split,
        Button::Hit => {
            correct == Resp::Hit
                || correct == Resp::DoubleElseHit && !player.can_double()
                || correct == Resp::SurrenderElseHit && !player.can_surrender()
        }
        Button::Stand => {
            correct == Resp::Stand
                || correct == Resp::DoubleElseStand && !player.can_double()
                || correct == Resp::SurrenderElseStand && !player.can_surrender()
        }
        Button::Double => {
            correct == Resp::DoubleElseHit && player.can_double()
                || correct == Resp::DoubleElseStand && player.can_double()
        }
        Button::Surrender => {
            correct == Resp::SurrenderElseHit && player.can_surrender()
                || correct == Resp::SurrenderElseStand && player.can_surrender()
                || correct == Resp::SurrenderElseSplit && player.can_surrender()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bj_core::deck::rand_card;

    const NUM_RAND_HANDS: usize = 1000;

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

    #[test]
    fn hit_1() {
        // Hit button with Resp::Hit and a random hand should be correct
        let btn = Button::Hit;
        let correct = Resp::Hit;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn hit_2() {
        // Hit button with Resp::DoubleElseHit and a 3 card hand should be correct
        let btn = Button::Hit;
        let correct = Resp::DoubleElseHit;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn hit_3() {
        // Hit button with Resp::SurrenderElseHit and a 3 card hand should be correct
        let btn = Button::Hit;
        let correct = Resp::SurrenderElseHit;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn hit_bad_1() {
        // Hit button with Resp::DoubleElseHit and a 2 card hand should be wrong
        let btn = Button::Hit;
        let correct = Resp::DoubleElseHit;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn hit_bad_2() {
        // Hit button with Resp::SurrenderElseHit and a 2 card hand should be wrong
        let btn = Button::Hit;
        let correct = Resp::SurrenderElseHit;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn hit_bad_3() {
        // Hit button with Resp not involving hit are wrong
        let btn = Button::Hit;
        for correct in &[
            Resp::Stand,
            Resp::DoubleElseStand,
            Resp::Split,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
        }
    }

    #[test]
    fn stand_1() {
        // Stand button with Resp::Stand and a random hand should be correct
        let btn = Button::Stand;
        let correct = Resp::Stand;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn stand_2() {
        // Stand button with Resp::DoubleElseStand and 3 card hand is correct
        let btn = Button::Stand;
        let correct = Resp::DoubleElseStand;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn stand_3() {
        // Stand button with Resp::SurrenderElseStand and 3 card hand is correct
        let btn = Button::Stand;
        let correct = Resp::SurrenderElseStand;
        for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
            assert!(is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn stand_bad_1() {
        // Stand button with Resp::DoubleElseStand and 2 card hand is wrong
        let btn = Button::Stand;
        let correct = Resp::DoubleElseStand;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn stand_bad_2() {
        // Stand button with Resp::SurrenderElseStand and 2 card hand is wrong
        let btn = Button::Stand;
        let correct = Resp::SurrenderElseStand;
        for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
            assert!(!is_correct_resp_button(&btn, correct, (&player, dealer)));
        }
    }

    #[test]
    fn stand_bad_3() {
        // Stand button with Resp not involving stand are wrong
        let btn = Button::Stand;
        for correct in &[
            Resp::Hit,
            Resp::DoubleElseHit,
            Resp::Split,
            Resp::SurrenderElseHit,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
        }
    }

    #[test]
    fn double() {
        // Double button with DoubleElse* and 2 card hand is correct
        let btn = Button::Double;
        for correct in &[Resp::DoubleElseHit, Resp::DoubleElseStand] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
        }
    }

    #[test]
    fn double_bad_1() {
        // Double button with DoubleElse* and 3 card hand is wrong
        let btn = Button::Double;
        for correct in &[Resp::DoubleElseHit, Resp::DoubleElseStand] {
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
        }
    }

    #[test]
    fn double_bad_2() {
        // Double button with Resp not involving double is wrong
        let btn = Button::Double;
        for correct in &[
            Resp::Hit,
            Resp::Stand,
            Resp::Split,
            Resp::SurrenderElseHit,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
        }
    }

    #[test]
    fn surrender_bad_1() {
        // Surrender button with Resp not involve surrender is wrong
        let btn = Button::Surrender;
        for correct in &[
            Resp::Hit,
            Resp::Stand,
            Resp::DoubleElseHit,
            Resp::DoubleElseStand,
            Resp::Split,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
        }
    }

    #[test]
    fn surrender_1() {
        // Surrender button always correct with SurrenderElse* and 2 cards: the times it isn't
        // correct are when the table rules don't allow for it, thus it wouldn't show up in the
        // chart, thus this function wouldn't even be getting called with this combination of
        // arguments.
        //
        // Always correct with 2, always wrong with 3
        let btn = Button::Surrender;
        for correct in &[
            Resp::SurrenderElseHit,
            Resp::SurrenderElseStand,
            Resp::SurrenderElseSplit,
        ] {
            for (player, dealer) in random_hands(2, NUM_RAND_HANDS) {
                assert!(is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
            for (player, dealer) in random_hands(3, NUM_RAND_HANDS) {
                assert!(!is_correct_resp_button(&btn, *correct, (&player, dealer)));
            }
        }
    }
}
