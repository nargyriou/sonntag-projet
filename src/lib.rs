#[macro_use]
extern crate nom;
extern crate utf8_ranges;

mod parser;
mod svg;
mod path;

use std::mem;
use std::ffi::CStr;
use std::os::raw::c_char;

pub use parser::XMLDoc;
pub use svg::{Node, Root};
pub use path::Path;

#[repr(C)]
pub struct Rects {
    size: u32,
    ptr: *const [f32; 4],
}

impl Rects {
    fn from_root(root: Root) -> Self {
        let rects = root.get_rects();
        let ret = Rects {
            size: rects.len() as u32,
            ptr: rects.as_ptr(),
        };
        mem::forget(ret.ptr);
        ret
    }
}

#[no_mangle]
pub extern "C" fn pretty_print(ptr: *const c_char) {
    let input = unsafe { CStr::from_ptr(ptr) };
    let input = input.to_str().expect("Invalid input");

    let doc = XMLDoc::parse(input).expect("Failed to parse document");

    let svg = Node::from_xml_doc(doc).expect("XML doc is not valid SVG");
    println!("{:?}", svg.get_rects());
}

#[no_mangle]
pub extern "C" fn get_bounding_rects(ptr: *const c_char) -> Rects {
    let input = unsafe { CStr::from_ptr(ptr) };
    let input = input.to_str().expect("Invalid input");

    let doc = XMLDoc::parse(input).expect("Failed to parse document");

    let root = Node::from_xml_doc(doc).expect("XML doc is not valid SVG");
    Rects::from_root(root)
}
