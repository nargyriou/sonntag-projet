use nom::float;
use parser::{Content, Element, XMLDoc};
use path::{comma_wsp, Bounding, BoundingBox, Path, Polygon};
use std::fmt;

#[derive(Debug)]
pub struct Root(Vec<Node>);

impl Root {
    pub fn get_rects(&self) -> Vec<[f32; 4]> {
        let root = if self.0.len() == 1 {
            if let Node::Group(ref children) = self.0[0] {
                children
            } else {
                &self.0
            }
        } else {
            &self.0
        };

        root.iter()
            .map(|child| child.bounding().to_rect())
            .collect()
    }
}

impl fmt::Display for Root {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Root")?;
        for child in &self.0 {
            child.pretty_print(f, 1)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Path(Vec<Polygon>),
    Group(Vec<Node>),
}

named!(
    parse_translate<(f32, f32)>,
    delimited!(
        tag!("translate("),
        separated_pair!(float, comma_wsp, float),
        tag!(")")
    )
);

fn translate(node: &Element) -> Option<(f32, f32)> {
    let attr = node.attributes
        .iter()
        .find(|a| a.name.as_str() == "transform")
        .map(|a| a.value.clone())?;
    let val = parse_translate(attr.as_bytes()).to_full_result().ok()?;
    Some(val)
}

impl Bounding for Node {
    fn bounding(&self) -> BoundingBox {
        match *self {
            Node::Path(ref polygons) => polygons.bounding(),
            Node::Group(ref nodes) => nodes.bounding(),
        }
    }
}

impl Node {
    pub fn from_xml_doc(doc: XMLDoc) -> Option<Root> {
        if doc.root.name != String::from("svg") {
            return None;
        }

        Some(Root(Node::list_from_children(doc.root.children)))
    }

    fn list_from_children(children: Vec<Content>) -> Vec<Self> {
        children
            .into_iter()
            .filter_map(|n| match n {
                Content::Element(e) => Some(e),
                _ => None,
            })
            .filter_map(Node::from_xml_node)
            .collect()
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        match *self {
            Node::Path(ref mut polygons) => for polygon in &mut *polygons {
                polygon.translate(dx, dy);
            },
            Node::Group(ref mut children) => for child in &mut *children {
                child.translate(dx, dy);
            },
        }
    }

    fn from_xml_node(xml_node: Element) -> Option<Self> {
        let delta = translate(&xml_node);
        let mut node = match xml_node.name.as_str() {
            "g" => Node::Group(Node::list_from_children(xml_node.children)),
            "path" => {
                let attr = xml_node
                    .attributes
                    .iter()
                    .find(|a| a.name.as_str() == "d")
                    .map(|a| a.value.clone())
                    .unwrap_or(String::new());

                let path = Path::parse(attr.as_str()).ok()?;
                Node::Path(path.draw())
            }
            _ => return None,
        };

        if let Some((dx, dy)) = delta {
            node.translate(dx, dy);
        }

        Some(node)
    }

    pub fn pretty_print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        match self {
            &Node::Path(ref polygons) => writeln!(
                f,
                "{0:1$}Path ({2} polygons, {3:?})",
                "",
                depth * 2,
                polygons.len(),
                polygons.bounding()
            ),
            &Node::Group(ref children) => {
                writeln!(f, "{0:1$}Group ({2:?})", "", depth * 2, children.bounding())?;
                for child in children {
                    child.pretty_print(f, depth + 1)?
                }
                Ok(())
            }
        }
    }
}
