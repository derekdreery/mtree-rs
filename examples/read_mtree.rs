extern crate mtree;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<Error>> {
    let path = env::current_dir()?.join("examples/gedit.mtree");
    let mut raw = io::BufReader::new(File::open(path)?);
    for line in raw.split(b'\n') {
        let line = line?;
        println!("{:?}", mtree::MTreeLine::from_bytes(&line));
    }
    Ok(())
}
