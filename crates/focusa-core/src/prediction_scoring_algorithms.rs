//! Deterministic algorithms for the Spec138 scorer registry.

use crate::prediction_scoring::{ScoreInput, ScorerId, ScoringError};

pub fn score(id: ScorerId, input: &ScoreInput) -> Result<f64, ScoringError> {
    use ScorerId::*;
    match id {
        BinaryAccuracy => binary(input, |p, y| Ok(f64::from((p >= 0.5) == y))),
        BrierScore => binary(input, |p, y| Ok((p - f64::from(y)).powi(2))),
        LogLoss => binary(input, |p, y| {
            Ok(if y {
                -safe_probability(p)?.ln()
            } else {
                -safe_probability(1.0 - p)?.ln()
            })
        }),
        MulticlassAccuracy => categorical(input, |p, y| Ok(f64::from(argmax(p)? == y))),
        MulticlassBrierScore => categorical(input, |p, y| {
            Ok(p.iter()
                .enumerate()
                .map(|(i, v)| (v - f64::from(i == y)).powi(2))
                .sum())
        }),
        MulticlassLogLoss => categorical(input, |p, y| Ok(-safe_probability(p[y])?.ln())),
        SphericalScore => categorical(input, |p, y| {
            let norm = p.iter().map(|v| v * v).sum::<f64>().sqrt();
            if norm == 0.0 {
                Err(ScoringError::DivisionByZero)
            } else {
                Ok(p[y] / norm)
            }
        }),
        ContinuousRankedProbabilityScore => samples(input),
        MeanAbsoluteError => numeric(input, |f, a| (f - a).abs(), false),
        MeanSquaredError => numeric(input, |f, a| (f - a).powi(2), false),
        RootMeanSquaredError => numeric(input, |f, a| (f - a).powi(2), true),
        MeanAbsolutePercentageError => numeric_percentage(input, false),
        SymmetricMape => numeric_percentage(input, true),
        QuantilePinballLoss => quantile(input),
        IntervalCoverage => interval(input, 0),
        IntervalWidth => interval(input, 1),
        WinklerIntervalScore => interval(input, 2),
        RankCorrelation | InformationCoefficient => ranking_correlation(input),
        TopKPrecision => top_k(input, true),
        TopKRecall => top_k(input, false),
        Ndcg => ndcg(input),
        ConcordanceIndex => concordance(input),
        SurvivalBrierScore => survival(input),
        ExpectedCalibrationError => calibration(input, 0),
        MaximumCalibrationError => calibration(input, 1),
        AdaptiveCalibrationError => calibration(input, 2),
        SkillScore => baseline(input),
        ExpectedUtility => utility(input, false),
        RealizedRegret => utility(input, true),
        CustomRegistered => Err(ScoringError::CustomScorerRequiresRegistration),
    }
}

