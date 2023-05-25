pub struct MVSD {
    started: bool,
    sum_squares: f64,
    means: f64,
    total_weights: f64,
}

impl MVSD {
    pub fn new() -> MVSD {
        MVSD {
            started: false,
            sum_squares: 0f64,
            means: 0f64,
            total_weights: 0f64,
        }
    }

    pub fn add(&mut self, value: f64, weight: f64) {
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
    }

    pub fn var(&self) -> f64 {
        self.sum_squares / self.total_weights
    }

    pub fn sd(&self) -> f64 {
        self.var().sqrt()
    }

    pub fn mean(&self) -> f64 {
        self.means
    }
}

#[cfg(test)]
mod tests {
    use super::MVSD;

    #[test]
    fn mvsd_correct_values() {
        let mut mvsd = MVSD::new();
        for value in 0..10 {
            mvsd.add(value as f64, 1.0);
        }

        assert_eq!(format!("{:.2}", mvsd.mean()), "4.50");
        assert_eq!(format!("{:.2}", mvsd.var()), "8.25");
        assert_eq!(format!("{:.2}", mvsd.sd()), "2.87");
    }
}
