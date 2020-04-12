use crate::basicstrategy::{rules, BasicStrategy};
use crate::resp::Resp;
use std::io::{self, Write};

pub struct HTMLTableRendererOpts {
    pub incl_bs_rules: bool,
    pub cell_onclick_cb: Option<String>,
}

pub struct HTMLTableRenderer;

impl HTMLTableRenderer {
    fn header(
        mut fd: impl Write,
        bs_rules: &Option<rules::Rules>,
        opts: &HTMLTableRendererOpts,
    ) -> io::Result<()> {
        writeln!(
            fd,
            "
<style>
#strat_source {{ color: inherit; }}
.hit, .stand, .double, .split, .surrender {{
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
"
        )?;
        if opts.incl_bs_rules {
            if let Some(rules) = bs_rules {
                writeln!(
                    fd,
                    "
<h1>Basic Strategy</h1>
<table>
<tr><td>Decks</td><td>{decks}</td></tr>
<tr><td>Soft 17</td><td>{soft_17}</td></tr>
<tr><td>Double after split</td><td>{das}</td></tr>
<tr><td>Surrender</td><td>{sur}</td></tr>
<tr><td>Dealer peek</td><td>{peek_bj}</td></tr>
</table>
    ",
                    decks = rules.decks,
                    soft_17 = rules.hit_soft_17,
                    das = rules.double_after_split,
                    peek_bj = rules.peek_bj,
                    sur = rules.surrender,
                )?;
            } else {
                writeln!(fd, "<h1>Custom Basic Strategy</h1>")?;
            }
        }
        Ok(())
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

    fn subtable(
        mut fd: impl Write,
        v: Vec<&Resp>,
        table_label: &str,
        opts: &HTMLTableRendererOpts,
    ) -> io::Result<()> {
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
        let mut dealer_val = 2;
        for (i, resp) in v.iter().enumerate() {
            if i % 10 == 0 {
                let s = if player_hand_val == 11 && table_label == "Pair" {
                    "A".to_string()
                } else {
                    player_hand_val.to_string()
                };
                writeln!(fd, "<th>{}</th>", s)?;
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
            let onclick_fn = match &opts.cell_onclick_cb {
                Some(fn_name) => format!(
                    "{}(\"{}\", {}, {})",
                    fn_name,
                    table_label.to_lowercase(),
                    player_hand_val,
                    dealer_val
                ),
                None => "".to_string(),
            };
            writeln!(
                fd,
                "<td class={} onclick='{}'>{}</td>",
                class, onclick_fn, label,
            )?;
            dealer_val += 1;
            if i % 10 == 9 {
                writeln!(fd, "</tr><tr>")?;
                player_hand_val += 1;
                dealer_val = 2;
            }
        }
        writeln!(fd, "</tr></table>")?;
        Ok(())
    }

    pub fn render(
        mut fd: impl Write,
        strat: &BasicStrategy,
        opts: HTMLTableRendererOpts,
    ) -> io::Result<()> {
        let BasicStrategy { rules, table } = strat;
        let (hards, softs, pairs) = table.as_values_sorted();
        Self::header(&mut fd, rules, &opts)?;
        Self::subtable(&mut fd, hards, "Hard", &opts)?;
        Self::subtable(&mut fd, softs, "Soft", &opts)?;
        Self::subtable(&mut fd, pairs, "Pair", &opts)?;
        Self::footer(&mut fd)?;
        Ok(())
    }
}

pub struct TXTTableRenderer;

impl TXTTableRenderer {
    fn header(mut fd: impl Write, bs_rules: &Option<rules::Rules>) -> io::Result<()> {
        if let Some(rules) = bs_rules {
            writeln!(
                fd,
                "
# Decks:              {decks}
# Soft 17:            {soft_17}
# Double after split: {das}
# Surrender:          {sur}
# Dealer peek:        {peek_bj}
# Source: https://wizardofodds.com/games/blackjack/strategy/calculator/
    ",
                decks = rules.decks,
                soft_17 = rules.hit_soft_17,
                das = rules.double_after_split,
                sur = rules.surrender,
                peek_bj = rules.peek_bj,
            )
        } else {
            writeln!(fd, "# Custom Basic Strategy")
        }
    }

    fn subtable(mut fd: impl Write, v: Vec<&Resp>, label: &str) -> io::Result<()> {
        writeln!(fd, "# {} table", label)?;
        for (i, resp) in v.iter().enumerate() {
            let label = match resp {
                Resp::Hit => "H ",
                Resp::Stand => "S ",
                Resp::DoubleElseHit => "Dh",
                Resp::DoubleElseStand => "Ds",
                Resp::Split => "P ",
                Resp::SurrenderElseHit => "Rh",
                Resp::SurrenderElseStand => "Rs",
                Resp::SurrenderElseSplit => "Rp",
            };
            write!(fd, "{} ", label)?;
            if i % 10 == 9 {
                writeln!(fd)?;
            }
        }
        Ok(())
    }

    pub fn render(mut fd: impl Write, strat: &BasicStrategy) -> io::Result<()> {
        let BasicStrategy { rules, table } = strat;
        let (hards, softs, pairs) = table.as_values_sorted();
        Self::header(&mut fd, &rules)?;
        Self::subtable(&mut fd, hards, "Hard")?;
        writeln!(fd)?;
        Self::subtable(&mut fd, softs, "Soft")?;
        writeln!(fd)?;
        Self::subtable(&mut fd, pairs, "Pair")?;
        writeln!(fd)?;
        Ok(())
    }
}
