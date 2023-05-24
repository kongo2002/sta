use std::io::BufRead;

use argh::FromArgs;

#[derive(Debug)]
struct DataPoint {
    // TODO: smth like decimal?
    pub value: i32,
    pub count: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "histogram from the command line")]
struct Args {
    #[argh(option, description = "number of buckets", short = 'b', default = "10")]
    buckets: i32,
    #[argh(option, description = "minimum value")]
    min: Option<i32>,
    #[argh(option, description = "maximum value")]
    max: Option<i32>,
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
    if data.is_empty() {
        return Err(String::from("empty input"));
    }

    let minimum = args
        .min
        .or_else(|| {
            data.iter()
                .min_by_key(|pt| pt.value)
                .as_ref()
                .map(|pt| pt.value)
        })
        .unwrap();

    let maximum = args
        .max
        .or_else(|| {
            data.iter()
                .max_by_key(|pt| pt.value)
                .as_ref()
                .map(|pt| pt.value)
        })
        .unwrap();

    let diff = maximum - minimum;

    let buckets = args.buckets;
    let step = diff / buckets;

    let mut excluded = 0usize;
    let mut boundaries = vec![];
    let mut bucket_counts = vec![0; buckets as usize];

    for idx in 0..buckets {
        boundaries.push(minimum + (step * (idx + 1)));
    }

    let last_bucket = bucket_counts.len() - 1;
    let get_bucket = |val: i32| {
        for (idx, boundary) in boundaries.iter().enumerate() {
            if val <= *boundary {
                return idx;
            }
        }
        last_bucket
    };

    for point in data {
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
            bucket_min,
            bucket_max,
            count,
            "*".repeat(dots)
        ));
    }

    print!("{}", table);

    Ok(())
}

fn stream() -> Result<Vec<DataPoint>, String> {
    let mut points = vec![];
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let raw_line = line.map_err(str_error)?;
        if raw_line.is_empty() {
            continue;
        }

        let value = raw_line.parse::<i32>().map_err(str_error)?;

        points.push(DataPoint { value, count: 1 })
    }

    Ok(points)
}

fn str_error<T: ToString>(error: T) -> String {
    error.to_string()
}
