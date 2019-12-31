use crossterm::{
    cursor::MoveToColumn,
    event, queue,
    style::Print,
    terminal::{Clear, ClearType},
};
use std::{
    io::{self, Write},
    iter,
    ops::FnMut,
    time::Duration,
    cmp,
};

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

    pub fn width(self, width: u64) -> Self {
        let mut builder = self;
        builder.width = Some(width);
        builder
    }

    pub fn max_width(self, width: u64) -> Self {
        let mut builder = self;
        builder.max_width = Some(width);
        builder
    }

    pub fn status(self, width: u64, fmt: F) -> Self {
        let mut builder = self;
        builder.status_width = Some(width);
        builder.status_fmt = Some(fmt);
        builder
    }

    pub fn build<W: Write>(self, target: &mut W) -> Result<Bar<W, F>, Error> {
        let mut builder = self;
        let width = if let Some(width) = builder.width.take() {
            width
        } else {
            crossterm::terminal::size()?.0.into()
        };
        let max_width = builder.max_width.unwrap_or(width);
        let mut bar = Bar {
            total: builder.total,
            width,
            max_width,
            status_width: builder
                .status_width
                .ok_or(Error::MissingField("status_width"))?,
            status_fmt: builder
                .status_fmt
                .ok_or(Error::MissingField("status_fmt"))?,
            count: 0,
            target,
        };
        bar.update(0)?;
        Ok(bar)
    }
}

pub fn percent_bar(total: u64) -> BarBuilder<fn(u64, u64) -> String> {
    BarBuilder::new(total).status(4, |count, total| format!("{}%", count * 100 / total))
}

pub struct Bar<'a, W, F>
where
    W: Write,
    F: FnMut(u64, u64) -> String,
{
    total: u64,
    width: u64,
    max_width: u64,
    status_width: u64,
    status_fmt: F,
    count: u64,
    target: &'a mut W,
}

impl<'a, W, F> Bar<'a, W, F>
where
    W: Write,
    F: FnMut(u64, u64) -> String,
{
    pub fn inc(&mut self, count: u64) -> Result<(), Error> {
        self.update(self.count + count)
    }

    pub fn update(&mut self, count: u64) -> Result<(), Error> {
        if count > self.total {
            return Err(Error::Overflow(count));
        }
        if let Some(event::Event::Resize(width, _)) = poll_event() {
            let width = width.into();
            if width < self.width() {
                queue!(self.target, Print("\n".to_owned()))?;
            }
            self.width = width;
        }
        self.count = count;
        let output = self.bar();
        queue!(
            self.target,
            MoveToColumn(1),
            Clear(ClearType::CurrentLine),
            Print(output),
        )?;
        self.target.flush()?;
        Ok(())
    }

    fn width(&self) -> u64 {
        cmp::min(self.width, self.max_width)
    }

    fn bar(&mut self) -> String {
        let width = self.width();
        let mut res = String::with_capacity(width as usize);
        res.push('[');

        let bar_width = width
            - 3 // around the bar
            - self.status_width // the text at the end
            ;
        let percent_width = cmp::min(self.count * bar_width / self.total, bar_width);
        if percent_width != 0 {
            res.extend(iter::repeat('#').take(percent_width as usize));
        }
        if percent_width != bar_width {
            res.extend(iter::repeat(' ').take((bar_width - percent_width) as usize));
        }
        let end = format!(
            "] {0:.1$}",
            (self.status_fmt)(self.count, self.total),
            self.status_width as usize,
        );
        res.push_str(&end);
        res
    }
}

fn poll_event() -> Option<event::Event> {
    if event::poll(Duration::from_secs(0)).expect("event::poll failed") {
        return Some(event::read().expect("event::read failed"));
    }
    None
}
