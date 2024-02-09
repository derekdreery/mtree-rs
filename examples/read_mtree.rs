use mtree::MTree;
use std::env;
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::current_dir()?.join("examples/gedit.mtree");
    let mtree = MTree::from_reader(File::open(path)?);
    for entry in mtree {
        println!("{}", entry?);
    }
    Ok(())
}
