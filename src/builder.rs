use crate::{bar::Bar, Error};

use std::{io::Write, ops::FnMut};

#[derive(Default)]
pub struct BarBuilder<F>
where
    F: FnMut(u64, u64) -> String,
{
    total: u64,
    width: Option<u64>,
    max_width: Option<u64>,
    status_width: Option<u64>,
    status_fmt: Option<F>,
}

impl<F> BarBuilder<F>
where
    F: FnMut(u64, u64) -> String,
{
    pub fn new(total: u64) -> Self {
        BarBuilder {
            total,
            width: None,
            max_width: None,
            status_width: None,
            status_fmt: None,
        }
    }

    pub fn width(mut self, width: u64) -> Self {
        self.width = Some(width);
        self
    }

    pub fn max_width(mut self, width: u64) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn status(mut self, width: u64, fmt: F) -> Self {
        self.status_width = Some(width);
        self.status_fmt = Some(fmt);
        self
    }

    pub fn build<W: Write>(mut self, target: &mut W) -> Result<Bar<W, F>, Error> {
        let width = if let Some(width) = self.width.take() {
            width
        } else {
            crossterm::terminal::size()?.0.into()
        };
        let max_width = self.max_width.unwrap_or(width);
        let mut bar = Bar {
            total: self.total,
            width,
            max_width,
            status_width: self
                .status_width
                .ok_or(Error::MissingField("status_width"))?,
            status_fmt: self.status_fmt.ok_or(Error::MissingField("status_fmt"))?,
            count: 0,
            target,
        };
        bar.update(0)?;
        Ok(bar)
    }
}
