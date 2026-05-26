# Dataset Prediction Substrate

Purpose: define the general substrate that lets an agent use Focusa to make, evaluate, and improve predictions against **any dataset**, with stocks as the first likely domain.

Focusa should not become a stock model, data warehouse, or broker integration. Focusa should provide the agent-facing prediction substrate: normalized observations, ontology context, forecast records, outcome evaluation, calibration, and metacognitive learning.

## Core contract

Any dataset or set of feeds should become this loop:

```text
feed(s) → canonical observations → aligned feature set → forecast target → Focusa prediction → outcome/evidence → calibration → metacog learning → next prediction
```

The substrate must support both single-feed prediction and multi-feed prediction. Multiple feeds are aligned by entity, timestamp, metric, forecast target, and ontology context before the agent predicts.

## Canonical objects

### 1. Feed

A concrete stream, file, API, table, or manual import that produces observations.

```json
{
  "feed_id": "prices.yahoo_daily",
  "feed_type": "market_price",
  "dataset_id": "stocks.daily_prices",
  "entity_namespace": "ticker",
  "time_grain": "1d",
  "freshness": "current",
  "reliability_score": 0.92,
  "raw_ref": "data/raw/prices/yahoo_daily.parquet"
}
```

A feed can be prices, fundamentals, news, sentiment, macro indicators, options, analyst ratings, or any future domain data.

### 2. Dataset

A logical collection of one or more feeds normalized into a common entity/time/metric frame.

```json
{
  "dataset_id": "stocks.daily_prices",
  "feed_ids": ["prices.yahoo_daily", "prices.stooq_daily"],
  "dataset_type": "market_timeseries",
  "entity_namespace": "ticker",
  "time_grain": "1d",
  "source_ref": "data/stocks/daily_prices.parquet",
  "freshness": "current",
  "confidence": 0.95
}
```

### 3. Observation

One normalized fact from a feed/dataset.

```json
{
  "feed_id": "prices.yahoo_daily",
  "dataset_id": "stocks.daily_prices",
  "entity_key": "AAPL",
  "metric_key": "close",
  "observed_at": "2026-05-26T20:00:00Z",
  "value_num": 192.31,
  "confidence": 0.99,
  "evidence_ref": "stocks.daily_prices:AAPL:2026-05-26:close"
}
```

### 4. Feature

A computed predictive input. Features may come from one feed or from cross-feed fusion.

```json
{
  "feature_key": "return_5d",
  "entity_key": "AAPL",
  "feature_time": "2026-05-26T20:00:00Z",
  "value_num": 0.031,
  "lookback_window": "5d",
  "source_observation_refs": ["stocks.daily_prices:AAPL:2026-05-26:close"],
  "feed_mix": {"prices.yahoo_daily": 1.0}
}
```

### 5. Forecast target

The question being predicted.

```json
{
  "forecast_target_key": "stock_direction_5d",
  "entity_key": "AAPL",
  "question": "Will AAPL close higher 5 trading days from now?",
  "horizon_start": "2026-05-26T20:00:00Z",
  "horizon_end": "2026-06-02T20:00:00Z",
  "resolution_rule": "future_close > current_close"
}
```

### 6. Focusa prediction record

The agent-facing forecast.

```json
{
  "prediction_type": "dataset_forecast",
  "predicted_outcome": "AAPL higher in 5 trading days",
  "confidence": 0.61,
  "recommended_action": "watch; require confirmation from additional features before trade-like action",
  "why": "5d momentum positive, volatility moderate, sector trend supportive",
  "context_refs": ["forecast_target:stock_direction_5d:AAPL:2026-05-26"],
  "ontology_context": {
    "object_refs": ["Dataset:stocks.daily_prices", "Entity:ticker/AAPL", "ForecastTarget:stock_direction_5d"],
    "action_refs": ["forecast_dataset_target"],
    "tool_refs": ["focusa_predict_record", "focusa_metacog_retrieve"],
    "evidence_refs": ["feature:return_5d:AAPL:2026-05-26"],
    "relation_refs": ["feature_supports_forecast", "outcome_evaluates_prediction"]
  }
}
```

## Stock MVP

Start with a tiny stock-prediction domain because it has clear timestamps, entities, features, and outcomes.

### Minimal feeds

| Feed | Role |
| --- | --- |
| daily OHLCV prices | base market observations |
| adjusted close / splits | clean target resolution |
| benchmark index price | market regime context |
| sector/industry mapping | peer/regime grouping |
| fundamentals | slower explanatory features |
| news/sentiment | narrative/attention context |
| options/volatility | implied risk context |
| macro/interest-rate context | broad regime context |

### Minimal features

| Feature | Meaning |
| --- | --- |
| `return_1d`, `return_5d`, `return_20d` | momentum windows |
| `volatility_20d` | risk/noise |
| `volume_zscore_20d` | unusual attention/liquidity |
| `drawdown_60d` | stress context |
| `relative_strength_vs_spy_20d` | market-adjusted trend |
| `sector_relative_strength_20d` | peer context |

### First forecast targets

| Target | Resolution |
| --- | --- |
| `stock_direction_5d` | future adjusted close > current adjusted close |
| `stock_outperform_spy_20d` | ticker 20d return > SPY 20d return |
| `volatility_expansion_10d` | future realized volatility above current 20d volatility |

## Multi-feed fusion layer

The important substrate is not “many feeds dumped into context.” It is **feed fusion**: each feed becomes canonical observations, then compact features, then a forecast-ready evidence packet.

```text
FeedAdapter[]
  → CanonicalObservation[]
  → Entity/time alignment
  → FeatureSet
  → SourceAgreement / Contradiction / Freshness scores
  → ForecastEvidencePacket
  → focusa_predict_record
```

