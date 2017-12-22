use std::fmt;
use std::str;
use std::str::FromStr;
use std::iter::FromIterator;
use nom::{is_alphabetic, is_alphanumeric, multispace, IResult, IError, Needed};

macro_rules! named_attr(
    ($i:expr, $name:expr, $submac:ident!( $($args:tt)* )) => (
        map!($i, attr!(tag!($name), $submac!( $($args)* )), |(_, v)| v)
    );

    ($i:expr, $f:expr, $g:expr) => (
        named_attr!($i, $f, call!($g));
    );
);

macro_rules! attr(
    ($i:expr, $submac:ident!( $($args:tt)* ), $submac2:ident!( $($args2:tt)* )) => (
        do_parse!($i,
            attr: $submac!($($args)*) >>
            ws!(tag!("=")) >>
            value: attr_value!($submac2!($($args2)*)) >>
            (attr, value)
        )
    );

    ($i:expr, $submac:ident!( $($args:tt)* ), $g:expr) => (
        attr!($i, $submac!($($args)*), call!($g));
    );

    ($i:expr, $f:expr, $submac:ident!( $($args:tt)* )) => (
        attr!($i, call!($f), $submac!($($args)*));
    );

    ($i:expr, $f:expr, $g:expr) => (
        attr!($i, call!($f), call!($g));
    );
);

macro_rules! attr_inner {
    ($i:expr, $c:expr, $submac:ident!( $($args:tt)* )) => {
        match is_not!($i, $c) {
            IResult::Done(i1, o1) => {
                match $submac!(o1, $($args)* ) {
                    IResult::Done(i2, o2) => {
                        if i2.len() > 0 {
                            IResult::Incomplete(Needed::Unknown)
                        } else {
                            IResult::Done(i1, o2)
                        }
                    },
                    e => e,
                }
            },
            e => e,
        }
    };

    ($i:expr, $f:expr, $g: expr) => (
        attr_value!($i, $f, call!($g))
    );
}

macro_rules! attr_value(
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        alt!($i,
            delimited!(char!('"'), escaped!(attr_inner!("\"", $submac!( $($args)* )), '\\', char!('"')), char!('"'))
            |
            delimited!(char!('\''), escaped!(attr_inner!("'", $submac!( $($args)* )), '\\', char!('\'')), char!('\''))
        )
    );
    ($i:expr, $f:expr) => (
        attr_value!($i, call!($f))
    );
);

macro_rules! space_first(
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        do_parse!(
            $i,
            multispace >>
            sub: $submac!( $($args)* ) >>
            (sub)
        )
    );
    ($i:expr, $j:expr) => (
        space_first!($i, call!($j))
    );
);

/// Matches "yes" or "no"
named!(yes_no, alt!(tag!("yes") | tag!("no")));

/// Matches `standalone="yes|no"`
named!(
    sd_decl<bool>,
    map_opt!(
        named_attr!("standalone", yes_no),
        |v| match str::from_utf8(v) {
            Ok("yes") => Some(true),
            Ok("no") => Some(false),
            _ => None,
        }
    )
);

fn is_enc_name(chr: u8) -> bool {
    is_alphanumeric(chr) || ['.', '-', '_'].contains(&(chr as char))
}

named!(
    enc_name,
    preceded!(
        peek!(take_while1_s!(is_alphabetic)),
        take_while1_s!(is_enc_name)
    )
);

/// Matches `encoding="…"`
named!(
    enc_decl<String>,
    map_res!(
        map_res!(named_attr!("encoding", enc_name), str::from_utf8),
        FromStr::from_str
    )
);

fn is_name_char(chr: u8) -> bool {
    is_alphanumeric(chr) || ['.', '-', '_', ':'].contains(&(chr as char))
}

fn is_name_start(chr: u8) -> bool {
    is_alphabetic(chr) || ['_', ':'].contains(&(chr as char))
}

fn is_version_num(chr: u8) -> bool {
    is_alphanumeric(chr) || ['.', '-', '_', ':'].contains(&(chr as char))
}

named!(
    name<String>,
    map_res!(
        map_res!(
            preceded!(
                peek!(take_while1_s!(is_name_start)),
                take_while1_s!(is_name_char)
            ),
            str::from_utf8
        ),
        FromStr::from_str
    )
);

