use std::io::BufRead;

use self::cli::{Args, LineFormat};

mod cli;

struct StreamResult {
    pub points: Vec<DataPoint>,
    pub min: f64,
    pub max: f64,
}

struct DataPoint {
    pub value: f64,
    pub count: usize,
}

fn main() {
    match process(Args::new()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("failed to process: {}", err);
            std::process::exit(1)
        }
    }
}

const MAX_DOT_COUNT: usize = 50;

fn process(args: Args) -> Result<(), String> {
    let data = stream(args.format)?;
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

fn stream(format: LineFormat) -> Result<StreamResult, String> {
    let mut points = vec![];
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    let stdin = std::io::stdin();
    for (idx, line) in stdin.lock().lines().enumerate() {
        let raw_line = line.map_err(|err| line_error(idx, err))?;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let point: DataPoint = match format {
            LineFormat::Single => {
                let value = trimmed.parse::<f64>().map_err(|err| line_error(idx, err))?;
                Ok::<DataPoint, String>(DataPoint { value, count: 1 })
            }
            LineFormat::KeyValue => {
                let (fst, snd) = tuple(trimmed).map_err(|err| line_error(idx, err))?;
                Ok(DataPoint {
                    value: fst,
                    count: snd as usize,
                })
            }
            LineFormat::ValueKey => {
                let (fst, snd) = tuple(trimmed).map_err(|err| line_error(idx, err))?;
                Ok(DataPoint {
                    value: snd,
                    count: fst as usize,
                })
            }
        }?;

        min = min.min(point.value);
        max = max.max(point.value);

        points.push(point)
    }

    Ok(StreamResult { points, min, max })
}

fn tuple(line: &str) -> Result<(f64, f64), String> {
    let mut splitter = line.split_whitespace();
    let fst = splitter
        .next()
        .ok_or_else(|| String::from("expecting two values"))?;
    let snd = splitter
        .next()
        .ok_or_else(|| String::from("expecting two values"))?;

    let fst_num = fst.parse::<f64>().map_err(|err| err.to_string())?;
    let snd_num = snd.parse::<f64>().map_err(|err| err.to_string())?;

    Ok((fst_num, snd_num))
}

fn line_error<T: ToString>(idx: usize, error: T) -> String {
    format!("line {}: {}", idx + 1, error.to_string())
}
