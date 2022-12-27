use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(arg_required_else_help = false)]
pub struct Cli {
    #[arg(short = 'c')]
    pub config_path: PathBuf,
    #[arg(short = 'm')]
    pub mode: Mode,
}

#[derive(Clone, Debug)]
pub enum Mode {
    Backtest,
    Hypertune,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backtest" => Ok(Mode::Backtest),
            "hypertune" => Ok(Mode::Hypertune),
            "b" => Ok(Mode::Backtest),
            "h" => Ok(Mode::Hypertune),
            _ => Err(format!("Invalid mode: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kline {
    pub open_time: i64,
    pub close_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingConfig {
    pub from: String,
    pub to: String,
    pub initial_captial: f64,
    pub fee_rate: f64,
    pub kline_percentage: f64,
    pub entry_portion: f64,
}
