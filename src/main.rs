use std::io::BufRead;
use std::str::FromStr;

use argh::FromArgs;

#[derive(Debug, PartialEq)]
enum LineFormat {
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

struct StreamResult {
    pub points: Vec<DataPoint>,
    pub min: f64,
    pub max: f64,
}

struct DataPoint {
    pub value: f64,
    pub count: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "histogram from the command line")]
struct Args {
    #[argh(option, description = "number of buckets", short = 'b', default = "10")]
    buckets: i32,
    #[argh(
        option,
        description = "line format (default: single)",
        short = 'f',
        default = "LineFormat::Single"
    )]
    format: LineFormat,
    #[argh(option, description = "minimum value")]
    min: Option<f64>,
    #[argh(option, description = "maximum value")]
    max: Option<f64>,
}

fn main() {
    let parsed = argh::from_env();
    match process(parsed) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("failed to process: {}", err);
            std::process::exit(1)
        }
    }
}

const MAX_DOT_COUNT: usize = 50;

fn process(args: Args) -> Result<(), String> {
    let data = stream()?;
    if data.points.is_empty() {
        return Err(String::from("empty input"));
    }

    let minimum = args.min.unwrap_or(data.min);
    let maximum = args.max.unwrap_or(data.max);

    let diff = maximum - minimum;

    let buckets = args.buckets;
    let step = diff / (buckets as f64);

    let mut excluded = 0usize;
    let mut boundaries = vec![];
    let mut bucket_counts = vec![0; buckets as usize];

    for idx in 0..buckets {
        boundaries.push(minimum + (step * (idx as f64 + 1.0)));
    }

    let last_bucket = bucket_counts.len() - 1;
    let get_bucket = |val: f64| {
        for (idx, boundary) in boundaries.iter().enumerate() {
            if val <= *boundary {
                return idx;
            }
        }
        last_bucket
    };

    for point in data.points {
        // check min/max
        if point.value < minimum || point.value > maximum {
            excluded += point.count;
            continue;
        }
        bucket_counts[get_bucket(point.value)] += point.count;
    }

    let max_bucket_count = bucket_counts.iter().max().unwrap_or(&0usize).clone();
    let bucket_scale = if max_bucket_count > MAX_DOT_COUNT {
        max_bucket_count / MAX_DOT_COUNT
    } else {
        1
    };

    println!("# each * represents a count of {}", bucket_scale);

    if excluded > 0 {
        println!("# excluded {} value(s) based on min/max", excluded);
    }

    let mut bucket_min;
    let mut bucket_max = minimum;
    let mut table = tabular::Table::new("{:>} - {:>} [{:>}] {:<}");

    for bucket in 0..buckets as usize {
        bucket_min = bucket_max;
        bucket_max = boundaries[bucket];
        let count = bucket_counts[bucket];
        let dots = if count > 0 { count / bucket_scale } else { 0 };

        table.add_row(tabular::row!(
            num_fmt(bucket_min, 2),
            num_fmt(bucket_max, 2),
            count,
            "*".repeat(dots)
        ));
    }

    print!("{}", table);

    Ok(())
}

#[inline]
fn num_fmt(value: f64, precision: usize) -> String {
    format!("{:.precision$}", value)
}

fn stream() -> Result<StreamResult, String> {
    let mut points = vec![];
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let raw_line = line.map_err(str_error)?;
        if raw_line.is_empty() {
            continue;
        }

        let value = raw_line.parse::<f64>().map_err(str_error)?;

        min = min.min(value);
        max = max.max(value);

        points.push(DataPoint { value, count: 1 })
    }

    Ok(StreamResult { points, min, max })
}

fn str_error<T: ToString>(error: T) -> String {
    error.to_string()
}
