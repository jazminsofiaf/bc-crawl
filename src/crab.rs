use ferris_says::say; // from the previous step
use std::io::{stdout, BufWriter};

pub fn foo() {
    let stdout = stdout();
    let out = b"Soy un cangrejo!";
    let width = 30;

    let mut writer = BufWriter::new(stdout.lock());
    say(out, width, &mut writer).unwrap();
}