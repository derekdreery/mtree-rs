extern crate failure;
extern crate mtree;

use failure::Fail;
use mtree::MTree;
use std::env;
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<Error>> {
    let path = env::current_dir()?.join("examples/gedit.mtree");
    let mtree = MTree::from_reader(File::open(path)?);
    for entry in mtree {
        println!("{}", entry.map_err(|e| e.compat())?);
    }
    Ok(())
}