named!(version_num, take_while1_s!(is_version_num));

/// Matches `version="…"`
named!(
    version_decl<String>,
    map_res!(
        map_res!(named_attr!("version", version_num), str::from_utf8),
        FromStr::from_str
    )
);

/// Used to store the XML declaration `<?xml … ?>`
#[derive(Debug, PartialEq)]
struct XMLDecl {
    version: String,
    encoding: String,
    standalone: bool,
}

/// Matches `<?xml version="…" encoding="…" standalone="…" ?>` to a XMLDecl structure
named!(
    xml_decl<XMLDecl>,
    delimited!(
        tag!("<?xml"),
        do_parse!(
            version: space_first!(version_decl)
                >> encoding: alt!(space_first!(enc_decl) | value!(String::from("UTF-8")))
                >> standalone: alt!(space_first!(sd_decl) | value!(false))
                >> opt!(multispace) >> (XMLDecl {
                version,
                encoding,
                standalone,
            })
        ),
        tag!("?>")
    )
);

/// Store a comment
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Comment(pub String);

/// Matches `<!-- … -->`
named!(
    comment<Comment>,
    map_res!(
        delimited!(tag!("<!--"), take_until_s!("--"), tag!("-->")),
        |c| str::from_utf8(c).map(|c| Comment(String::from(c)))
    )
);

named!(comment_list<Vec<Comment>>, ws!(many0!(ws!(comment))));

#[derive(Debug, PartialEq)]
struct XMLProlog {
    decl: Option<XMLDecl>,
    comments: Vec<Comment>,
    doctype: Option<Doctype>,
}

named!(
    xml_prolog<XMLProlog>,
    do_parse!(
        decl: opt!(xml_decl) >>
        comments_before: comment_list >>
        doctype: ws!(opt!(doctype_decl)) >>
        comments_after: comment_list >>
        (XMLProlog {
            decl,
            doctype,
            comments: [comments_before, comments_after].concat(),
        })
    )
);

/// Store the DOCTYPE
#[derive(Debug, PartialEq)]
struct Doctype {
    name: String,
}

/// Matches `<!DOCTYPE …>`
named!(
    doctype_decl<Doctype>,
    delimited!(
        tag!("<!DOCTYPE "),
        do_parse!(name: ws!(name) >> (Doctype { name })),
        tag!(">")
    )
);

/// Store a node attribute
#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl Attribute {
    fn pretty_print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        write!(f, "{0:1$}", "", depth * 2)?;
        let mut value = self.value.clone();
        value.truncate(40);
        writeln!(f, "Attribute: {} = {}", self.name, value)
    }
}

/// Matches `key="value"`
named!(
    attribute<Attribute>,
    map_res!(
        attr!(name, is_not!("<&")),
        |(name, value)| -> Result<Attribute, str::Utf8Error> {
            Ok(Attribute {
                name,
                value: String::from(str::from_utf8(value)?),
            })
        }
    )
);

#[derive(Debug, PartialEq)]
pub enum Content {
    Comment(Comment),
    Element(Element),
    Chars(String),
}

impl Content {
    fn pretty_print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        match self {
            &Content::Comment(ref c) => {
                writeln!(f, "{0:1$}{2:?}", "", depth * 2, c)
            }
            &Content::Element(ref e) => e.pretty_print(f, depth),
            &Content::Chars(ref s) => {
                let mut content = s.clone();
                content.truncate(40);
                writeln!(f, "{0:1$}{2}", "", depth * 2, content)
            }
        }
    }
}

/// A Node
#[derive(Debug, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Content>,
}

impl Element {
    fn pretty_print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        writeln!(f, "{0:1$}Element: {2}", "", depth * 2, self.name)?;
        let depth = depth + 1;
        for attr in &self.attributes {
            attr.pretty_print(f, depth)?;
        }
        for child in &self.children {
            child.pretty_print(f, depth)?;
        }
        Ok(())
    }
}

/// Matches a node (empty tag or tag pair)
named!(
    element<Element>,
    preceded!(peek!(not!(tag!("</"))), alt!(empty_elem_tag | tag_pair))
);

