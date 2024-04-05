use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::time::SystemTime;

use crate::station_data::StationData;

mod station_data;

const BUF_SIZE: usize = 1024 * 1024 * 32;

pub fn execute<P: AsRef<Path>, W: Write>(path: P, mut writer: W) -> io::Result<()> {
    let start = SystemTime::now();

    with_buffered_reader(path, writer)?;

    let duration = SystemTime::now().duration_since(start).unwrap();
    println!("\n\nDuration: {:?}", duration);

    Ok(())
}

fn with_buffered_reader<P: AsRef<Path>, W: Write>(path: P, mut writer: W) -> io::Result<()> {
    let file = File::open(path.as_ref())?;
    let mut reader = BufReader::new(file);
    let mut data = StationData::new();

    // TODO - create an mpsc, pass in the sender to the thread pool, make threads do the work and
    //  then send the result to us. We loop over the receive N times (N = chunks) to get the data
    //  and assemble it again.

    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        let line = line.as_bytes();
        data.consume_line(line);
        count += 1;
    }

    output(&data, &mut writer);
    Ok(())
}

fn read<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buf = vec![0u8; BUF_SIZE];

    let mut count = 0;
    loop {
        let read = file.read(&mut buf)?;
        if read != BUF_SIZE {
            break;
        }

        count += 1;
    }

    println!("count: {}", count);
    Ok(())
}

fn output<W: Write>(data: &StationData, writer: &mut W) {
    write!(writer, "{{").unwrap();

    let mut first = true;
    for k in data.sorted_keys() {
        if first {
            first = false;
        } else {
            write!(writer, ", ").unwrap();
        }

        let min = data.min_for(&k);
        let mean = data.mean_for(&k);
        let max = data.max_for(&k);
        let name = String::from_utf8(k).unwrap();
        write!(writer, "{}={:.1}/{:.1}/{:.1}", name, min, mean, max).unwrap();
    }

    write!(writer, "}}").unwrap();
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::execute;

    #[test]
    fn test() {
        let expected = fs::read_to_string("./expected.txt").unwrap();
        let mut v: Vec<u8> = vec![];

        execute("./test.txt", &mut v).unwrap();

        assert_eq!(expected, String::from_utf8(v).unwrap());
    }
}