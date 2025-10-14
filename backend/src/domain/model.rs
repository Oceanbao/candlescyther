use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Kline {
    pub k_ticker: String,
    pub k_date: i64,
    pub k_open: f64,
    pub k_high: f64,
    pub k_low: f64,
    pub k_close: f64,
    pub k_volume: f64,
    pub k_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Signal {
    pub ticker: String,
    pub kdj_k: f64,
    pub kdj_d: f64,
}