fn binary(
    input: &ScoreInput,
    f: impl FnOnce(f64, bool) -> Result<f64, ScoringError>,
) -> Result<f64, ScoringError> {
    let ScoreInput::Binary {
        probability,
        outcome,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    validate_probability(*probability)?;
    f(*probability, *outcome)
}
fn categorical(
    input: &ScoreInput,
    f: impl FnOnce(&[f64], usize) -> Result<f64, ScoringError>,
) -> Result<f64, ScoringError> {
    let ScoreInput::Categorical {
        probabilities,
        outcome_index,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    validate_distribution(probabilities)?;
    if *outcome_index >= probabilities.len() {
        return Err(ScoringError::InvalidOutcome);
    }
    f(probabilities, *outcome_index)
}
fn samples(input: &ScoreInput) -> Result<f64, ScoringError> {
    let ScoreInput::Samples {
        forecast_samples,
        actual,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    if forecast_samples.is_empty() {
        return Err(ScoringError::EmptyInput);
    }
    let n = forecast_samples.len() as f64;
    let first = forecast_samples
        .iter()
        .map(|x| (x - actual).abs())
        .sum::<f64>()
        / n;
    let pair = forecast_samples
        .iter()
        .flat_map(|x| forecast_samples.iter().map(move |y| (x - y).abs()))
        .sum::<f64>()
        / (n * n);
    Ok(first - 0.5 * pair)
}
fn numeric(
    input: &ScoreInput,
    f: impl Fn(f64, f64) -> f64,
    root: bool,
) -> Result<f64, ScoringError> {
    let ScoreInput::NumericSeries { forecasts, actuals } = input else {
        return Err(ScoringError::ShapeMismatch);
    };
    pairs(forecasts, actuals)?;
    let value = forecasts
        .iter()
        .zip(actuals)
        .map(|(x, y)| f(*x, *y))
        .sum::<f64>()
        / forecasts.len() as f64;
    Ok(if root { value.sqrt() } else { value })
}
fn numeric_percentage(input: &ScoreInput, symmetric: bool) -> Result<f64, ScoringError> {
    let ScoreInput::NumericSeries { forecasts, actuals } = input else {
        return Err(ScoringError::ShapeMismatch);
    };
    pairs(forecasts, actuals)?;
    let mut total = 0.0;
    for (f, a) in forecasts.iter().zip(actuals) {
        let d = if symmetric {
            (f.abs() + a.abs()) / 2.0
        } else {
            a.abs()
        };
        if d == 0.0 {
            return Err(ScoringError::DivisionByZero);
        }
        total += (f - a).abs() / d;
    }
    Ok(total / forecasts.len() as f64)
}
fn quantile(input: &ScoreInput) -> Result<f64, ScoringError> {
    let ScoreInput::Quantile {
        quantile,
        forecast,
        actual,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    if !(*quantile > 0.0 && *quantile < 1.0) {
        return Err(ScoringError::InvalidParameter);
    }
    let e = actual - forecast;
    Ok(if e >= 0.0 {
        quantile * e
    } else {
        (quantile - 1.0) * e
    })
}
fn interval(input: &ScoreInput, mode: u8) -> Result<f64, ScoringError> {
    let ScoreInput::Interval {
        lower,
        upper,
        alpha,
        actual,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    if lower > upper || !(*alpha > 0.0 && *alpha < 1.0) {
        return Err(ScoringError::InvalidParameter);
    }
    match mode {
        0 => Ok(f64::from(actual >= lower && actual <= upper)),
        1 => Ok(upper - lower),
        _ => Ok((upper - lower)
            + if actual < lower {
                2.0 / alpha * (lower - actual)
            } else if actual > upper {
                2.0 / alpha * (actual - upper)
            } else {
                0.0
            }),
    }
}
fn ranking(input: &ScoreInput) -> Result<(&[f64], &[f64], usize), ScoringError> {
    let ScoreInput::Ranking {
        scores,
        relevance,
        k,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    pairs(scores, relevance)?;
    if *k == 0 || *k > scores.len() {
        return Err(ScoringError::InvalidParameter);
    }
    Ok((scores, relevance, *k))
}
fn ranking_correlation(input: &ScoreInput) -> Result<f64, ScoringError> {
    let (s, r, _) = ranking(input)?;
    let rs = ranks(s);
    let rr = ranks(r);
    let n = s.len() as f64;
    if n < 2.0 {
        return Err(ScoringError::InvalidParameter);
    }
    let d = rs.iter().zip(rr).map(|(a, b)| (a - b).powi(2)).sum::<f64>();
    Ok(1.0 - 6.0 * d / (n * (n * n - 1.0)))
}
fn top_k(input: &ScoreInput, precision: bool) -> Result<f64, ScoringError> {
    let (s, r, k) = ranking(input)?;
    let mut idx = (0..s.len()).collect::<Vec<_>>();
    idx.sort_by(|a, b| s[*b].total_cmp(&s[*a]));
    let hits = idx[..k].iter().filter(|i| r[**i] > 0.0).count() as f64;
    if precision {
        Ok(hits / k as f64)
    } else {
        let total = r.iter().filter(|v| **v > 0.0).count();
        if total == 0 {
            Err(ScoringError::DivisionByZero)
        } else {
            Ok(hits / total as f64)
        }
    }
}
fn ndcg(input: &ScoreInput) -> Result<f64, ScoringError> {
    let (s, r, k) = ranking(input)?;
    let mut idx = (0..s.len()).collect::<Vec<_>>();
    idx.sort_by(|a, b| s[*b].total_cmp(&s[*a]));
    let dcg = idx[..k]
        .iter()
        .enumerate()
        .map(|(rank, i)| (2f64.powf(r[*i]) - 1.0) / ((rank + 2) as f64).log2())
        .sum::<f64>();
    let mut ideal = r.to_vec();
    ideal.sort_by(|a, b| b.total_cmp(a));
    let idcg = ideal[..k]
        .iter()
        .enumerate()
        .map(|(rank, v)| (2f64.powf(*v) - 1.0) / ((rank + 2) as f64).log2())
        .sum::<f64>();
    if idcg == 0.0 {
        Err(ScoringError::DivisionByZero)
    } else {
        Ok(dcg / idcg)
    }
}
fn concordance(input: &ScoreInput) -> Result<f64, ScoringError> {
    let (s, r, _) = ranking(input)?;
    let mut good = 0.0;
    let mut total = 0.0;
    for i in 0..s.len() {
        for j in i + 1..s.len() {
            if r[i] != r[j] {
                total += 1.0;
                if (s[i] - s[j]) * (r[i] - r[j]) > 0.0 {
                    good += 1.0
                } else if s[i] == s[j] {
                    good += 0.5
                }
            }
        }
    }
    if total == 0.0 {
        Err(ScoringError::DivisionByZero)
    } else {
        Ok(good / total)
    }
}
fn survival(input: &ScoreInput) -> Result<f64, ScoringError> {
    let ScoreInput::Survival {
        probabilities,
        outcomes,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    if probabilities.len() != outcomes.len() || probabilities.is_empty() {
        return Err(ScoringError::LengthMismatch);
    }
    for p in probabilities {
        validate_probability(*p)?
    }
    Ok(probabilities
        .iter()
        .zip(outcomes)
        .map(|(p, y)| (p - f64::from(*y)).powi(2))
        .sum::<f64>()
        / probabilities.len() as f64)
}
fn calibration(input: &ScoreInput, mode: u8) -> Result<f64, ScoringError> {
    let ScoreInput::Calibration {
        probabilities,
        outcomes,
        bucket_count,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    if probabilities.len() != outcomes.len() || probabilities.is_empty() || *bucket_count == 0 {
        return Err(ScoringError::InvalidParameter);
    }
    for p in probabilities {
        validate_probability(*p)?
    }
    let mut bins = vec![Vec::new(); *bucket_count];
    for (i, p) in probabilities.iter().enumerate() {
        let b = ((p * (*bucket_count as f64)) as usize).min(*bucket_count - 1);
        bins[b].push(i)
    }
    let mut gaps = Vec::new();
    for bin in bins.iter().filter(|b| !b.is_empty()) {
        let confidence = bin.iter().map(|i| probabilities[*i]).sum::<f64>() / bin.len() as f64;
        let accuracy = bin.iter().map(|i| f64::from(outcomes[*i])).sum::<f64>() / bin.len() as f64;
        gaps.push(((confidence - accuracy).abs(), bin.len()))
    }
    match mode {
        0 => Ok(gaps
            .iter()
            .map(|(g, n)| g * (*n as f64) / probabilities.len() as f64)
            .sum()),
        1 => Ok(gaps.iter().map(|(g, _)| *g).fold(0.0, f64::max)),
        _ => Ok(gaps.iter().map(|(g, _)| *g).sum::<f64>() / gaps.len() as f64),
    }
}
fn baseline(input: &ScoreInput) -> Result<f64, ScoringError> {
    let ScoreInput::Baseline {
        score,
        baseline_score,
        lower_is_better,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    if *baseline_score == 0.0 {
        return Err(ScoringError::DivisionByZero);
    }
    Ok(if *lower_is_better {
        1.0 - score / baseline_score
    } else {
        score / baseline_score - 1.0
    })
}
fn utility(input: &ScoreInput, regret: bool) -> Result<f64, ScoringError> {
    let ScoreInput::Utility {
        expected,
        realized,
        best_available,
    } = input
    else {
        return Err(ScoringError::ShapeMismatch);
    };
    Ok(if regret {
        best_available - realized
    } else {
        *expected
    })
}
fn validate_probability(p: f64) -> Result<(), ScoringError> {
    if p.is_finite() && (0.0..=1.0).contains(&p) {
        Ok(())
    } else {
        Err(ScoringError::InvalidProbability)
    }
}
fn safe_probability(p: f64) -> Result<f64, ScoringError> {
    validate_probability(p)?;
    Ok(p.clamp(1e-15, 1.0 - 1e-15))
}
fn validate_distribution(p: &[f64]) -> Result<(), ScoringError> {
    if p.is_empty() {
        return Err(ScoringError::EmptyInput);
    }
    for v in p {
        validate_probability(*v)?
    }
    if (p.iter().sum::<f64>() - 1.0).abs() > 1e-9 {
        Err(ScoringError::InvalidDistribution)
    } else {
        Ok(())
    }
}
fn pairs(a: &[f64], b: &[f64]) -> Result<(), ScoringError> {
    if a.is_empty() {
        Err(ScoringError::EmptyInput)
    } else if a.len() != b.len() {
        Err(ScoringError::LengthMismatch)
    } else {
        Ok(())
    }
}
fn argmax(values: &[f64]) -> Result<usize, ScoringError> {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .ok_or(ScoringError::EmptyInput)
}
fn ranks(values: &[f64]) -> Vec<f64> {
    let mut idx = (0..values.len()).collect::<Vec<_>>();
    idx.sort_by(|a, b| values[*a].total_cmp(&values[*b]));
    let mut out = vec![0.0; values.len()];
    for (rank, i) in idx.into_iter().enumerate() {
        out[i] = rank as f64 + 1.0
    }
    out
}
