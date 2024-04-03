use std::io;

use onebrc_rust::execute;

fn main() -> io::Result<()> {
    execute("./measurements.txt", io::stdout())?;

    Ok(())
}
