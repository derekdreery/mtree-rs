extern crate mtree;
extern crate failure;

use std::env;
use std::error::Error;
use std::fs::File;
use mtree::MTree;
use failure::Fail;

fn main() -> Result<(), Box<Error>> {
    let path = env::current_dir()?.join("examples/gedit.mtree");
    let mtree = MTree::from_reader(File::open(path)?);
    for entry in mtree {
        println!("{}", entry.map_err(|e| e.compat())?);
    }
    Ok(())
}
