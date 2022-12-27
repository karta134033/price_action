use std::collections::VecDeque;

use chrono::NaiveDateTime;
use log::{info, warn};

use crate::{
    consts::TradeSide,
    types::{Kline, SettingConfig},
};

pub struct Backtest {
    look_back_count: usize,
    kline_percentage: f64,
    kline_history: VecDeque<Kline>,
    high: VecDeque<f64>,
    low: VecDeque<f64>,
    initial_captial: f64,
    entry_portion: f64,
    fee_rate: f64,
}

#[derive(Default)]
pub struct BacktestMetric {
    initial_captial: f64,
    usd_balance: f64,
    position: f64,
    entry_price: f64,
    entry_side: TradeSide,
    tp_price: f64, // take profit
    sl_price: f64, // stop loss
    win: usize,
    lose: usize,
    total_fee: f64,
    total_profit: f64,
    max_usd: f64,
    min_usd: f64,
    fee: f64,
    profit: f64,
}

impl BacktestMetric {
    pub fn init_trade(&mut self) {
        self.position = 0.;
        self.entry_price = 0.;
        self.tp_price = 0.;
        self.sl_price = 0.;
        self.entry_side = TradeSide::None;
    }
}

impl Backtest {
    pub fn new(config: SettingConfig) -> Backtest {
        Backtest {
            look_back_count: 20,
            kline_percentage: config.kline_percentage,
            kline_history: VecDeque::new(),
            high: VecDeque::new(),
            low: VecDeque::new(),
            initial_captial: config.initial_captial,
            entry_portion: config.entry_portion,
            fee_rate: config.fee_rate,
        }
    }

    pub fn add_history(&mut self, kline: Kline) {
        let mut highest: f64 = f64::MIN;
        let mut lowest: f64 = f64::MAX;

        self.high.clear();
        self.low.clear();
        self.kline_history.push_back(kline);
        for k in &self.kline_history {
            highest = highest.max(k.close);
            lowest = lowest.min(k.close);
            self.high.push_back(highest);
            self.low.push_back(lowest);
        }
        if self.kline_history.len() > self.look_back_count {
            self.kline_history.pop_front();
        }
    }

    pub fn run(&mut self, klines: Vec<Kline>) {
        let mut metric = BacktestMetric {
            usd_balance: self.initial_captial,
            initial_captial: self.initial_captial,
            ..Default::default()
        };
        for kline in klines {
            self.add_history(kline.clone());
            let kline_percentage = (kline.close - kline.open) / kline.open;
            if metric.entry_side == TradeSide::None {
                if self.higher_high()
                    && self.higher_low()
                    && kline_percentage >= self.kline_percentage
                {
                    let entry_price = kline.close;
                    let sl_price_diff = f64::abs(entry_price - self.low[self.low.len() - 1]);
                    metric.entry_price = entry_price;
                    metric.position = metric.usd_balance * self.entry_portion / entry_price;
                    metric.entry_side = TradeSide::Buy;
                    metric.sl_price = entry_price - sl_price_diff;
                    metric.tp_price = entry_price + 2. * sl_price_diff;
                    metric.fee = metric.entry_price * metric.position * self.fee_rate;
                    metric.total_fee += metric.fee;
                } else if self.lower_low()
                    && self.lower_high()
                    && kline_percentage <= self.kline_percentage * -1.
                {
                    let entry_price = kline.close;
                    let sl_price_diff = f64::abs(entry_price - self.high[self.high.len() - 1]);
                    metric.entry_price = entry_price;
                    metric.position = metric.usd_balance * self.entry_portion / entry_price;
                    metric.entry_side = TradeSide::Sell;
                    metric.sl_price = entry_price + sl_price_diff;
                    metric.tp_price = entry_price - 1.2 * sl_price_diff;
                    metric.fee = metric.entry_price * metric.position * self.fee_rate;
                    metric.total_fee += metric.fee;
                }
            } else if metric.position != 0. {
                if metric.entry_side == TradeSide::Buy {
                    if kline.low <= metric.sl_price {
                        let profit = (metric.tp_price - metric.entry_price) * metric.position;
                        metric.usd_balance += profit;
                        metric.win += 1;
                        metric.fee = metric.tp_price * metric.position * self.fee_rate;
                        metric.total_fee += metric.fee;
                        metric.profit = profit;
                        metric.total_profit += profit;
                        trade_log(&metric, &kline);
                        metric.init_trade();
                    } else if kline.high >= metric.tp_price {
                        let profit = (metric.sl_price - metric.entry_price) * metric.position;
                        metric.usd_balance += profit;
                        metric.lose += 1;
                        metric.fee = metric.sl_price * metric.position * self.fee_rate;
                        metric.total_fee += metric.fee;
                        metric.profit = profit;
                        metric.total_profit += profit;
                        trade_log(&metric, &kline);
                        metric.init_trade();
                    }
                } else if metric.entry_side == TradeSide::Sell {
                    if kline.high >= metric.sl_price {
                        let profit = (metric.entry_price - metric.sl_price) * metric.position;
                        metric.usd_balance += profit;
                        metric.lose += 1;
                        metric.fee = metric.sl_price * metric.position * self.fee_rate;
                        metric.total_fee += metric.fee;
                        metric.profit = profit;
                        metric.total_profit += profit;
                        trade_log(&metric, &kline);
                        metric.init_trade();
                    } else if kline.low <= metric.tp_price {
                        let profit = (metric.entry_price - metric.tp_price) * metric.position;
                        metric.usd_balance += profit;
                        metric.win += 1;
                        metric.fee = metric.tp_price * metric.position * self.fee_rate;
                        metric.total_fee += metric.fee;
                        metric.profit = profit;
                        metric.total_profit += profit;
                        trade_log(&metric, &kline);
                        metric.init_trade();
                    }
                }
            }
        }
    }

    pub fn higher_high(&mut self) -> bool {
        if self.high.len() >= self.look_back_count {
            self.high[self.look_back_count - 2] < self.high[self.look_back_count - 1]
        } else {
            false
        }
    }

    pub fn higher_low(&mut self) -> bool {
        if self.low.len() >= self.look_back_count {
            self.low[self.look_back_count - 2] == self.low[self.look_back_count - 1]
        } else {
            false
        }
    }

    pub fn lower_high(&mut self) -> bool {
        if self.high.len() >= self.look_back_count {
            self.high[self.look_back_count - 2] == self.high[self.look_back_count - 1]
        } else {
            false
        }
    }

    pub fn lower_low(&mut self) -> bool {
        if self.low.len() >= self.look_back_count {
            self.low[self.look_back_count - 2] > self.low[self.look_back_count - 1]
        } else {
            false
        }
    }
}

fn trade_log(metric: &BacktestMetric, curr_kline: &Kline) {
    let curr_date = NaiveDateTime::from_timestamp_millis(curr_kline.close_time).unwrap();
    let mut msg = "".to_string();
    msg += &format!("date: {:?}, ", curr_date);
    msg += &format!("win: {:?}, ", metric.win);
    msg += &format!("lose: {:?}, ", metric.lose);
    msg += &format!("usd_balance: {:.4}, ", metric.usd_balance);
    msg += &format!("position: {:.4}, ", metric.position);
    msg += &format!("entry_price: {:.4}, ", metric.entry_price);
    msg += &format!("profit: {:.4}, ", metric.profit);
    msg += &format!("fee: {:.4}, ", metric.fee);

    if metric.profit >= 0. {
        info!("{}", msg);
    } else {
        warn!("{}", msg);
    }
}
