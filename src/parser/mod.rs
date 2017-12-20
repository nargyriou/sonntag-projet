use std::str;
use std::str::FromStr;
use nom::{is_alphabetic, is_alphanumeric};

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

macro_rules! attr_value(
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        alt!($i,
            delimited!(tag!("\""), $submac!($($args)* ), tag!("\""))
            | delimited!(tag!("'"), $submac!($($args)* ), tag!("'"))
        )
    );
    ($i:expr, $f:expr) => (
        attr_value!($i, call!($f))
    );
);

named!(
    yes_no<bool>,
    alt!(value!(true, tag!("yes")) | value!(false, tag!("no")))
);

named!(sd_decl<bool>, ws!(named_attr!("standalone", yes_no)));

fn is_enc_name(chr: u8) -> bool {
    is_alphanumeric(chr) || ['.', '-', '_'].contains(&(chr as char))
}

named!(
    enc_name<String>,
    map_res!(
        map_res!(
            preceded!(
                peek!(take_while1_s!(is_alphabetic)),
                take_while1_s!(is_enc_name)
            ),
            str::from_utf8
        ),
        FromStr::from_str
    )
);

named!(enc_decl<String>, ws!(named_attr!("encoding", enc_name)));

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

named!(
    version_num<String>,
    map_res!(
        map_res!(take_while1_s!(is_version_num), str::from_utf8),
        FromStr::from_str
    )
);

named!(
    version_decl<String>,
    ws!(named_attr!("version", version_num))
);

#[derive(Debug, PartialEq)]
struct XMLDecl {
    version: String,
    encoding: String,
    standalone: bool,
}

named!(
    xml_decl<XMLDecl>,
    delimited!(
        tag!("<?xml"),
        do_parse!(
            version: preceded!(tag!(" "), version_decl)
                >> encoding: alt!(preceded!(tag!(" "), enc_decl) | value!(String::from("UTF-8")))
                >> standalone: alt!(preceded!(tag!(" "), sd_decl) | value!(false))
                >> (XMLDecl {
                    version: version,
                    encoding: encoding,
                    standalone: standalone,
                })
        ),
        tag!("?>")
    )
);

#[derive(Debug, PartialEq, Eq, Clone)]
struct Comment(String);

named!(
    comment<Comment>,
    map_res!(
        delimited!(tag!("<!--"), take_until_s!("--"), tag!("-->")),
        |c| str::from_utf8(c).map(|c| Comment(String::from(c)))
    )
);

named!(prolog_misc<Vec<Comment>>, ws!(many0!(ws!(comment))));

#[derive(Debug, PartialEq)]
pub struct XMLProlog {
    decl: Option<XMLDecl>,
    comments: Vec<Comment>,
    doctype: Option<Doctype>,
}

named!(
    pub xml_prolog<XMLProlog>,
    do_parse!(
        decl: opt!(xml_decl) >>
        comments1: prolog_misc >>
        doctype: ws!(opt!(doctype_decl)) >>
        comments2: prolog_misc >>
        (XMLProlog {
            decl: decl,
            doctype: doctype,
            comments: [&comments1[..], &comments2[..]].concat()
        })
    )
);

#[derive(Debug, PartialEq)]
pub struct Doctype {
    name: String,
}

named!(
    doctype_decl<Doctype>,
    delimited!(
        tag!("<!DOCTYPE"),
        do_parse!(name: ws!(name) >> (Doctype { name: name })),
        tag!(">")
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
            <!-- Ho. -->
        ",
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
}
