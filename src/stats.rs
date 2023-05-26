pub struct MVSD {
    started: bool,
    sum_squares: f64,
    means: f64,
    total_weights: f64,
    values: Vec<f64>,
}

impl MVSD {
    pub fn new() -> MVSD {
        MVSD {
            started: false,
            sum_squares: 0f64,
            means: 0f64,
            total_weights: 0f64,
            values: vec![],
        }
    }

    pub fn add(&mut self, value: f64, weight: f64) {
        if weight <= 0.0 {
            return;
        }

        if !self.started {
            self.means = value;
            self.sum_squares = 0f64;
            self.total_weights = weight;
            self.started = true
        } else {
            let new_weight = self.total_weights + weight;
            self.sum_squares +=
                (self.total_weights * weight * (value - self.means) * (value - self.means))
                    / new_weight;
            self.means += (value - self.means) / new_weight;
            self.total_weights = new_weight;
        }
        self.values.push(value)
    }

    pub fn var(&self) -> f64 {
        if self.started {
            self.sum_squares / self.total_weights
        } else {
            0.0
        }
    }

    pub fn sd(&self) -> f64 {
        self.var().sqrt()
    }

    pub fn mean(&self) -> f64 {
        self.means
    }

    pub fn median(self) -> f64 {
        median(self.values)
    }
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let length = values.len();
    let indices = if length % 2 != 0 {
        vec![length / 2]
    } else {
        vec![length / 2 - 1, length / 2]
    };

    values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let num_indices = indices.len() as f64;
    let median_values = indices.into_iter().map(|idx| values[idx]);

    median_values.sum::<f64>() / num_indices
}

#[cfg(test)]
mod tests {
    use super::{median, MVSD};

    #[test]
    fn correct_mvsd() {
        let mut mvsd = MVSD::new();
        for value in 0..10 {
            mvsd.add(value as f64, 1.0);
        }

        assert_eq!(format!("{:.2}", mvsd.mean()), "4.50");
        assert_eq!(format!("{:.2}", mvsd.var()), "8.25");
        assert_eq!(format!("{:.2}", mvsd.sd()), "2.87");
    }

    #[test]
    fn correct_median() {
        assert_eq!(median(vec![8.0, 7.0, 9.0, 1.0, 2.0, 6.0, 3.0]), 6.0);
        assert_eq!(median(vec![4.0, 5.0, 2.0, 1.0, 9.0, 10.0]), 4.5);
    }
}
