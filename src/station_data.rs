use std::collections::HashMap;
use std::fmt::{Display, Formatter};

struct Values {
    min: i64,
    max: i64,
    count: u64,
    sum: i64,
}

impl Values {
    pub fn new(reading: i64) -> Self {
        Self { min: reading, max: reading, count: 1, sum: reading }
    }

    pub fn add(&mut self, reading: i64) {
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

    pub fn consume_line(&mut self, line: &[u8]) {
        let (name, reading) = self.parse_line_bytes(line);
        if let Some(v) = self.data_bytes.get_mut(&name) {
            v.add(reading);
        } else {
            self.data_bytes.insert(name, Values::new(reading));
        }
    }

    pub fn sorted_keys(&self) -> Vec<Vec<u8>> {
        let mut keys: Vec<Vec<u8>> = self.data_bytes.keys().map(|v| v.clone()).collect();
        keys.sort();

        keys
    }

    pub fn min_for(&self, name: &[u8]) -> f64 {
        match self.data_bytes.get(name) {
            None => 0.0,
            Some(v) => (v.min as f64) / 10.0,
        }
    }

    pub fn max_for(&self, name: &[u8]) -> f64 {
        match self.data_bytes.get(name) {
            None => 0.0,
            Some(v) => (v.max as f64) / 10.0,
        }
    }

    pub fn mean_for(&self, name: &[u8]) -> f64 {
        match self.data_bytes.get(name) {
            None => 0.0,
            Some(v) => ((v.sum as f64) / 10.0) / (v.count as f64),
        }
    }

    fn parse_line_bytes(&self, line: &[u8]) -> (Vec<u8>, i64) {
        for i in 0..line.len() {
            let ch = line[i];
            let mut value: u64 = 0;
            if ch == b';' {
                let name = line[0..i].to_vec();
                let mut value : i64 = 0;
                let mut negative = false;
                for j in i + 1..line.len() {
                    let ch = line[j];
                    if ch == b'-' {
                        negative = true;
                    } else if ch.is_ascii_digit() {
                        value = (value * 10) + (ch as i64 - 48);
                    } else if ch == b'.' {
                        continue;
                    } else {
                        panic!("unexpected value")
                    }
                }
                return (name, if negative { -value } else { value });
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