//! Small numeric helpers: seeded RNG, inverse normal CDF, quantiles, Bailey/Lopez de Prado haircut.

pub const SEED: u64 = 20261101;
#[allow(clippy::approx_constant)]
pub const EULER: f64 = 0.5772156649; // same truncated value as the Python partition module

/// splitmix64-seeded xorshift64* generator (deterministic, dependency-free).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Rng((z ^ (z >> 31)) | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Uniform integer in [0, n).
    pub fn below(&mut self, n: usize) -> usize {
        ((self.next_f64() * n as f64) as usize).min(n.saturating_sub(1))
    }
    /// Uniform integer in [lo, hi).
    pub fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + self.below(hi.saturating_sub(lo).max(1))
    }
}

/// Inverse standard normal CDF (Acklam's rational approximation, |rel err| < 1.2e-9).
pub fn norm_ppf(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996,
        3.754408661907416,
    ];
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    if p < 0.02425 {
        tail((-2.0 * p.ln()).sqrt())
    } else if p > 1.0 - 0.02425 {
        -tail((-2.0 * (1.0 - p).ln()).sqrt())
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    }
}

/// Expected maximum total log return of N zero-skill trials (Bailey & Lopez de Prado).
pub fn expected_max_noise(sd_trial: f64, n_trials: f64) -> f64 {
    if n_trials < 2.0 || !sd_trial.is_finite() {
        return 0.0;
    }
    sd_trial
        * ((1.0 - EULER) * norm_ppf(1.0 - 1.0 / n_trials)
            + EULER * norm_ppf(1.0 - 1.0 / (n_trials * std::f64::consts::E)))
}

pub fn mean(x: &[f64]) -> f64 {
    if x.is_empty() {
        f64::NAN
    } else {
        x.iter().sum::<f64>() / x.len() as f64
    }
}

/// Sample standard deviation (ddof = 1).
pub fn std(x: &[f64]) -> f64 {
    if x.len() < 2 {
        return f64::NAN;
    }
    let m = mean(x);
    (x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (x.len() - 1) as f64).sqrt()
}

pub fn median(x: &[f64]) -> f64 {
    if x.is_empty() {
        return f64::NAN;
    }
    let mut v = x.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// Linear-interpolation quantile (numpy default) of an already sorted slice.
pub fn quantile_sorted(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let pos = q * (v.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    v[lo] + (v[hi] - v[lo]) * (pos - lo as f64)
}

/// Pearson correlation.
pub fn corr(a: &[f64], b: &[f64]) -> f64 {
    let (ma, mb) = (mean(a), mean(b));
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma).powi(2);
        sbb += (y - mb).powi(2);
    }
    sab / (saa * sbb).sqrt()
}

/// OLS slope and its t-stat for y ~ 1 + x.
pub fn ols(y: &[f64], x: &[f64]) -> (f64, f64) {
    let n = x.len();
    if n < 5 {
        return (f64::NAN, f64::NAN);
    }
    let (mx, my) = (mean(x), mean(y));
    let sxx: f64 = x.iter().map(|v| (v - mx).powi(2)).sum();
    if sxx == 0.0 {
        return (f64::NAN, f64::NAN);
    }
    let b = x
        .iter()
        .zip(y)
        .map(|(a, c)| (a - mx) * (c - my))
        .sum::<f64>()
        / sxx;
    let a = my - b * mx;
    let rss: f64 = x
        .iter()
        .zip(y)
        .map(|(xi, yi)| (yi - a - b * xi).powi(2))
        .sum();
    let se = (rss / (n - 2) as f64 / sxx).sqrt();
    (b, if se > 0.0 { b / se } else { f64::NAN })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppf_known_values() {
        assert!(norm_ppf(0.5).abs() < 1e-9);
        assert!((norm_ppf(0.975) - 1.959963985).abs() < 1e-6);
        assert!((norm_ppf(0.001) + 3.090232306).abs() < 1e-6);
    }

    #[test]
    fn haircut_grows_with_trials_and_zero_for_one() {
        assert_eq!(expected_max_noise(1.0, 1.0), 0.0);
        assert!(expected_max_noise(1.0, 10.0) > expected_max_noise(1.0, 2.0));
    }

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        for _ in 0..100 {
            let x = a.next_f64();
            assert_eq!(x, b.next_f64());
            assert!((0.0..1.0).contains(&x));
        }
        assert!(a.range(3, 5) < 5);
    }

    #[test]
    fn ols_recovers_slope() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|v| 2.0 * v + 1.0 + if (*v as i32) % 2 == 0 { 0.1 } else { -0.1 })
            .collect();
        assert!((ols(&y, &x).0 - 2.0).abs() < 0.05);
        assert!((quantile_sorted(&[1.0, 2.0, 3.0, 4.0], 0.5) - 2.5).abs() < 1e-12);
    }
}
