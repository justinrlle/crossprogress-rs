use std::{
    io::self,
    thread,
    time::Duration,
};

fn main() {
    let (columns, rows) = crossterm::terminal::size().unwrap();
    println!("terminal: {} columns, {} rows", columns, rows);
    let mut stdout = io::stdout();
    let mut bar = crossprogress::percent_bar(50).build(&mut stdout).unwrap();
    for _ in 0..50 {
        thread::sleep(Duration::from_millis(1000));
        bar.inc(1).unwrap();
    }
}
