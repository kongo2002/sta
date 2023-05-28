use std::collections::HashMap;
use std::io::BufRead;

use self::cli::{Args, BarArgs, HistArgs, LineFormat};

mod cli;
mod stats;

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
    let result = match Args::new().command {
        cli::Command::HistCommand(args) => {
            if args.buckets <= 0 {
                bail_out("buckets must be positive");
            }

            histogram(args)
        }
        cli::Command::BarCommand(args) => bar(args),
    };

    match result {
        Ok(()) => {}
        Err(err) => {
            bail_out(format!("failed to process: {}", err).as_str());
        }
    }
}

const MAX_DOT_COUNT: usize = 50;

fn bar(args: BarArgs) -> Result<(), String> {
    let values = stream_unique_values(args.format)?;
    if values.is_empty() {
        return Err(String::from("empty input"));
    }

    let max_count = values
        .iter()
        .map(|(_, v)| v)
        .max()
        .unwrap_or(&0usize)
        .clone();

    let scale = if max_count > MAX_DOT_COUNT {
        max_count / MAX_DOT_COUNT
    } else {
        1
    };

    println!("# each ∎ represents a count of {}", scale);

    let mut table = tabular::Table::new("{:>} [{:>}] {:<}");

    for (key, count) in values {
        let dots = if count > 0 { count / scale } else { 0 };

        table.add_row(tabular::row!(key, count, "∎".repeat(dots)));
    }

    print!("{}", table);
    Ok(())
}

fn histogram(args: HistArgs) -> Result<(), String> {
    let data = stream_data_points(args.format)?;
    if data.points.is_empty() {
        return Err(String::from("empty input"));
    }

    let minimum = args.min.unwrap_or(data.min);
    let maximum = args.max.unwrap_or(data.max);

    let diff = maximum - minimum;

    let buckets = args.buckets;

    let mut mvsd = stats::MVSD::new();
    let mut samples = 0usize;
    let mut excluded = 0usize;
    let mut bucket_counts = vec![0; buckets as usize];

    let boundaries = if args.log {
        log_scale_buckets(buckets, minimum, diff)
    } else {
        linear_scale_buckets(buckets, minimum, diff)
    };

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
        if point.count <= 0 {
            continue;
        }

        samples += point.count;
        if !args.quiet {
            mvsd.add(point.value, point.count as f64);
        }

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

    println!("# samples: {}; min: {}; max: {}", samples, minimum, maximum);

    if !args.quiet {
        println!(
            "# mean: {:.2}; var: {:.2}; sd: {:.2}, median: {:.2}",
            mvsd.mean(),
            mvsd.var(),
            mvsd.sd(),
            mvsd.median()
        );
    }

    println!("# each ∎ represents a count of {}", bucket_scale);

    if excluded > 0 {
        println!("# excluded {} value(s) based on min/max", excluded);
    }

    let mut bucket_min;
    let mut bucket_max = minimum;
    let mut table = tabular::Table::new("{:>} - {:>} [{:>}] {:<}");

    let precision = if args.log {
        if minimum <= 10.0 {
            2
        } else if minimum <= 25.0 {
            1
        } else {
            0
        }
    } else if diff <= 3.0 {
        3
    } else if diff <= 10.0 {
        2
    } else if diff <= 25.0 {
        1
    } else {
        0
    };

    for bucket in 0..buckets as usize {
        bucket_min = bucket_max;
        bucket_max = boundaries[bucket];
        let count = bucket_counts[bucket];
        let dots = if count > 0 { count / bucket_scale } else { 0 };

        table.add_row(tabular::row!(
            num_fmt(bucket_min, precision),
            num_fmt(bucket_max, precision),
            count,
            "∎".repeat(dots)
        ));
    }

    print!("{}", table);

    Ok(())
}

fn log_scale_buckets(buckets: i32, min: f64, diff: f64) -> Vec<f64> {
    const BASE: u32 = 2;
    let first_bucket_size = diff / ((BASE.pow(buckets as u32) - 1) as f64);
    let mut boundaries = vec![];
    let mut sum = 0f64;

    for idx in 0..buckets as u32 {
        sum += BASE.pow(idx) as f64 * first_bucket_size;
        boundaries.push(min + sum);
    }

    boundaries
}

