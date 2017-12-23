use std::str;
use std::str::FromStr;
use std::fmt;
use nom::{is_digit, space, IError};

#[derive(Debug, PartialEq, Clone)]
struct Point(f32, f32);

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[derive(Debug, PartialEq)]
struct MoveTo {
    start: Point,
    cmd_type: CommandType,
    commands: Vec<DrawTo>,
}

impl MoveTo {
    fn pretty_print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        writeln!(f, "{0:1$}MoveTo {2}", "", depth * 2, self.start)?;
        let depth = depth + 1;
        for ref command in &self.commands {
            writeln!(f, "{0:1$}{2:?}", "", depth * 2, command)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Path(Vec<MoveTo>);

impl Path {
    pub fn parse(str: &str) -> Result<Self, IError<u32>> {
        path(str.as_bytes()).to_full_result()
    }

    pub fn pretty_print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        writeln!(f, "{0:1$}Path", "", depth * 2)?;
        for ref child in &self.0 {
            child.pretty_print(f, depth + 1)?;
        }
        Ok(())
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pretty_print(f, 0)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum CommandType {
    Relative,
    Absolute
}

macro_rules! cmd (
    ($i:expr, $abs:expr, $rel:expr) => (
        alt!($i, value!(CommandType::Absolute, char!($abs)) | value!(CommandType::Relative, char!($rel)))
    );
);

#[derive(Debug, PartialEq, Clone)]
enum DrawTo {
    ClosePath,
    LineTo(CommandType, Point),
    CurveTo(CommandType, Point, Point, Point),
}

named!(
    comma_wsp<()>,
    value!(
        (),
        preceded!(
            alt!(value!((), preceded!(space, opt!(char!(',')))) | value!((), char!(','))),
            opt!(space)
        )
    )
);

named!(
    number<f32>,
    map_opt!(
        take_while1!(|c| is_digit(c) || (c as char) == '.' || (c as char) == '-'
            || (c as char) == 'e'),
        |c| {
            let s = str::from_utf8(c).ok()?;
            f32::from_str(s).or(i32::from_str(s).map(|c| c as f32)).ok()
        }
    )
);

named!(coordinate<f32>, call!(number));

named!(
    coordinate_pair<Point>,
    do_parse!(x: coordinate >> opt!(comma_wsp) >> y: coordinate >> (Point(x, y)))
);

named!(
    moveto<(CommandType, Point)>,
    do_parse!(cmd_type: cmd!('M', 'm') >> opt!(space) >> point: coordinate_pair >> ((cmd_type, point)))
);

named!(
    lineto<Vec<DrawTo>>,
    do_parse!(
        cmd_type: cmd!('L', 'l') >> 
        opt!(space) >> 
        points: map!(
            coordinate_list, 
            |v| v.iter()
                    .cloned()
                    .map(|p| DrawTo::LineTo(cmd_type, p))
                    .collect()
        ) >> 
        (points)
    )
);
named!(
    curveto<Vec<DrawTo>>,
    do_parse!(
        cmd_type: cmd!('C', 'c') >> 
        commands: ws!(many1!(curveto_argument)) >> 
        (
            commands.iter()
                .cloned()
                .map(|(p1, p2, p3)| DrawTo::CurveTo(cmd_type, p1, p2, p3))
                .collect()
        )
    )
);

named!(
    curveto_argument<(Point, Point, Point)>,
    do_parse!(
        p1: coordinate_pair >>
        opt!(comma_wsp) >>
        p2: coordinate_pair >>
        opt!(comma_wsp) >>
        p3: coordinate_pair >>
        ((p1, p2, p3))
    )
);

named!(coordinate_list<Vec<Point>>, many0!(ws!(coordinate_pair)));

named!(
    closepath<DrawTo>,
    value!(DrawTo::ClosePath, alt!(char!('z') | char!('Z')))
);

named!(
    commands<Vec<DrawTo>>,
    map!(
        many0!(ws!(drawto_command)),
        |commands| commands.iter().flat_map(|c| c.clone()).collect()
    )
);

named!(
    drawto_command<Vec<DrawTo>>,
    alt!(map!(closepath, |v| vec![v]) | lineto | curveto)
);

named!(
    moveto_drawto_command_group<MoveTo>,
    do_parse!(
        cmd: moveto >> 
        opt!(space) >> 
        implicite_linetos: coordinate_list >>
        commands: commands >> 
        ({
            let (cmd_type, start) = cmd;
            let commands = implicite_linetos.iter()
                .map(|p| DrawTo::LineTo(cmd_type, p.clone()))
                .chain(commands)
                .collect();

            MoveTo {
                cmd_type,
                start,
                commands: commands,
            }
        })
    )
);

named!(
    path<Path>,
    do_parse!(commands: ws!(many0!(ws!(moveto_drawto_command_group))) >> (Path(commands)))
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_pair() {
        assert_eq!(
            coordinate_pair(b"12.5,3.2").to_result().unwrap(),
            Point(12.5, 3.2)
        );
        assert_eq!(
            coordinate_pair(b"12.5 3.2").to_result().unwrap(),
            Point(12.5, 3.2)
        );
    }

    #[test]
    fn test_moveto() {
        assert_eq!(moveto(b"M 12.5,3.2").to_result().unwrap(), (CommandType::Absolute, Point(12.5, 3.2)));
    }

    #[test]
    fn test_curveto_argument() {
        assert_eq!(
            curveto(b"c 1,1 2,2 3,3").to_result().unwrap(),
            vec![DrawTo::CurveTo(CommandType::Relative, Point(1., 1.), Point(2., 2.), Point(3., 3.))]
        );
    }

    #[test]
    fn test_moveto_drawto_command_group() {
        let path = b"m 12.5 3.2 l 5. 4. 8. 2. c 1,1 2,2 3,3 z";
        assert_eq!(
            moveto_drawto_command_group(path).to_result().unwrap(),
            MoveTo {
                cmd_type: CommandType::Relative,
                start: Point(12.5, 3.2),
                commands: vec![
                    DrawTo::LineTo(CommandType::Relative, Point(5., 4.)),
                    DrawTo::LineTo(CommandType::Relative, Point(8., 2.)),
                    DrawTo::CurveTo(CommandType::Relative, Point(1., 1.), Point(2., 2.), Point(3., 3.)),
                    DrawTo::ClosePath,
                ],
            }
        );
    }

    #[test]
    fn test_path() {
        let path = "M 12.5 3.2 4 3 m 1 2 3 4";
        assert_eq!(
            Path::parse(path),
            Ok(Path(vec![
                MoveTo {
                    cmd_type: CommandType::Absolute,
                    start: Point(12.5, 3.2),
                    commands: vec![DrawTo::LineTo(CommandType::Absolute, Point(4., 3.))],
                },
                MoveTo {
                    cmd_type: CommandType::Relative,
                    start: Point(1., 2.),
                    commands: vec![DrawTo::LineTo(CommandType::Relative, Point(3., 4.))],
                },
            ]))
        )
    }
}
