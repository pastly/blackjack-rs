use bj_bin::prompt;
use bj_core::deck::{Card, Deck};
use bj_core::hand::{Hand, HandError};
use bj_core::resp::Resp;
use std::io::{self, BufRead, BufReader, Write};

fn hand_with_value(hand: &Hand) -> String {
    format!(
        "{} ({}{})",
        hand,
        hand.value(),
        if hand.is_soft() { "s" } else { "" },
    )
}

fn _prompt_for_bet(
    in_buf: &mut impl BufRead,
    out_buf: &mut impl Write,
) -> io::Result<prompt::Command> {
    let s = "bet";
    loop {
        if let prompt::Command::Bet(amt) = prompt::prompt(s, in_buf, out_buf)? {
            break Ok(prompt::Command::Bet(amt));
        } else {
            writeln!(out_buf, "Expecting bet. E.g. 'bet 10'")?;
        }
    }
}

fn prompt_for_resp(
    p: &Hand,
    d: Card,
    in_buf: &mut impl BufRead,
    out_buf: &mut impl Write,
) -> io::Result<prompt::Command> {
    let s = &format!("{} / {}", hand_with_value(&p), d);
    loop {
        match prompt::prompt(s, in_buf, out_buf)? {
            prompt::Command::Save => {
                writeln!(out_buf, "Nothing to save")?;
                continue;
            }
            prompt::Command::Bet(_) => {
                writeln!(out_buf, "Not time to bet")?;
                continue;
            }
            prompt::Command::SaveQuit => break Ok(prompt::Command::Quit),
            cmd => break Ok(cmd),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = BufReader::new(io::stdin());
    let mut output = io::stdout();
    let mut working_hands: Vec<Hand> = vec![];
    let mut finished_hands: Vec<Hand> = vec![];
    let mut deck = Deck::new_infinite();
    // play forever
    loop {
        // make sure no left over hands
        assert!(working_hands.is_empty());
        assert!(finished_hands.is_empty());
        //// Disabled bet stuff
        //let bet = match prompt_for_bet(&mut input, &mut output)? {
        //    prompt::Command::Bet(amt) => amt,
        //    _ => unreachable!(),
        //};
        //println!("{:?}", bet);
        // generate hands
        // TODO handle dealer up 10 or up ace
        let dealer = deck.draw()?;
        working_hands.push(Hand::new(&[deck.draw()?, deck.draw()?]));
        // keep looping while the player has an unfinished hand and we need a response from them.
        // They will have more than one hand if they split, and if starting out with 2+ hands is
        // ever implemented without updating this comment.
        while let Some(mut hand) = working_hands.pop() {
            // prompt them for their move. They can quit or they can stand/hit/etc.
            // prompt_for_resp() will not return any other type of command
            let resp = match prompt_for_resp(&hand, dealer, &mut input, &mut output)? {
                prompt::Command::Quit => return Ok(()),
                prompt::Command::Resp(r) => r,
                _ => unreachable!(),
            };
            // player wants to stand/hit/etc. this hand. So handle that.
            match resp {
                Resp::DoubleElseHit | Resp::DoubleElseStand => {
                    // TODO need to add to the bet if betting is implemented
                    if !hand.can_double() {
                        println!("Cannot double {}", hand_with_value(&hand));
                        working_hands.push(hand);
                        continue;
                    }
                    hand.push(deck.draw()?);
                    println!(
                        "dobule {}{}",
                        hand_with_value(&hand),
                        if hand.is_bust() { " bust" } else { "" }
                    );
                    finished_hands.push(hand);
                }
                Resp::Split => {
                    // TODO need to have two bets if betting is implemented
                    let (c1, c2) = match hand.split() {
                        Err(HandError::CannotSplit(h)) => {
                            println!("Cannot split {}", hand_with_value(&h));
                            working_hands.push(h);
                            continue;
                        }
                        Ok((c1, c2)) => (c1, c2),
                        _ => unreachable!(),
                    };
                    working_hands.push(Hand::new(&[c2, deck.draw()?]));
                    working_hands.push(Hand::new(&[c1, deck.draw()?]));
                }
                Resp::Stand => {
                    println!("stand {}", hand_with_value(&hand));
                    finished_hands.push(hand);
                }
                Resp::Hit => {
                    hand.push(deck.draw()?);
                    if hand.is_bust() {
                        println!("{} bust", hand_with_value(&hand));
                        finished_hands.push(hand);
                    } else {
                        working_hands.push(hand);
                    }
                }
                Resp::SurrenderElseHit | Resp::SurrenderElseStand | Resp::SurrenderElseSplit => {
                    unimplemented!()
                }
            };
            // There must not be anything else at the end of this loop.
            //
            // Okay maybe there can be. But the preceeding match statement on the player's
            // hit/stand/etc. choice sometimes continues back to the beginning of this while loop
            // and sometimes falls through to here. As of now there is nothing here so the
            // difference is inconsequential. But if something is added here, maybe continues
            // should be added/removed.
        }
        // Done with player. Have the dealer deal themself build their hand
        assert!(working_hands.is_empty());
        let mut dealer = Hand::new(&[dealer, deck.draw()?]);
        // The implemented game is hit soft 17
        while dealer.value() < 16 || (dealer.value() == 17 && dealer.is_soft()) {
            println!("dealer {}", hand_with_value(&dealer));
            dealer.push(deck.draw()?);
        }
        println!("dealer {}", hand_with_value(&dealer));
        // let the player know what happened
        for hand in finished_hands.drain(0..) {
            if hand.is_bust() {
                println!("bust {}", hand_with_value(&hand));
            } else if dealer.is_bust() {
                assert!(!hand.is_bust());
                println!("dealer bust; win {}", hand_with_value(&hand));
            } else if hand.value() > dealer.value() {
                assert!(!hand.is_bust());
                assert!(!dealer.is_bust());
                println!(
                    "win {} over dealer {}",
                    hand_with_value(&hand),
                    hand_with_value(&dealer),
                );
            } else if hand.value() == dealer.value() {
                assert!(!hand.is_bust());
                assert!(!dealer.is_bust());
                println!(
                    "push {} with dealer {}",
                    hand_with_value(&hand),
                    hand_with_value(&dealer),
                );
            } else {
                assert!(!hand.is_bust());
                assert!(!dealer.is_bust());
                assert!(hand.value() < dealer.value());
                println!(
                    "lose {} to dealer {}",
                    hand_with_value(&hand),
                    hand_with_value(&dealer),
                );
            }
        }
    }
}
