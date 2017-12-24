use std::str;
use std::str::FromStr;
use std::fmt;
use std::f32;
use nom::{is_digit, space, IError};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point(pub f32, pub f32);

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl Point {
    fn translate(self, to: Point) -> Point {
        Point(self.0 + to.0, self.1 + to.1)
    }

    fn adjust(self, cmd_type: CommandType, start: Point) -> Point {
        if cmd_type.is_relative() {
            self.translate(start)
        } else {
            self
        }
    }

    fn min(self, other: Point) -> Point {
        Point(
            f32::min(self.0, other.0), 
            f32::min(self.1, other.1)
        )
    }

    fn max(self, other: Point) -> Point {
        Point(
            f32::max(self.0, other.0), 
            f32::max(self.1, other.1)
        )
    }
}

#[derive(Debug, PartialEq)]
struct MoveTo {
    start: Point,
    cmd_type: CommandType,
    commands: Vec<DrawTo>,
}

fn bezier(p1: Point, p2: Point, ctrl1: Point, ctrl2: Point, precision: usize) -> Vec<Point> {
    let mut path = Vec::with_capacity(precision - 1);
    let step = 1. / (precision as f32);
    for i in 1..precision {
        let t = (i as f32) * step;

        let x = (1. - t).powi(3) * p1.0 
            + 3. * (1. - t).powi(2) * t * ctrl1.0
            + 3. * (1. - t) * t.powi(2) * ctrl2.0
            + t.powi(3) * p2.0;

        let y = (1. - t).powi(3) * p1.1 
            + 3. * (1. - t).powi(2) * t * ctrl1.1
            + 3. * (1. - t) * t.powi(2) * ctrl2.1
            + t.powi(3) * p2.1;

        path.push(Point(x, y));
    }
    path
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

    fn draw(&self, start: Point) -> (Point, Polygon) {
        let start = self.start.adjust(self.cmd_type, start);
        let mut points = vec![start];
        let mut current = start;

        for command in &self.commands {
            match *command {
                DrawTo::LineTo(cmd_type, p) => {
                    let p = p.adjust(cmd_type, current);
                    points.push(p);
                    current = p;
                }
                DrawTo::ClosePath => {
                    points.push(start);
                    current = start;
                }
                DrawTo::CurveTo(cmd_type, ctrl1, ctrl2, p2) => {
                    let p1 = current;
                    let p2 = p2.adjust(cmd_type, current);
                    let ctrl1 = ctrl1.adjust(cmd_type, current);
                    let ctrl2 = ctrl2.adjust(cmd_type, current);

                    let curve = bezier(p1, p2, ctrl1, ctrl2, 10);
                    points.extend(curve);

                    points.push(p2);
                    current = p2;
                }
            }
        }

        let polygon = Polygon::from(points);

        (current, polygon)
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox(Point, Point);

impl BoundingBox {
    pub fn to_rect(&self) -> [f32; 4] {
        let (p1, p2) = (self.0, self.1);
        let (x, y) = (p1.0, p1.1);
        let (h, w) = (p2.0 - p1.0, p2.1 - p1.1);
        [x, y, h, w]
    }
}

pub trait Bounding {
    fn bounding(&self) -> BoundingBox;
}

impl Bounding for BoundingBox {
    fn bounding(&self) -> BoundingBox {
        self.clone()
    }
}

impl<T: Bounding> Bounding for (T, T) {
    fn bounding(&self) -> BoundingBox {
        let b1 = self.0.bounding();
        let b2 = self.1.bounding();
        BoundingBox(
            b1.0.min(b2.0),
            b1.1.max(b2.1),
        )
    }
}

impl<T: Bounding> Bounding for Vec<T> {
    fn bounding(&self) -> BoundingBox {
        let mut bounding = BoundingBox(
            Point(f32::MAX, f32::MAX),
            Point(f32::MIN, f32::MIN),
        );

        for boxes in self {
            bounding = (bounding, boxes.bounding()).bounding();
        }

        bounding
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub closed: bool,
    pub points: Vec<Point>
}

impl Polygon {
    fn from(mut points: Vec<Point>) -> Self {
        let closed = points.first() == points.last();

        if closed {
            points.pop();
        }

        Polygon {
            closed,
            points
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        for point in &mut self.points {
            *point = point.translate(Point(dx, dy));
        }
    }
}

impl Bounding for Polygon {
    fn bounding(&self) -> BoundingBox {
        let mut min = Point(f32::MAX, f32::MAX);
        let mut max = Point(f32::MIN, f32::MIN);
        for point in &self.points {
            min = min.min(*point);
            max = max.max(*point);
        }
        BoundingBox(min, max)
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

        writeln!(f, "{:?}", self.draw())
    }

    pub fn draw(&self) -> Vec<Polygon> {
        let mut start = Point(0., 0.);
        let mut polygons = vec![];

        for path in &self.0 {
            let (next, polygon) = path.draw(start);
            start = next;
            polygons.push(polygon);
        }

        polygons
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

impl CommandType {
    fn is_relative(self) -> bool {
        self == CommandType::Relative
    }
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
    pub comma_wsp<()>,
    value!(
        (),
        preceded!(
            alt!(value!((), preceded!(space, opt!(char!(',')))) | value!((), char!(','))),
            opt!(space)
        )
    )
);

named!(
    pub number<f32>,
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
    closepath<Vec<DrawTo>>,
    value!(vec![DrawTo::ClosePath], alt!(char!('z') | char!('Z')))
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
    alt!(closepath | lineto | curveto)
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
