use std::fmt;

#[derive(Clone, Copy)]
pub enum GameButton {
    Hit,
    Stand,
    Double,
    Split,
    Surrender,
}

impl fmt::Display for GameButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Hit => "Hit",
                Self::Stand => "Stand",
                Self::Double => "Double",
                Self::Split => "Split",
                Self::Surrender => "Surrender",
            }
        )
    }
}
