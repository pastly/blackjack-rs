use bj_core::deck::Deck;
use bj_core::hand::Hand;
use bj_core::table::{GameDesc, Table};
use clap::{crate_authors, crate_name, crate_version, value_t, App, Arg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(String::from(crate_name!()) + " likelyhood")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("num")
                .short("n")
                .long("num-hands")
                .value_name("N")
                .help("Number of hands to simulate")
                .default_value("10"),
        )
        .get_matches();
    let num_hands = value_t!(matches, "num", usize)?;
    let mut deck = Deck::new_infinite();
    let mut table: Table<usize> = Table::new();
    table.fill(std::iter::repeat(0).take(360))?;
    for _ in 0..num_hands {
        let hand = Hand::new(&[deck.draw()?, deck.draw()?]);
        let dealer = deck.draw()?;
        //eprintln!("{} / {}", hand, dealer);
        let count = table.get(&hand, dealer)?;
        table.update(&hand, dealer, count + 1)?;
    }
    let mut hands: Vec<(GameDesc, &usize)> = table.iter().collect();
    hands.sort_by_key(|a| a.1);
    hands.reverse();
    for h in hands.iter().take(10) {
        println!("{:?} {}", h.0, h.1);
    }
    Ok(())
}
