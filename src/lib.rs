use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::time::SystemTime;

use crate::station_data::StationData;
use crate::thread_pool::ThreadPool;

mod station_data;
mod thread_pool;

const BUF_SIZE: usize = 1024 * 1024 * 4;
const USE_BUFFERED_READER: bool = false;
const THREAD_COUNT: usize = 15;

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

    let mut total_chunks = 0;
    let (result_sender, result_receiver) = mpsc::channel::<StationData>();
    let thread_pool = ThreadPool::new(THREAD_COUNT, result_sender);

    loop {
        let read = file.read(&mut buf)?;
        if read != BUF_SIZE {
            let chunk = create_chunk(&buf, read, &leftover, leftover_size);
            total_chunks += 1;
            thread_pool.execute(chunk);
            break;
        }

        let last_newline_pos = find_last_newline_pos(&buf, read);
        let chunk = create_chunk(&buf, last_newline_pos, &leftover, leftover_size);

        total_chunks += 1;
        thread_pool.execute(chunk);
        leftover_size = copy_leftover(&mut leftover, &buf, last_newline_pos);
    }

    for _ in 0..total_chunks {
        let result = result_receiver.recv().unwrap();
        data.extend(result);
    }

    output(&data, &mut writer);
    Ok(())
}

fn create_chunk(buf: &[u8], buf_pos: usize, leftover: &[u8], leftover_size: usize) -> Vec<u8> {
    let mut chunk = vec![0u8; leftover_size + buf_pos];
    chunk[..leftover_size].copy_from_slice(&leftover[..leftover_size]);
    chunk[leftover_size..leftover_size + buf_pos].copy_from_slice(&buf[0..buf_pos]);

    chunk
}

fn find_last_newline_pos(buf: &[u8], read: usize) -> usize {
    let mut last_newline_pos = read - 1;
    while last_newline_pos != 0 {
        if buf[last_newline_pos] == b'\n' {
            break;
        }
        last_newline_pos -= 1;
    }
    last_newline_pos
}

fn copy_leftover(leftover: &mut [u8], buf: &[u8], last_newline_pos: usize) -> usize {
    let leftover_start = last_newline_pos + 1;
    let leftover_size = buf.len() - leftover_start;
    leftover[0..leftover_size].copy_from_slice(&buf[leftover_start..]);

    leftover_size
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