### ForecastEvidencePacket

```json
{
  "forecast_evidence_id": "forecast-evidence:AAPL:stock_direction_5d:2026-05-26",
  "forecast_target_key": "stock_direction_5d",
  "entity_key": "AAPL",
  "as_of": "2026-05-26T20:00:00Z",
  "feature_refs": [
    "feature:return_5d:AAPL:2026-05-26",
    "feature:relative_strength_vs_spy_20d:AAPL:2026-05-26",
    "feature:news_sentiment_3d:AAPL:2026-05-26"
  ],
  "feed_mix": {
    "prices.yahoo_daily": 0.45,
    "benchmark.spy_daily": 0.20,
    "news.sentiment_daily": 0.20,
    "options.iv_daily": 0.15
  },
  "source_agreement_score": 0.72,
  "contradiction_score": 0.18,
  "freshness_score": 0.94,
  "data_quality_score": 0.88,
  "top_supporting_signals": ["positive 5d momentum", "outperforming SPY", "sentiment improving"],
  "top_opposing_signals": ["implied volatility elevated"],
  "evidence_refs": ["features:AAPL:2026-05-26"]
}
```

The agent should see this compact packet, not raw rows.

### Fusion scores

| Score | Meaning |
| --- | --- |
| `source_agreement_score` | independent feeds point same direction |
| `contradiction_score` | feeds disagree or feature signs conflict |
| `freshness_score` | feed timestamps satisfy target horizon needs |
| `data_quality_score` | missing values, duplicates, stale feeds, source reliability |
| `regime_match_score` | prior lessons/examples match current conditions |

## Model role vs Focusa role

| Layer | Owned by |
| --- | --- |
| Feed loading | external script/library, current agent tool, or future connector |
| Feed normalization | external script/library or future connector using Focusa schema |
| Feature calculation | external script/library or future connector |
| Feed fusion / evidence packet | external script/library or future Focusa helper |
| Forecast probability generation | model/agent/ML code |
| Prediction record | Focusa `focusa_predict_record` |
| Outcome capture | Focusa `focusa_predict_evaluate` / `capture-outcome` |
| Calibration stats | Focusa `focusa_predict_stats` |
| Reusable learning | Focusa `focusa_metacog_*` |
| Object/action/evidence binding | Focusa ontology context |
| Continuity across runs | Focusa Workpoint/Trajectory |

## Required Focusa substrate additions

To make this strong across any dataset:

1. **Feed identity** — stable `feed_id`, source type, freshness, reliability, raw refs.
2. **Dataset identity** — stable `dataset_id`, entity namespace, time grain, freshness, confidence.
3. **Forecast target identity** — stable target key, entity, horizon, resolution rule.
4. **Feature refs** — compact handles for features used in each prediction.
5. **Forecast evidence packet** — feed mix, source agreement, contradictions, freshness, quality.
6. **Outcome resolver refs** — compact handles proving how the target resolved.
7. **Ontology context** — bind feed/dataset/entity/target/features/tools/evidence.
8. **Calibration groups** — stats by prediction type, feed mix, dataset, target, entity group, horizon, trajectory.
9. **Metacog promotion** — reusable lessons such as “5d momentum failed during high-volatility regime.”
10. **Guardrails** — predictions guide analysis; they are not financial advice or automatic trades.

## Agent workflow

```text
1. Identify feed(s), dataset, entity, and forecast target.
2. Normalize observations and compute features.
3. Fuse feeds into a compact ForecastEvidencePacket.
4. Retrieve prior lessons for that dataset/target/horizon/feed mix.
5. Record prediction with ontology context and evidence packet refs.
6. Wait until resolution horizon.
7. Capture outcome with resolver evidence.
8. Review calibration by dataset, target, horizon, and feed mix.
9. Promote reusable lesson via metacognition.
10. Record follow-up prediction with improved strategy.
```

## Generic API shape for future connector

This can be implemented later as API/CLI/Pi tools, but the schema is enough for agents now. The connector accepts one feed or many feeds and returns a compact forecast evidence packet plus prediction-ready ontology context:

```json
{
  "feed_ids": ["prices.yahoo_daily", "benchmark.spy_daily", "news.sentiment_daily"],
  "dataset_id": "stocks.multi_feed_market_context",
  "forecast_target_key": "stock_direction_5d",
  "entity_key": "AAPL",
  "horizon": "5d",
  "feature_refs": [
    "feature:return_5d:AAPL:2026-05-26",
    "feature:volatility_20d:AAPL:2026-05-26",
    "feature:news_sentiment_3d:AAPL:2026-05-26"
  ],
  "feed_mix": {"prices.yahoo_daily": 0.55, "benchmark.spy_daily": 0.25, "news.sentiment_daily": 0.20},
  "source_agreement_score": 0.72,
  "contradiction_score": 0.18,
  "probability": 0.61,
  "confidence": 0.68,
  "evidence_refs": ["features:AAPL:2026-05-26"],
  "ontology_context": {
    "object_refs": ["Dataset:stocks.multi_feed_market_context", "Feed:prices.yahoo_daily", "Feed:news.sentiment_daily", "Entity:ticker/AAPL"],
    "action_refs": ["forecast_dataset_target"],
    "tool_refs": ["focusa_predict_record"],
    "evidence_refs": ["features:AAPL:2026-05-26"],
    "relation_refs": ["dataset_feature_supports_prediction"]
  }
}
```

## Design rule

The substrate is powerful when predictions are **resolvable, calibrated, ontology-bound, and learned from**. Any dataset can be used if it can provide stable entities, timestamps, features, forecast targets, and outcome evidence.
