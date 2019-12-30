use crossterm::{
    cursor::{Hide, MoveToColumn, Show},
    event, queue,
    style::Print,
    terminal::{Clear, ClearType},
};
use std::{
    io::{self, Write},
    iter,
    ops::FnMut,
    thread,
    time::Duration,
};

fn main() {
    let (columns, rows) = crossterm::terminal::size().unwrap();
    println!("terminal: {} columns, {} rows", columns, rows);
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, Hide).unwrap();
    let mut bar = BarBuilder::new()
        .total(50)
        .width(columns.into(), 80)
        .status(4, |count, total| format!("{}%", count * 100 / total))
        .build(&mut stdout);
    for i in 0..=50 {
        if let Some(event::Event::Resize(new_cols, _)) = poll_event() {
            bar.resize(new_cols.into());
        }
        bar.update(i);
        thread::sleep(Duration::from_millis(100));
    }
    crossterm::execute!(stdout, Show).unwrap();
}

fn poll_event() -> Option<event::Event> {
    if event::poll(Duration::from_secs(0)).expect("event::poll failed") {
        return Some(event::read().expect("event::read failed"));
    }
    None
}

pub struct BarBuilder<F>
where
    F: FnMut(u64, u64) -> String,
{
    total: Option<u64>,
    width: Option<u64>,
    max_width: Option<u64>,
    status_width: Option<u64>,
    status_fmt: Option<F>,
}

impl<F> BarBuilder<F>
where
    F: FnMut(u64, u64) -> String,
{
    pub fn new() -> Self {
        BarBuilder {
            total: None,
            width: None,
            max_width: None,
            status_width: None,
            status_fmt: None,
        }
    }
    pub fn total(self, total: u64) -> Self {
        let mut builder = self;
        builder.total = Some(total);
        builder
    }
    pub fn width(self, width: u64, max: u64) -> Self {
        let mut builder = self;
        builder.width = Some(width);
        builder.max_width = Some(max);
        builder
    }

    pub fn status(self, width: u64, fmt: F) -> Self {
        let mut builder = self;
        builder.status_width = Some(width);
        builder.status_fmt = Some(fmt);
        builder
    }

    pub fn build<W: Write>(self, target: &mut W) -> Bar<W, F> {
        Bar {
            total: self.total.expect("`total` field not specified"),
            width: self.width.expect("`width` field not specified"),
            max_width: self.max_width.expect("`max_width` field not specified"),
            status_width: self
                .status_width
                .expect("`status_width` field not specified"),
            status_fmt: self.status_fmt.expect("`status_fmt` field not specified"),
            count: 0,
            new_width: None,
            target,
        }
    }
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
    new_width: Option<u64>,
    target: &'a mut W,
}

impl<'a, W, F> Bar<'a, W, F>
where
    W: Write,
    F: FnMut(u64, u64) -> String,
{
    pub fn update(&mut self, count: u64) {
        if let Some(width) = self.new_width.take() {
            if width < self.width() {
                queue!(self.target, Print("\n".to_owned()),).unwrap();
            }
            self.width = width;
        }
        self.count = count;
        let bar = self.bar();
        queue!(
            self.target,
            MoveToColumn(1),
            Clear(ClearType::CurrentLine),
            Print(bar),
        )
        .unwrap();
        self.target.flush().unwrap();
    }

    pub fn resize(&mut self, width: u64) {
        self.new_width = Some(width);
    }

    fn width(&self) -> u64 {
        std::cmp::min(self.width, self.max_width)
    }

    fn bar(&mut self) -> String {
        let width = self.width();
        let mut res = String::with_capacity(width as usize);
        res.push('[');

        let bar_width = width
            - 3 // around the bar
            - self.status_width // the text at the end
            ;
        let percent_width = self.count * bar_width / self.total;
        if percent_width != 0 {
            res.extend(iter::repeat('#').take(percent_width as usize));
        }
        if percent_width != bar_width {
            res.extend(iter::repeat(' ').take((bar_width - percent_width) as usize));
        }
        let end = format!("] {}", (self.status_fmt)(self.count, self.total));
        res.push_str(&end);
        res
    }
}
