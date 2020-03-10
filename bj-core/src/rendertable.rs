use crate::resp::Resp;
use crate::table::Table;
use std::io::{self, Write};

pub trait TableRenderer {
    fn render(fd: impl Write, table: Table<Resp>) -> io::Result<()>;
}

pub struct HTMLTableRenderer;

impl HTMLTableRenderer {
    fn header(mut fd: impl Write) -> io::Result<()> {
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
<tr><td>Decks</td><td>4+</td></tr>
<tr><td>Soft 17</td><td>Dealer hits</td></tr>
<tr><td>Double after split</td><td>Allowed</td></tr>
<tr><td>Surrender</td><td>Not allowed</td></tr>
<tr><td>Dealer peek</td><td>Dealer peeks for BJ</td></tr>
</table>
"
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

    fn subtable(mut fd: impl Write, v: Vec<Resp>, label: &str) -> io::Result<()> {
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
    fn render(mut fd: impl Write, table: Table<Resp>) -> io::Result<()> {
        let (hards, softs, pairs) = table.into_values_sorted();
        HTMLTableRenderer::header(&mut fd)?;
        HTMLTableRenderer::subtable(&mut fd, hards, "Hard")?;
        HTMLTableRenderer::subtable(&mut fd, softs, "Soft")?;
        HTMLTableRenderer::subtable(&mut fd, pairs, "Pair")?;
        HTMLTableRenderer::footer(&mut fd)?;
        Ok(())
    }
}
