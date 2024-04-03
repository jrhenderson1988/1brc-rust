use std::collections::HashMap;
use std::fmt::{Display, Formatter};

struct Values {
    min: f64,
    max: f64,
    count: u64,
    sum: f64,
}

impl Values {
    pub fn new(reading: f64) -> Self {
        Self { min: reading, max: reading, count: 1, sum: reading }
    }

    pub fn add(&mut self, reading: f64) {
        self.min = if reading < self.min { reading } else { self.min };
        self.max = if reading > self.max { reading } else { self.max };
        self.count = self.count + 1;
        self.sum = self.sum + reading;
    }
}

impl Display for Values {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "min={}/max={}/sum={}/count={}", self.min, self.max, self.sum, self.count)
    }
}

pub struct StationData {
    data: HashMap<String, Values>,
}

impl StationData {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn consume_line(&mut self, line: &str) {
        let (name, reading) = self.parse_line(&line);
        if let Some(v) = self.data.get_mut(&name) {
            v.add(reading);
        } else {
            self.data.insert(name, Values::new(reading));
        }
    }

    pub fn sorted_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.data.keys().map(|k| k.clone()).collect();
        keys.sort();

        keys
    }

    pub fn min_for(&self, name: &str) -> f64 {
        match self.data.get(name) {
            None => 0.0,
            Some(v) => v.min,
        }
    }

    pub fn max_for(&self, name: &str) -> f64 {
        match self.data.get(name) {
            None => 0.0,
            Some(v) => v.max,
        }
    }

    pub fn mean_for(&self, name: &str) -> f64 {
        match self.data.get(name) {
            None => 0.0,
            Some(v) => v.sum / (v.count as f64),
        }
    }

    fn parse_line(&self, s: &str) -> (String, f64) {
        let parts: Vec<&str> = s.split(";").collect();
        let name = parts.first().unwrap();
        let reading = parts.last().unwrap();
        let reading: f64 = reading.parse().unwrap();

        (name.to_string(), reading)
    }
}

impl Display for StationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for k in self.data.keys() {
            write!(f, "{} : {}", k, self.data.get(k).unwrap()).unwrap();
        }
        Ok(())
    }
}