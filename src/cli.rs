use std::str::FromStr;

use argh::FromArgs;

#[derive(Debug, PartialEq)]
pub enum LineFormat {
    Single,
    KeyValue,
    ValueKey,
}

impl FromStr for LineFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single" => Ok(LineFormat::Single),
            "kv" => Ok(LineFormat::KeyValue),
            "vk" => Ok(LineFormat::ValueKey),
            _ => Err(format!("unknown line format: '{}'", s)),
        }
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "data analysis from the command line")]
pub struct Args {
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    HistCommand(HistArgs),
    BarCommand(BarArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "bar",
    description = "show bar diagram of input values"
)]
pub struct BarArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "histogram",
    description = "show histogram of input values"
)]
pub struct HistArgs {
    #[argh(option, description = "number of buckets", short = 'b', default = "10")]
    pub buckets: i32,
    #[argh(
        option,
        description = "line format (default: single)",
        short = 'f',
        default = "LineFormat::Single"
    )]
    pub format: LineFormat,
    #[argh(option, description = "minimum value")]
    pub min: Option<f64>,
    #[argh(option, description = "maximum value")]
    pub max: Option<f64>,
    #[argh(switch, short = 'q', description = "disable mean/std/var calculation")]
    pub quiet: bool,
    #[argh(switch, description = "use log scale")]
    pub log: bool,
}

impl Args {
    pub fn new() -> Args {
        argh::from_env()
    }
}
