extern crate elayr;

use std::io::{self, Read};
use elayr::{Node, XMLDoc};

pub fn main() {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();

    let doc = XMLDoc::parse(buffer.as_str()).expect("Failed to parse document");
    let svg = Node::from_xml_doc(doc).expect("XML doc is not valid SVG");
    println!("{}", svg);
}
