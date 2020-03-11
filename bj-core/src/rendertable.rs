use crate::basicstrategy::{rules, BasicStrategy};
use crate::resp::Resp;
use std::io::{self, Write};

pub trait TableRenderer {
    fn render(fd: impl Write, strat: &BasicStrategy) -> io::Result<()>;
}

pub struct HTMLTableRenderer;

impl HTMLTableRenderer {
    fn header(mut fd: impl Write, bs_rules: &rules::Rules) -> io::Result<()> {
        let decks = match bs_rules.decks {
            rules::NumDecks::One => "1",
            rules::NumDecks::Two => "2",
            rules::NumDecks::Three => "3",
            rules::NumDecks::FourPlus => "4+",
        };
        let soft_17 = if bs_rules.hit_soft_17 {
            "Dealer hits"
        } else {
            "Dealer stands"
        };
        let das = if bs_rules.double_after_split {
            "Allowed"
        } else {
            "Not allowed"
        };
        let peek_bj = if bs_rules.peek_bj { "Yes" } else { "No" };
        let sur = match bs_rules.surrender {
            rules::Surrender::No => "No",
            rules::Surrender::Yes => "Any upcard",
            rules::Surrender::NotAce => "On dealer 2 through 10",
        };
        writeln!(
            fd,
            "
<style>
#strat_source {{ color: inherit; }}
.hit, .stand, .double, .split {{
    width:  1.5em;
    height: 1.5em;
    text-align: center;
    color: #333;
}}
.hit       {{ background-color: #ff3333; }}
.stand     {{ background-color: #ffff00; }}
.double    {{ background-color: #6666ff; }}
.split     {{ background-color: #00ff00; }}
.surrender {{ background-color: #dddddd; }}
</style>
<h1>Basic Strategy</h1>
<table>
<tr><td>Decks</td><td>{decks}</td></tr>
<tr><td>Soft 17</td><td>{soft_17}</td></tr>
<tr><td>Double after split</td><td>{das}</td></tr>
<tr><td>Surrender</td><td>{sur}</td></tr>
<tr><td>Dealer peek</td><td>{peek_bj}</td></tr>
</table>
",
            decks = decks,
            soft_17 = soft_17,
            das = das,
            peek_bj = peek_bj,
            sur = sur,
        )
    }

    fn footer(mut fd: impl Write) -> io::Result<()> {
        writeln!(
            fd,
            "
<span class=hit>H</span>&nbsp;Hit<br/>
<span class=stand>S</span>&nbsp;Stand<br/>
<span class=double>D</span>&nbsp;Double<br/>
<span class=split>P</span>&nbsp;Split<br/>
<span class=surrender>R</span>&nbsp;Surrender<br/>
Source: <a id=strat_source href='https://wizardofodds.com/games/blackjack/strategy/calculator/'>wizardofodds.com</a><br/>
"
        )
    }

    fn subtable(mut fd: impl Write, v: Vec<&Resp>, label: &str) -> io::Result<()> {
        let mut player_hand_val = match label {
            "Hard" => 5,
            "Soft" => 13,
            "Pair" => 2,
            _ => unreachable!("Impossible label"),
        };
        writeln!(fd, "<h1>{}</h1><table>", label)?;
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
        for (i, resp) in v.iter().enumerate() {
            if i % 10 == 0 {
                let s = player_hand_val.to_string();
                writeln!(fd, "<th>{}</th>", s)?;
                player_hand_val += 1;
            }
            let (class, label) = match resp {
                Resp::Hit => ("hit", "H"),
                Resp::Stand => ("stand", "S"),
                Resp::DoubleElseHit => ("double", "Dh"),
                Resp::DoubleElseStand => ("double", "Ds"),
                Resp::Split => ("split", "P"),
                Resp::SurrenderElseHit => ("surrender", "Rh"),
                Resp::SurrenderElseStand => ("surrender", "Rs"),
                Resp::SurrenderElseSplit => ("surrender", "Rp"),
            };
            writeln!(fd, "<td class={}>{}</td>", class, label)?;
            if i % 10 == 9 {
                writeln!(fd, "</tr><tr>")?;
            }
        }
        writeln!(fd, "</tr></table>")?;
        Ok(())
    }
}

impl TableRenderer for HTMLTableRenderer {
    fn render(mut fd: impl Write, strat: &BasicStrategy) -> io::Result<()> {
        //fn render(mut fd: impl Write, table: Table<Resp>) -> io::Result<()> {
        let BasicStrategy { rules, table } = strat;
        let (hards, softs, pairs) = table.as_values_sorted();
        HTMLTableRenderer::header(&mut fd, &rules)?;
        HTMLTableRenderer::subtable(&mut fd, hards, "Hard")?;
        HTMLTableRenderer::subtable(&mut fd, softs, "Soft")?;
        HTMLTableRenderer::subtable(&mut fd, pairs, "Pair")?;
        HTMLTableRenderer::footer(&mut fd)?;
        Ok(())
    }
}
