use bj_core::deck::Deck;
use bj_core::hand::Hand;
use bj_core::table::Table;
use clap::{crate_authors, crate_name, crate_version, value_t, App, Arg};
use rayon::prelude::*;
use std::io::{self, Write};

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
    let num_threads = 10;
    assert_eq!(num_hands % num_threads, 0);
    let tables: Vec<Table<usize>> = (0..num_threads)
        .into_par_iter()
        .map(|_| {
            let this_num_hands = num_hands / num_threads;
            let mut table: Table<usize> = Table::new(std::iter::repeat(0).take(360)).unwrap();
            let mut deck = Deck::new_infinite();
            for _ in 0..this_num_hands {
                let hand = Hand::new(&[deck.draw().unwrap(), deck.draw().unwrap()]);
                let dealer = deck.draw().unwrap();
                //eprintln!("{} / {}", hand, dealer);
                let count = table.get(&hand, dealer).unwrap();
                table.update(&hand, dealer, count + 1).unwrap();
            }
            table
        })
        .collect();
    let table = {
        let mut table: Table<usize> = Table::new(std::iter::repeat(0).take(360))?;
        for t in tables {
            table += t;
        }
        table
    };
    let (hard, soft, pair) = table.into_values_sorted();
    let hard: Vec<f64> = hard
        .into_iter()
        .map(|v| v as f64 / num_hands as f64)
        .collect();
    let soft: Vec<f64> = soft
        .into_iter()
        .map(|v| v as f64 / num_hands as f64)
        .collect();
    let pair: Vec<f64> = pair
        .into_iter()
        .map(|v| v as f64 / num_hands as f64)
        .collect();
    let mut fd = io::stdout();
    subtable(&mut fd, hard, "Hard")?;
    subtable(&mut fd, soft, "Soft")?;
    subtable(&mut fd, pair, "Pair")?;
    Ok(())
}

fn subtable(mut fd: impl Write, v: Vec<f64>, table_label: &str) -> io::Result<()> {
    let mut player_hand_val = match table_label {
        "Hard" => 5,
        "Soft" => 13,
        "Pair" => 2,
        _ => unreachable!("Impossible table_label"),
    };
    writeln!(fd, "<h1>{}</h1><table>", table_label)?;
    write!(fd, "<tr><td></td>")?;
    for i in 2..=11 {
        let s = if i == 11 {
            "A".to_string()
        } else {
            i.to_string()
        };
        writeln!(fd, "<th>{}</th>", s)?;
    }
    write!(fd, "</tr><tr>")?;
    //let mut dealer_val = 2;
    for (i, value) in v.iter().enumerate() {
        if i % 10 == 0 {
            let s = if player_hand_val == 11 && table_label == "Pair" {
                "A".to_string()
            } else {
                player_hand_val.to_string()
            };
            writeln!(fd, "<th>{}</th>", s)?;
        }
        let percent = value * 100.0;
        let red_shade = 50 + ((percent / 4.0) * 205.0) as u16;
        writeln!(
            fd,
            "<td style='color: white; background-color: rgb({}, 0, 0);'>{:.2}%</td>",
            red_shade, percent,
        )?;
        //dealer_val += 1;
        if i % 10 == 9 {
            writeln!(fd, "</tr><tr>")?;
            player_hand_val += 1;
            //dealer_val = 2;
        }
    }
    writeln!(fd, "</tr></table>")?;
    Ok(())
}