fn linear_scale_buckets(buckets: i32, min: f64, diff: f64) -> Vec<f64> {
    let mut boundaries = vec![];
    let step = diff / (buckets as f64);

    for idx in 0..buckets {
        boundaries.push(min + (step * (idx as f64 + 1.0)));
    }
    boundaries
}

#[inline]
fn num_fmt(value: f64, precision: usize) -> String {
    format!("{:.precision$}", value)
}

fn process_lines<T: FnMut(&str, usize) -> Result<(), String>>(mut func: T) -> Result<(), String> {
    let stdin = std::io::stdin();
    for (idx, line) in stdin.lock().lines().enumerate() {
        let raw_line = line.map_err(|err| line_error(idx, err))?;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            continue;
        }

        func(trimmed, idx)?;
    }

    Ok(())
}

fn stream_unique_values(format: LineFormat) -> Result<HashMap<String, usize>, String> {
    let mut map = HashMap::new();

    process_lines(|trimmed, idx| {
        let (key, count): (String, usize) = match format {
            LineFormat::Single => Ok::<(String, usize), String>((trimmed.to_string(), 1usize)),
            LineFormat::KeyValue => {
                let (fst, snd) = tuple(trimmed).map_err(|err| line_error(idx, err))?;

                let count = snd.parse::<f64>().map_err(|err| line_error(idx, err))?;
                Ok((fst.to_string(), count as usize))
            }
            LineFormat::ValueKey => {
                let (fst, snd) = tuple(trimmed).map_err(|err| line_error(idx, err))?;
                let count = fst.parse::<f64>().map_err(|err| line_error(idx, err))?;
                Ok((snd.to_string(), count as usize))
            }
        }?;

        *map.entry(key).or_insert(0) += count;

        Ok(())
    })?;

    Ok(map)
}

fn stream_data_points(format: LineFormat) -> Result<StreamResult, String> {
    let mut points = vec![];
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    process_lines(|trimmed, idx| {
        let point: DataPoint = match format {
            LineFormat::Single => {
                let value = trimmed.parse::<f64>().map_err(|err| line_error(idx, err))?;
                Ok::<DataPoint, String>(DataPoint { value, count: 1 })
            }
            LineFormat::KeyValue => {
                let (fst, snd) = f64_tuple(trimmed).map_err(|err| line_error(idx, err))?;
                Ok(DataPoint {
                    value: fst,
                    count: snd as usize,
                })
            }
            LineFormat::ValueKey => {
                let (fst, snd) = f64_tuple(trimmed).map_err(|err| line_error(idx, err))?;
                Ok(DataPoint {
                    value: snd,
                    count: fst as usize,
                })
            }
        }?;

        min = min.min(point.value);
        max = max.max(point.value);

        points.push(point);
        Ok(())
    })?;

    Ok(StreamResult { points, min, max })
}

fn tuple(line: &str) -> Result<(&str, &str), String> {
    let mut splitter = line.split_whitespace();
    let fst = splitter
        .next()
        .ok_or_else(|| String::from("expecting two values"))?;
    let snd = splitter
        .next()
        .ok_or_else(|| String::from("expecting two values"))?;

    Ok((fst, snd))
}

fn f64_tuple(line: &str) -> Result<(f64, f64), String> {
    let (fst, snd) = tuple(line)?;

    let fst_num = fst.parse::<f64>().map_err(|err| err.to_string())?;
    let snd_num = snd.parse::<f64>().map_err(|err| err.to_string())?;

    Ok((fst_num, snd_num))
}

fn bail_out(err: &str) {
    eprintln!("{}", err);
    std::process::exit(1)
}

fn line_error<T: ToString>(idx: usize, error: T) -> String {
    format!("line {}: {}", idx + 1, error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::{linear_scale_buckets, log_scale_buckets};

    #[test]
    fn linear_scale() {
        let buckets = linear_scale_buckets(4, 0.0, 100.0);
        assert_eq!(buckets, vec![25.0, 50.0, 75.0, 100.0]);
    }

    #[test]
    fn log_scale() {
        let buckets = log_scale_buckets(4, 0.0, 300.0);
        assert_eq!(buckets, vec![20.0, 60.0, 140.0, 300.0]);
    }
}
