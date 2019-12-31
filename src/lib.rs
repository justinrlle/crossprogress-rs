pub mod bar;
pub mod builder;

use crate::builder::BarBuilder;
use std::{error, fmt, io};

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    Crossterm(crossterm::ErrorKind),
    MissingField(&'static str),
    Overflow(u64),
    Io(io::Error),
}

impl From<crossterm::ErrorKind> for Error {
    fn from(error: crossterm::ErrorKind) -> Error {
        Error::Crossterm(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Crossterm(_) => write!(f, "an error occured in crossterm"),
            Error::MissingField(name) => write!(f, "the field `{}` is missing", name),
            Error::Overflow(count) => write!(f, "the bar has been overflowed at {}", count),
            Error::Io(_) => write!(f, "io error"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Crossterm(err) => Some(err),
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

pub fn percent_bar(total: u64) -> BarBuilder<fn(u64, u64) -> String> {
    BarBuilder::new(total).status(4, |count, total| format!("{}%", count * 100 / total))
}
