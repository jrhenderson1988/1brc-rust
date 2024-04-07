use std::fmt::{Display, Formatter};

use rustc_hash::FxHashMap;

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

    pub fn extend(&mut self, other: Values) {
        self.min = if other.min < self.min { other.min } else { self.min };
        self.max = if other.max > self.max { other.max } else { self.max };
        self.count = self.count + other.count;
        self.sum = self.sum + other.sum;
    }
}

impl Display for Values {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "min={}/max={}/sum={}/count={}", self.min, self.max, self.sum, self.count)
    }
}

pub struct StationData {
    data: FxHashMap<Vec<u8>, Values>,
}

impl StationData {
    pub fn new() -> Self {
        Self { data: FxHashMap::default() }
    }

    pub fn consume_line(&mut self, line: &[u8]) {
        let (name, reading) = self.parse_line(line);
        self.add_reading(name, reading);
    }

    fn add_reading(&mut self, name: &[u8], reading: i64) {
        if let Some(v) = self.data.get_mut(name) {
            v.add(reading);
        } else {
            self.data.insert(name.to_vec(), Values::new(reading));
        }
    }

    pub fn consume_chunk(&mut self, chunk: Vec<u8>) {
        let mut start = 0;
        for i in 0..chunk.len() {
            if chunk[i] == b'\n' {
                self.consume_line(&chunk[start..i]);
                start = i + 1;
            }
        }

        if start < chunk.len() - 1 {
            self.consume_line(&chunk[start..]);
        }
    }

    pub fn extend(&mut self, other: StationData) {
        for (name, values) in other.data.into_iter() {
            if let Some(v) = self.data.get_mut(&name) {
                v.extend(values);
            } else {
                self.data.insert(name, values);
            }
        }
    }

    pub fn sorted_keys(&self) -> Vec<Vec<u8>> {
        let mut keys: Vec<Vec<u8>> = self.data.keys().map(|v| v.clone()).collect();
        keys.sort();

        keys
    }

    pub fn min_for(&self, name: &[u8]) -> f64 {
        match self.data.get(name) {
            None => 0.0,
            Some(v) => (v.min as f64) / 10.0,
        }
    }

    pub fn max_for(&self, name: &[u8]) -> f64 {
        match self.data.get(name) {
            None => 0.0,
            Some(v) => (v.max as f64) / 10.0,
        }
    }

    pub fn mean_for(&self, name: &[u8]) -> f64 {
        match self.data.get(name) {
            None => 0.0,
            Some(v) => ((v.sum as f64) / 10.0) / (v.count as f64),
        }
    }

    fn parse_line<'a>(&self, line: &'a [u8]) -> (&'a [u8], i64) {
        for i in 0..line.len() {
            let ch = line[i];
            if ch == b';' {
                let semicolon_pos = i;
                let mut value: i64 = 0;
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
                return (&line[0..semicolon_pos], if negative { -value } else { value });
            }
        }
        let text = String::from_utf8(line.to_vec()).unwrap();
        panic!("invalid line (length: {}, line: {:?})", line.len(), text);
    }
}
