use std::io::{self, Read, BufRead};
use std::iter;
use std::path::PathBuf;
use std::env;

mod parser;
mod util;

pub use parser::MTreeLine;

pub struct MTree {
    inner: Box<Iterator<Item=Result<Vec<u8>, io::Error>>>,
    cwd: Option<PathBuf>,
    set_params: Params,
}

impl MTree {
    pub fn from_reader(reader: impl Read + 'static) -> MTree {
        let reader = io::BufReader::new(reader);
        MTree {
            inner: Box::new(reader.split(b'\n')
                // remove trailing '\r'
                .map(|line|
                    line.map(|mut line| {
                        if ! line.is_empty() && line[line.len()-1] == b'r' {
                            line.pop();
                        }
                        line
                    })
                )
            ),
            cwd: env::current_dir().ok(),
            set_params: Params::default(),
        }
    }
}

impl Iterator for MTree {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        unimplemented!()
    }
}

pub struct Entry {
    path: PathBuf,
    params: Params,
}

#[derive(Default)]
pub struct Params;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

