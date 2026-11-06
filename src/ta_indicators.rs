//! TA-Lib indicator adapters that align output values with their input bars.

use ta_lib::{Core, RetCode};

fn aligned_output(
    len: usize,
    calculate: impl FnOnce(&mut [f64]) -> Result<(usize, usize), RetCode>,
) -> Result<Vec<f64>, RetCode> {
    if len == 0 {
        return Ok(Vec::new());
    }

    let mut raw = vec![0.0; len];
    let (begin, count) = calculate(&mut raw)?;
    if begin > len || count > len - begin {
        return Err(RetCode::BadParam);
    }

    let mut aligned = vec![f64::NAN; len];
    aligned[begin..begin + count].copy_from_slice(&raw[..count]);
    Ok(aligned)
}

fn period_as_i32(period: usize) -> Result<i32, RetCode> {
    i32::try_from(period).map_err(|_| RetCode::BadParam)
}

/// TA-Lib simple moving average, with warm-up values represented as NaN.
pub fn sma(values: &[f64], period: usize) -> Result<Vec<f64>, RetCode> {
    let period = period_as_i32(period)?;
    let core = Core::new();
    core.sma_lookback(period)?;
    let Some(end) = values.len().checked_sub(1) else {
        return Ok(Vec::new());
    };
    aligned_output(values.len(), |out| {
        let range = core.sma(0, end, values, period, out)?;
        Ok((range.beg_idx, range.count))
    })
}

/// TA-Lib Wilder RSI, with warm-up values represented as NaN.
pub fn rsi(close: &[f64], period: usize) -> Result<Vec<f64>, RetCode> {
    let period = period_as_i32(period)?;
    let core = Core::new();
    core.rsi_lookback(period)?;
    if close.is_empty() {
        return Ok(Vec::new());
    }
    let mut aligned = vec![f64::NAN; close.len()];
    let mut start = 0;
    while start < close.len() {
        while start < close.len() && !close[start].is_finite() {
            start += 1;
        }
        let segment_start = start;
        while start < close.len() && close[start].is_finite() {
            start += 1;
        }
        if segment_start == start {
            continue;
        }

        let segment = &close[segment_start..start];
        let end = segment.len() - 1;
        let values = aligned_output(segment.len(), |out| {
            let range = core.rsi(0, end, segment, period, out)?;
            Ok((range.beg_idx, range.count))
        })?;
        aligned[segment_start..start].copy_from_slice(&values);
    }
    Ok(aligned)
}

/// TA-Lib Wilder ATR, with warm-up values represented as NaN.
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Result<Vec<f64>, RetCode> {
    let period = period_as_i32(period)?;
    if high.len() != low.len() || high.len() != close.len() {
        return Err(RetCode::BadParam);
    }
    let core = Core::new();
    core.atr_lookback(period)?;
    let Some(end) = close.len().checked_sub(1) else {
        return Ok(Vec::new());
    };
    aligned_output(close.len(), |out| {
        let range = core.atr(0, end, high, low, close, period, out)?;
        Ok((range.beg_idx, range.count))
    })
}

#[cfg(test)]
mod tests {
    use super::{atr, rsi, sma, RetCode};

    #[test]
    fn sma_aligns_warmup_to_input_indices() {
        let values = sma(&[1.0, 2.0, 3.0, 4.0], 3).unwrap();
        assert!(values[0].is_nan() && values[1].is_nan());
        assert_eq!(&values[2..], &[2.0, 3.0]);
    }

    #[test]
    fn rsi_matches_ta_lib_wilder_seed_and_alignment() {
        let values = rsi(&[10.0, 11.0, 12.0, 10.0], 2).unwrap();
        assert!(values[0].is_nan() && values[1].is_nan());
        assert!((values[2] - 100.0).abs() < 1e-12);
        assert!((values[3] - (100.0 / 3.0)).abs() < 1e-12);
    }

    #[test]
    fn rsi_uses_ta_lib_neutral_value_for_flat_prices() {
        let values = rsi(&[100.0; 4], 2).unwrap();
        assert!(values[0].is_nan() && values[1].is_nan());
        assert_eq!(&values[2..], &[50.0, 50.0]);
    }

    #[test]
    fn rsi_restarts_after_non_finite_prices() {
        let values = rsi(&[10.0, 11.0, 12.0, f64::NAN, 8.0, 9.0, 10.0], 2).unwrap();
        assert!((values[2] - 100.0).abs() < 1e-12);
        assert!(values[3].is_nan() && values[4].is_nan() && values[5].is_nan());
        assert!((values[6] - 100.0).abs() < 1e-12);
    }

    #[test]
    fn rsi_rejects_invalid_period_even_when_input_is_empty() {
        assert_eq!(rsi(&[], 1).unwrap_err(), RetCode::BadParam);
    }

    #[test]
    fn atr_matches_ta_lib_seed_and_alignment() {
        let values = atr(&[2.0, 3.0, 4.0], &[1.0, 1.0, 2.0], &[1.5, 2.0, 3.0], 2).unwrap();
        assert!(values[0].is_nan() && values[1].is_nan());
        assert!((values[2] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn atr_rejects_mismatched_input_lengths() {
        assert!(atr(&[2.0], &[], &[1.5], 2).is_err());
    }
}

/// TA-Lib-convention EMA (SMA-seeded after `n` consecutive finite values; NaN
/// warm-up; a non-finite input leaves state unchanged).
pub fn ema(x: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; x.len()];
    if n == 0 {
        return out;
    }
    let k = 2.0 / (n as f64 + 1.0);
    let (mut run, mut sum, mut state) = (0usize, 0.0, f64::NAN);
    for (i, &v) in x.iter().enumerate() {
        if !v.is_finite() {
            if state.is_nan() {
                run = 0;
                sum = 0.0;
            }
            continue;
        }
        if state.is_nan() {
            run += 1;
            sum += v;
            if run == n {
                state = sum / n as f64;
                out[i] = state;
            }
        } else {
            state = k * v + (1.0 - k) * state;
            out[i] = state;
        }
    }
    out
}

/// Median of the `window` values ending the session BEFORE `day` (no NaN in
/// the sample allowed).
pub fn median_before(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day < window {
        return f64::NAN;
    }
    let mut sample = values[day - window..day].to_vec();
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.sort_by(f64::total_cmp);
    let middle = sample.len() / 2;
    if sample.len() % 2 == 0 {
        (sample[middle - 1] + sample[middle]) / 2.0
    } else {
        sample[middle]
    }
}

/// Mean of the `window` values ending the session BEFORE `day`.
pub fn mean_before(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day < window {
        return f64::NAN;
    }
    let sample = &values[day - window..day];
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.iter().sum::<f64>() / window as f64
}

/// Median of the `window` values ending AT `day` (inclusive).
pub fn median_at(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day + 1 < window {
        return f64::NAN;
    }
    let mut sample = values[day + 1 - window..=day].to_vec();
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.sort_by(f64::total_cmp);
    let middle = sample.len() / 2;
    if sample.len() % 2 == 0 {
        (sample[middle - 1] + sample[middle]) / 2.0
    } else {
        sample[middle]
    }
}

/// Mean of the `window` values ending AT `day` (inclusive).
pub fn mean_at(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day + 1 < window {
        return f64::NAN;
    }
    let sample = &values[day + 1 - window..=day];
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.iter().sum::<f64>() / window as f64
}
