#[macro_use]
extern crate nom;

mod parser;
mod svg;
mod path;

use std::mem;
use std::ffi::CStr;
use std::os::raw::c_char;

pub use parser::XMLDoc;
pub use svg::{Node, Root};
pub use path::{Bounding, Path};

#[repr(C)]
pub struct Drawing {
    size: u32,
    groups: *const Group,
}

#[repr(C)]
pub struct Group {
    x: f32,
    y: f32,
    h: f32,
    w: f32,
    size: u32,
    polygons: *const Polygon,
}

#[repr(C)]
pub struct Polygon {
    size: u32,
    vertices: *const Point,
}

#[repr(C)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Drawing {
    fn from_root(root: Root) -> Self {
        let groups = root.simplify();
        let groups: Vec<_> = groups
            .into_iter()
            .map(|group| {
                let rect = group.bounding().to_rect();
                let polygons: Vec<_> = group
                    .into_iter()
                    .map(|polygon| {
                        let vertices: Vec<_> = polygon
                            .points
                            .into_iter()
                            .map(|p| Point { x: p.0, y: p.1 })
                            .collect();
                        let ret = Polygon {
                            size: vertices.len() as u32,
                            vertices: vertices.as_ptr(),
                        };

                        mem::forget(ret.size);
                        mem::forget(ret.vertices);
                        mem::forget(vertices);

                        ret
                    })
                    .collect();
                let ret = Group {
                    x: rect[0],
                    y: rect[1],
                    h: rect[2],
                    w: rect[3],
                    size: polygons.len() as u32,
                    polygons: polygons.as_ptr(),
                };

                mem::forget(ret.size);
                mem::forget(ret.polygons);
                mem::forget(polygons);
                mem::forget(rect);

                ret
            })
            .collect();

        let ret = Drawing {
            size: groups.len() as u32,
            groups: groups.as_ptr(),
        };

        mem::forget(ret.size);
        mem::forget(ret.groups);
        mem::forget(groups);
        ret
    }
}

#[no_mangle]
pub extern "C" fn pretty_print(ptr: *const c_char) {
    let input = unsafe { CStr::from_ptr(ptr) };
    let input = input.to_str().expect("Invalid input");

    let doc = XMLDoc::parse(input).expect("Failed to parse document");

    let svg = Node::from_xml_doc(doc).expect("XML doc is not valid SVG");
    println!("{}", svg);
}

#[no_mangle]
pub extern "C" fn parse(ptr: *const c_char) -> Drawing {
    let input = unsafe { CStr::from_ptr(ptr) };
    let input = input.to_str().expect("Invalid input");

    let doc = XMLDoc::parse(input).expect("Failed to parse document");

    let root = Node::from_xml_doc(doc).expect("XML doc is not valid SVG");
    Drawing::from_root(root)
}
