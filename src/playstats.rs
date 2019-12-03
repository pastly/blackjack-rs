use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Copy, Clone, Default, Debug)]
pub struct PlayStats {
    seen: u16,
    correct: u16,
}

impl PlayStats {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn weight(&self) -> f32 {
        if self.seen == 0 {
            return 1f32;
        }
        1f32 - self.correct as f32 / self.seen as f32
    }

    pub fn inc(&mut self, correct: bool) {
        self.inc_by(1, correct)
    }

    pub fn inc_by(&mut self, amt: u16, correct: bool) {
        self.seen += amt;
        if correct {
            self.correct += amt;
        }
    }

    pub fn seen(&self) -> u16 {
        self.seen
    }

    pub fn correct(&self) -> u16 {
        self.correct
    }
}

#[cfg(test)]
mod tests {
    use super::PlayStats;
    const COUNT_MANY: usize = 10;

    #[test]
    fn weight_one() {
        // brand new stats have weight of 1. primarily a test to avoid NaN from 0 rolls seen
        assert_eq!(PlayStats::new().weight(), 1f32);
    }

    #[test]
    fn weight_half() {
        let mut s = PlayStats::new();
        s.inc_by(10, false);
        s.inc_by(10, true);
        assert_eq!(s.weight(), 0.5);
        let mut s = PlayStats::new();
        s.inc(false);
        s.inc(true);
        assert_eq!(s.weight(), 0.5);
    }

    #[test]
    fn weight_quarters() {
        let mut s = PlayStats::new();
        s.inc_by(3, false);
        s.inc_by(1, true);
        assert_eq!(s.weight(), 0.75);
        let mut s = PlayStats::new();
        s.inc_by(30, true);
        s.inc_by(10, false);
        assert_eq!(s.weight(), 0.25);
    }

    #[test]
    fn weight_same() {
        let mut s1 = PlayStats::new();
        let mut s2 = PlayStats::new();
        s1.inc_by(10, false);
        s1.inc_by(30, true);
        s2.inc_by(1, false);
        s2.inc_by(3, true);
        assert_eq!(s1.weight(), s2.weight());
    }

    #[test]
    fn weight_incorrect() {
        // if always incorrect when inc, weight is 1
        let mut s = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s.inc(false);
            assert_eq!(s.weight(), 1f32);
        }
    }

    #[test]
    fn weight_correct() {
        // if always correct when inc, weight is 0
        let mut s = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s.inc(true);
            assert_eq!(s.weight(), 0f32);
        }
    }

    #[test]
    fn inc_incorrect() {
        // increment incorrect resps many times performs ... correctly
        let mut s = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s.inc(false);
        }
        assert_eq!(s.seen, COUNT_MANY as u16);
        assert_eq!(s.correct, 0);
    }

    #[test]
    fn inc_correct() {
        // increment correct resps many times performs ... correctly
        let mut s = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s.inc(true);
        }
        assert_eq!(s.seen, COUNT_MANY as u16);
        assert_eq!(s.correct, COUNT_MANY as u16);
    }

    #[test]
    fn inc_by_incorrect() {
        // increment by many works
        let mut s = PlayStats::new();
        s.inc_by(COUNT_MANY as u16, false);
        s.inc_by(COUNT_MANY as u16, false);
        assert_eq!(s.seen, COUNT_MANY as u16 * 2);
        assert_eq!(s.correct, 0);
    }

    #[test]
    fn inc_by_correct() {
        // increment by many works
        let mut s = PlayStats::new();
        s.inc_by(COUNT_MANY as u16, true);
        s.inc_by(COUNT_MANY as u16, true);
        assert_eq!(s.seen, COUNT_MANY as u16 * 2);
        assert_eq!(s.correct, COUNT_MANY as u16 * 2);
    }

    #[test]
    fn inc_same() {
        // inc and inc_by act the same
        let mut s1 = PlayStats::new();
        s1.inc_by(COUNT_MANY as u16, true);
        s1.inc_by(COUNT_MANY as u16, false);
        let mut s2 = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s2.inc(true);
            s2.inc(false);
        }
        assert_eq!(s1, s2);
    }
}