/// Matches a node's content (child node, string or comment)
named!(
    content<Content>,
    alt!(
        map!(node_value, |c| Content::Chars(c)) | map!(element, |e| Content::Element(e))
            | map!(cdata, |c| Content::Chars(c)) | map!(comment, |c| Content::Comment(c))
    )
);

/// Matches `<tag attr="value" />`
named!(
    empty_elem_tag<Element>,
    do_parse!(
        tag!("<") >> name: name >> attributes: many0!(space_first!(attribute)) >> opt!(multispace)
            >> tag!("/>") >> (Element {
            name,
            attributes,
            children: Vec::new(),
        })
    )
);

/// Matches `<tag attr="value">…</tag>`
named!(
    tag_pair<Element>,
    do_parse!(
        tag!("<") >> tag_name: name >> attributes: many0!(space_first!(attribute))
            >> opt!(multispace) >> tag!(">") >> children: ws!(many0!(ws!(content)))
            >> tag!("</") >> tag!(tag_name.as_str()) >> opt!(multispace) >> tag!(">")
            >> (Element {
                name: tag_name,
                attributes,
                children,
            })
    )
);

/// Matches `<![CDATA[ … ]]>`
named!(
    cdata<String>,
    map_res!(
        map_res!(
            delimited!(tag!("<![CDATA["), take_until_s!("]]>"), tag!("]]>")),
            str::from_utf8
        ),
        FromStr::from_str
    )
);

named!(char_data<char>, none_of!("<&"));

/// Matches entity references, like `&amp;`
named!(
    entity_ref<char>,
    delimited!(
        char!('&'),
        alt!(
            value!('"', tag!("quot")) | value!('&', tag!("amp")) | value!('\'', tag!("apos"))
                | value!('<', tag!("lt")) | value!('>', tag!("gt"))
        ),
        char!(';')
    )
);

/// Matches a node value (with entity refs converted)
named!(
    node_value<String>,
    map!(ws!(many1!(alt!(char_data | entity_ref))), String::from_iter)
);

#[derive(Debug, PartialEq)]
pub struct XMLDoc {
    prolog: XMLProlog,
    pub root: Element,
    misc: Vec<Comment>,
}

impl XMLDoc {
    pub fn parse(doc: &str) -> Result<Self, IError<u32>> {
        xml_doc(doc.as_bytes()).to_full_result()
    }
}

impl fmt::Display for XMLDoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.prolog)?;
        self.root.pretty_print(f, 0)?;

        for comment in &self.misc {
            writeln!(f, "{:?}", comment)?;
        }

        Ok(())
    }
}

named!(
    pub xml_doc<XMLDoc>,
    do_parse!(
        prolog: xml_prolog >>
        root: element >>
        misc: ws!(comment_list) >>
        (XMLDoc { prolog, root, misc })
    )
);

#[cfg(test)]
mod tests {
    use nom::IResult;
    use parser::*;

    #[test]
    fn parse_version() {
        assert_eq!(
            version_decl(b"version='123'"),
            IResult::Done(&b""[..], String::from("123"))
        );
    }

    #[test]
    fn parse_encoding() {
        assert_eq!(
            enc_decl(b"encoding='UTF-8'"),
            IResult::Done(&b""[..], String::from("UTF-8"))
        );
    }

    #[test]
    fn parse_standalone() {
        assert_eq!(sd_decl(b"standalone='yes'"), IResult::Done(&b""[..], true));
        assert_eq!(
            sd_decl(b"standalone=\"no\""),
            IResult::Done(&b""[..], false)
        );
    }

    #[test]
    fn parse_xml_decl() {
        assert_eq!(
            xml_decl(b"<?xml version='1.0' ?>"),
            IResult::Done(
                &b""[..],
                XMLDecl {
                    version: String::from("1.0"),
                    encoding: String::from("UTF-8"),
                    standalone: false,
                }
            )
        );
    }

    #[test]
    fn parse_comment() {
        assert_eq!(
            comment(b"<!-- comment -->"),
            IResult::Done(&b""[..], Comment(String::from(" comment ")))
        );
    }

