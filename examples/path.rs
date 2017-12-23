extern crate elayr;

use std::io;
use elayr::Path;

pub fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();

    let path = Path::parse(buffer.as_str()).expect("Failed to parse path");
    println!("{}", path);
}
