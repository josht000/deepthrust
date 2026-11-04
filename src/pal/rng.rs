//! Deterministic RNG (xoshiro256**) seeded from one master seed.
//! Every draw site forks the master stream by label, so each result
//! can be rebuilt exactly (spec: one master seed for every random draw).

/// The one master seed for the lab (decision #4).
pub const MASTER_SEED: u64 = 0x9E3779B97F4A7C15;

pub struct Rng {
    s: [u64; 4],
}

fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn label_hash(label: &str) -> u64 {
    // FNV-1a 64
    let mut h: u64 = 0xcbf29ce484222325;
    for b in label.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    mix64(h)
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut x = seed;
        Rng {
            s: [
                splitmix64(&mut x),
                splitmix64(&mut x),
                splitmix64(&mut x),
                splitmix64(&mut x),
            ],
        }
    }

    /// Deterministically derived stream for a named draw site.
    pub fn fork(label: &str) -> Rng {
        Rng::new(MASTER_SEED ^ label_hash(label))
    }

    /// Derive a sub-stream for an item index inside this site
    /// (e.g. per version, per bootstrap replicate).
    pub fn at(&self, index: u64) -> Rng {
        Rng::new(mix64(self.s[0] ^ index))
    }

    pub fn next_u64(&mut self) -> u64 {
        let [s0, s1, s2, s3] = self.s;
        let result = (s1.wrapping_mul(5)).rotate_left(7).wrapping_mul(9);
        let t = s1 << 17;
        self.s[2] = s2 ^ s0;
        self.s[3] = s3 ^ s1;
        self.s[1] = s1 ^ s2;
        self.s[0] = s0 ^ s3;
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform f64 in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in [lo, hi).
    pub fn gen_range(&mut self, lo: usize, hi: usize) -> usize {
        debug_assert!(hi > lo);
        lo + (self.next_u64() % (hi - lo) as u64) as usize
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.gen_range(0, i + 1);
            v.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Rng::fork("x");
        let mut b = Rng::fork("x");
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn labels_differ() {
        assert_ne!(Rng::fork("a").next_u64(), Rng::fork("b").next_u64());
    }

    #[test]
    fn range_bounds() {
        let mut r = Rng::fork("t");
        for _ in 0..1000 {
            let v = r.gen_range(3, 7);
            assert!((3..7).contains(&v));
        }
    }
}
