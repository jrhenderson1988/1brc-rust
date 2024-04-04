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
    data_bytes: HashMap<Vec<u8>, Values>,
}

impl StationData {
    pub fn new() -> Self {
        Self { data: HashMap::new(), data_bytes: HashMap::new() }
    }

    pub fn consume_line_bytes(&mut self, line: &[u8]) {
        let (name, reading) = self.parse_line_bytes(line);
        if let Some(v) = self.data_bytes.get_mut(&name) {
            v.add(reading);
        } else {
            self.data_bytes.insert(name, Values::new(reading));
        }
    }

    pub fn sorted_keys_bytes(&self) -> Vec<Vec<u8>> {
        let mut keys: Vec<Vec<u8>> = self.data_bytes.keys().map(|v| v.clone()).collect();
        keys.sort();

        keys
    }

    pub fn min_for_bytes(&self, name: &[u8]) -> f64 {
        match self.data_bytes.get(name) {
            None => 0.0,
            Some(v) => v.min,
        }
    }

    pub fn max_for_bytes(&self, name: &[u8]) -> f64 {
        match self.data_bytes.get(name) {
            None => 0.0,
            Some(v) => v.max,
        }
    }

    pub fn mean_for_bytes(&self, name: &[u8]) -> f64 {
        match self.data_bytes.get(name) {
            None => 0.0,
            Some(v) => v.sum / (v.count as f64),
        }
    }

    fn parse_line_bytes(&self, line: &[u8]) -> (Vec<u8>, f64) {
        for i in 0..line.len() {
            let ch = line[i];
            if ch == b';' {
                let name = line[0..i].to_vec();
                let mut reading: f64 = String::from_utf8(line[i + 1..line.len()].to_vec())
                    .unwrap()
                    .parse()
                    .unwrap();
                return (name, reading);
            }
        }
        panic!("invalid line");
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