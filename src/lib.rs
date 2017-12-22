#[macro_use]
extern crate nom;
extern crate utf8_ranges;

mod parser;
mod svg;
mod path;

use std::ffi::CStr;
use std::os::raw::c_char;

pub use parser::XMLDoc;
pub use svg::Node;

#[no_mangle]
pub fn pretty_print(ptr: *const c_char) {
    let input = unsafe { CStr::from_ptr(ptr) };
    let input = input.to_str().expect("Invalid input");

    let doc = XMLDoc::parse(input).expect("Failed to parse document");

    let svg = Node::from_xml_doc(doc).expect("XML doc is not valid SVG");
    println!("{}", svg);
}
