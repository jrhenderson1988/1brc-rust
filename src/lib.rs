use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::time::SystemTime;

use crate::station_data::StationData;

mod station_data;

const BUF_SIZE: usize = 1024 * 1024 * 16;
const USE_BUFFERED_READER: bool = false;

pub fn execute<P: AsRef<Path>, W: Write>(path: P, writer: W) -> io::Result<()> {
    let start = SystemTime::now();

    if USE_BUFFERED_READER {
        with_buffered_reader(path, writer)?;
    } else {
        with_chunked_reader(path, writer)?;
    }

    let duration = SystemTime::now().duration_since(start).unwrap();
    println!("\n\nDuration: {:?}", duration);

    Ok(())
}

fn with_buffered_reader<P: AsRef<Path>, W: Write>(path: P, mut writer: W) -> io::Result<()> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let mut data = StationData::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.as_bytes();
        data.consume_line(line);
    }

    output(&data, &mut writer);
    Ok(())
}

fn with_chunked_reader<P: AsRef<Path>, W: Write>(path: P, mut writer: W) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buf = vec![0u8; BUF_SIZE];
    let mut leftover = vec![0u8; BUF_SIZE];
    let mut leftover_size: usize = 0;
    let mut data = StationData::new();

    // TODO - create an mpsc, pass in the sender to the thread pool, make threads do the work and
    //  then send the result to us. We loop over the receive N times (N = chunks) to get the data
    //  and assemble it again.

    loop {
        let read = file.read(&mut buf)?;
        if read != BUF_SIZE {
            let mut chunk = vec![0u8; leftover_size + read];
            for i in 0..leftover_size {
                chunk[i] = leftover[i];
            }
            for i in 0..read {
                chunk[i + leftover_size] = buf[i];
            }

            data.extend(handle_chunk(chunk));
            break;
        }

        let mut last_newline_pos = read - 1;
        while last_newline_pos >= 0 {
            if buf[last_newline_pos] == b'\n' {
                break;
            }
            last_newline_pos -= 1;
        }

        let mut chunk = vec![0u8; leftover_size + last_newline_pos];
        for i in 0..leftover_size {
            chunk[i] = leftover[i];
        }
        for i in 0..last_newline_pos {
            chunk[i + leftover_size] = buf[i];
        }

        let data_for_chunk = handle_chunk(chunk);
        data.extend(data_for_chunk);

        let leftover_start = last_newline_pos + 1;
        leftover_size = buf.len() - leftover_start;
        for i in leftover_start..buf.len() {
            leftover[i - leftover_start] = buf[i];
        }
    }

    output(&data, &mut writer);
    Ok(())
}

fn handle_chunk(chunk: Vec<u8>) -> StationData {
    let mut data = StationData::new();

    let mut start = 0;
    for i in 0..chunk.len() {
        if chunk[i] == b'\n' {
            data.consume_line(&chunk[start..i]);
            start = i + 1;
        }
    }

    if start < chunk.len() - 1 {
        data.consume_line(&chunk[start..]);
    }

    data
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