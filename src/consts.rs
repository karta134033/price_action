pub const BTCUSDT_15M: &str = "BTCUSDT_15m";
pub const KLINE_DB: &str = "klines";
pub const LOCAL_MONGO_CONNECTION_STRING: &str = "mongodb://localhost:27017";

#[derive(Debug, PartialEq, Clone)]
pub enum TradeSide {
    Sell,
    Buy,
    Stop,
    None,
}

impl TradeSide {
    pub fn value(&self) -> f64 {
        match *self {
            TradeSide::Sell => -1.,
            TradeSide::Buy => 1.,
            TradeSide::Stop | TradeSide::None => 0.,
        }
    }
}

impl Default for TradeSide {
    fn default() -> Self {
        TradeSide::None
    }
}
