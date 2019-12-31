use crate::Error;
use crossterm::{
    cursor::MoveToColumn,
    event, queue,
    style::Print,
    terminal::{Clear, ClearType},
};
use std::{cmp, io::Write, iter, ops::FnMut, time::Duration};

pub struct Bar<'a, W, F>
where
    W: Write,
    F: FnMut(u64, u64) -> String,
{
    pub(crate) total: u64,
    pub(crate) width: u64,
    pub(crate) max_width: u64,
    pub(crate) status_width: u64,
    pub(crate) status_fmt: F,
    pub(crate) count: u64,
    pub(crate) target: &'a mut W,
}

impl<'a, W, F> Bar<'a, W, F>
where
    W: Write,
    F: FnMut(u64, u64) -> String,
{
    pub fn inc(&mut self, count: u64) -> Result<(), Error> {
        self.update(self.count + count)
    }

    pub fn finish(mut self) -> Result<(), Error> {
        self.update(self.total)
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
