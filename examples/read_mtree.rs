extern crate mtree;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use mtree::MTree;

fn main() -> Result<(), Box<Error>> {
    let path = env::current_dir()?.join("examples/gedit.mtree");
    let mtree = MTree::from_reader(File::open(path)?);
    for entry in mtree {
        println!("{:#?}", entry);
    }
    Ok(())
}