    #[test]
    fn parse_xml_prolog() {
        let prolog = xml_prolog(
            b"<?xml version='1.0' ?>
            <!-- Hey. -->
            <!DOCTYPE html>
            <!-- Ho. -->",
        );

        let comments = vec![
            Comment(String::from(" Hey. ")),
            Comment(String::from(" Ho. ")),
        ];

        let expected = XMLProlog {
            decl: Some(XMLDecl {
                version: String::from("1.0"),
                encoding: String::from("UTF-8"),
                standalone: false,
            }),
            comments: comments,
            doctype: Some(Doctype {
                name: String::from("html"),
            }),
        };

        assert_eq!(prolog, IResult::Done(&b""[..], expected));
    }

    #[test]
    fn parse_doctype_decl() {
        assert_eq!(
            doctype_decl(b"<!DOCTYPE html>"),
            IResult::Done(
                &b""[..],
                Doctype {
                    name: String::from("html"),
                }
            )
        );
    }

    #[test]
    fn parse_attribute() {
        assert_eq!(
            attribute(b"src='test'"),
            IResult::Done(
                &b""[..],
                Attribute {
                    name: String::from("src"),
                    value: String::from("test"),
                }
            )
        );
    }

    #[test]
    fn parse_empty_elem_tag() {
        let tag = empty_elem_tag(b"<img src='test' />");
        let expected = Element {
            name: String::from("img"),
            attributes: vec![
                Attribute {
                    name: String::from("src"),
                    value: String::from("test"),
                },
            ],
            children: vec![],
        };

        assert_eq!(tag, IResult::Done(&b""[..], expected));
    }

    #[test]
    fn parse_cdata() {
        let cdata = cdata(b"<![CDATA[<i>test</i>]]>");
        assert_eq!(cdata, IResult::Done(&b""[..], String::from("<i>test</i>")));
    }

    #[test]
    fn parse_tag_pair() {
        let tag = tag_pair(
            b"<p>
                <img src='bleh' width=\"42\" />
                <!-- Separator -->
                <i>italic</i>
            </p>",
        );

        let expected = Element {
            name: String::from("p"),
            attributes: vec![],
            children: vec![
                Content::Element(Element {
                    name: String::from("img"),
                    attributes: vec![
                        Attribute {
                            name: String::from("src"),
                            value: String::from("bleh"),
                        },
                        Attribute {
                            name: String::from("width"),
                            value: String::from("42"),
                        },
                    ],
                    children: vec![],
                }),
                Content::Comment(Comment(String::from(" Separator "))),
                Content::Element(Element {
                    name: String::from("i"),
                    attributes: vec![],
                    children: vec![Content::Chars(String::from("italic"))],
                }),
            ],
        };

        let tag = tag.map_err(|e| {
            println!("{}", e.description());
            e
        });

        assert_eq!(tag, IResult::Done(&b""[..], expected));
    }

    #[test]
    fn parse_xml_doc() {
        let doc = xml_doc(
            b"<?xml version='1.0' ?>
            <!-- Hey. -->
            <!DOCTYPE html>
            <!-- Ho. -->
            <svg>
                <img src='test' width='42' />
                <!-- Separator -->
                <hr />
            </svg>
            <!-- End. -->",
        );

        let prolog = XMLProlog {
            decl: Some(XMLDecl {
                version: String::from("1.0"),
                encoding: String::from("UTF-8"),
                standalone: false,
            }),
            comments: vec![
                Comment(String::from(" Hey. ")),
                Comment(String::from(" Ho. ")),
            ],
            doctype: Some(Doctype {
                name: String::from("html"),
            }),
        };

        let root = Element {
            name: String::from("svg"),
            attributes: vec![],
            children: vec![
                Content::Element(Element {
                    name: String::from("img"),
                    attributes: vec![
                        Attribute {
                            name: String::from("src"),
                            value: String::from("test"),
                        },
                        Attribute {
                            name: String::from("width"),
                            value: String::from("42"),
                        },
                    ],
                    children: vec![],
                }),
                Content::Comment(Comment(String::from(" Separator "))),
                Content::Element(Element {
                    name: String::from("hr"),
                    attributes: vec![],
                    children: vec![],
                }),
            ],
        };

        let misc = vec![Comment(String::from(" End. "))];

        let expected = XMLDoc { prolog, root, misc };

        assert_eq!(doc, IResult::Done(&b""[..], expected));
    }
}
