use async_trait::async_trait;

use crate::{
    domain::model::{Signal, Stock},
    infra::data::moneyflow::MoneyflowEastmoney,
};

/// Repository for the main domain of candlescyther, it includes stock analysis related persistence.
/// READ/WRITE stocks, klines, signals, etc.
#[async_trait]
pub trait DomainRepository: Send + Sync {
    async fn create_stock(&self, stock: Stock) -> Result<(), anyhow::Error>;
    async fn get_stock(&self, ticker: &str) -> Result<Stock, anyhow::Error>;
    async fn get_stock_all(&self) -> Result<Vec<Stock>, anyhow::Error>;
    async fn delete_stocks(&self, tickers: &[&str]) -> Result<(), anyhow::Error>;

    // async fn create_klines(&self, ticker: &str, klines: &[Kline]) -> Result<(), anyhow::Error>;
    // async fn get_klines(&self, ticker: &str) -> Result<Vec<Kline>, anyhow::Error>;

    async fn create_signals_d(&self, signal: Signal) -> Result<(), anyhow::Error>;
    async fn create_signals_w(&self, signal: Signal) -> Result<(), anyhow::Error>;
    async fn get_signals_stock_d(&self) -> Result<Vec<Signal>, anyhow::Error>;
    async fn get_signals_stock_w(&self) -> Result<Vec<Signal>, anyhow::Error>;
    async fn get_signals_sector_d(&self) -> Result<Vec<Signal>, anyhow::Error>;
    async fn get_signals_sector_w(&self) -> Result<Vec<Signal>, anyhow::Error>;
    async fn delete_signals_d(&self) -> Result<(), anyhow::Error>;
    async fn delete_signals_w(&self) -> Result<(), anyhow::Error>;

    async fn create_mf_sector(&self, flows: &[MoneyflowEastmoney]) -> Result<(), anyhow::Error>;
    async fn get_mf_sector(&self) -> Result<Vec<MoneyflowEastmoney>, anyhow::Error>;
    async fn delete_mf_sector(&self) -> Result<(), anyhow::Error>;

    async fn get_sector_tickers(&self) -> Result<Vec<String>, anyhow::Error>;
    async fn get_stock_tickers(&self) -> Result<Vec<String>, anyhow::Error>;
}
