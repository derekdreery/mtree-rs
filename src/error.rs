use std::io;

#[derive(Debug, Fail)]
pub enum Error {
    Io(#[cause] io::Error)
}
