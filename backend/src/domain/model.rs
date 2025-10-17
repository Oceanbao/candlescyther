use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Stock {
    pub ticker: String,
    pub realname: String,
    pub market: i64,
    pub total_cap: Option<f64>,
    pub pe: Option<f64>,
    pub pb: Option<f64>,
    pub revenue: Option<f64>,
    pub net: Option<f64>,
    pub margin: Option<f64>,
    pub debt: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Signal {
    pub ticker: String,
    pub kdj_k: f64,
    pub kdj_d: f64,
}
