#[derive(Debug, Clone)]
pub struct GameRng {
    state: [i64; 31],
    fptr: usize,
    rptr: usize,
    seed: i32,
}

impl GameRng {
    pub fn new(seed: i32) -> Self {
        let mut rng = Self {
            state: [0; 31],
            fptr: 3,
            rptr: 0,
            seed,
        };
        rng.reseed(seed);
        rng
    }

    pub fn seed(&self) -> i32 {
        self.seed
    }

    pub fn reseed(&mut self, seed: i32) {
        self.seed = seed;
        self.state[0] = seed as i64;

        for i in 1..31 {
            self.state[i] = 1_103_515_245_i64
                .wrapping_mul(self.state[i - 1])
                .wrapping_add(12_345);
        }

        self.fptr = 3;
        self.rptr = 0;

        for _ in 0..(10 * 31) {
            let _ = self.rrandom();
        }
    }

    pub fn rrandom(&mut self) -> i64 {
        self.state[self.fptr] = self.state[self.fptr].wrapping_add(self.state[self.rptr]);
        let value = (self.state[self.fptr] >> 1) & 0x7fff_ffff;

        self.fptr += 1;
        if self.fptr >= 31 {
            self.fptr = 0;
        }

        self.rptr += 1;
        if self.rptr >= 31 {
            self.rptr = 0;
        }

        value
    }

    pub fn get_rand(&mut self, mut x: i32, mut y: i32) -> i32 {
        if x > y {
            std::mem::swap(&mut x, &mut y);
        }

        let mut lr = self.rrandom();
        lr &= 0x0000_7fff;

        let span = (y - x) + 1;
        ((lr as i32) % span) + x
    }

    pub fn rand_percent(&mut self, percentage: i32) -> bool {
        self.get_rand(1, 100) <= percentage
    }

    pub fn coin_toss(&mut self) -> bool {
        (self.rrandom() & 1) == 1
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::GameRng;

    #[test]
    fn rrandom_matches_c_seed_12345() {
        let mut rng = GameRng::new(12345);
        let got: Vec<i64> = (0..10).map(|_| rng.rrandom()).collect();
        let expected = vec![
            1720401481, 2096901210, 1997223871, 1743202534, 376205223, 1143102709, 524743342,
            48316531, 149193331, 1357336562,
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn get_rand_matches_c_seed_12345() {
        let mut rng = GameRng::new(12345);
        let got: Vec<i32> = (0..10).map(|_| rng.get_rand(1, 100)).collect();
        let expected = vec![46, 55, 72, 71, 84, 98, 59, 100, 28, 67];
        assert_eq!(got, expected);
    }

    #[test]
    fn rrandom_matches_c_seed_1() {
        let mut rng = GameRng::new(1);
        let got: Vec<i64> = (0..10).map(|_| rng.rrandom()).collect();
        let expected = vec![
            2078917053, 143302914, 1027100827, 1953210302, 755253631, 2002600785, 1405390230,
            45248011, 1099951567, 433832350,
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn rand_percent_matches_c_seed_1() {
        let mut rng = GameRng::new(1);
        let got: Vec<i32> = (0..10)
            .map(|_| if rng.rand_percent(50) { 1 } else { 0 })
            .collect();
        let expected = vec![1, 0, 1, 1, 0, 1, 0, 0, 1, 0];
        assert_eq!(got, expected);
    }

    proptest! {
        #[test]
        fn get_rand_always_within_requested_bounds(seed in any::<i32>(), x in -500i32..500, y in -500i32..500) {
            let mut rng = GameRng::new(seed);
            let value = rng.get_rand(x, y);
            let lo = x.min(y);
            let hi = x.max(y);

            prop_assert!(value >= lo);
            prop_assert!(value <= hi);
        }

        #[test]
        fn reseed_reproduces_rrandom_sequence(seed in any::<i32>()) {
            let mut rng = GameRng::new(seed);
            let first: Vec<i64> = (0..20).map(|_| rng.rrandom()).collect();

            rng.reseed(seed);
            let second: Vec<i64> = (0..20).map(|_| rng.rrandom()).collect();

            prop_assert_eq!(first, second);
        }
    }
}
