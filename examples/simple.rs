use std::{io, thread, time::Duration};

fn main() {
    let mut stdout = io::stdout();
    let mut bar = crossprogress::percent_bar(50).build(&mut stdout).unwrap();
    for _ in 0..50 {
        thread::sleep(Duration::from_millis(200));
        bar.inc(1).unwrap();
    }
}
