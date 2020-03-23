use std::{cmp, f32, mem};

/// A generic 2D software rasterizer.
pub struct Cloth<T: Target> {
    path: Path,
    target: T,
    fill: Color,
}

impl<T: Target> Cloth<T> {
    /// Initializes a new rasterizer that will use the given target as its output.
    pub fn new(target: T) -> Cloth<T> {
        let path = Path::new();
        let fill = [0, 0, 0, 255];
        Cloth { target, path, fill }
    }

    /// Decomposes the `Cloth` into its inner `Target`.
    pub fn into_target(self) -> T {
        self.target
    }

    /// Sets the active fill color.
    pub fn set_fill(&mut self, fill: Color) {
        self.fill = fill;
    }

    /// Starts a new active path.
    pub fn begin_path(&mut self) {
        self.path = Path::new();
    }

    /// Closes the active path.
    pub fn close_path(&mut self) {
        self.path.close();
    }

    /// Begins a new sub-path on the active path at the specified point.
    pub fn move_to(&mut self, x: f32, y: f32) {
        let point = Point::new(x, y);
        self.path.start = point;
        self.path.add(Subpath::Move(point));
    }

    /// Adds a straight line to the current sub-path by connecting the sub-path's last point to the
    /// specified point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.add(Subpath::Line(Point::new(x, y)));
    }

    /// Fills the active path with the active fill color.
    pub fn fill(&mut self) {
        self.close_path();
        let lines = self.path.to_lines();
        let bounds = lines.bounds();
        let mut y = f32::max(0.0, f32::min(bounds.br.y, self.target.height() as f32));
        while y >= bounds.tl.y {
            // TODO: The problem with this scanline rasterization method is that lines that exist on
            // non-integer y-coordinates don't get antialiased:
            let xs = lines.x_intersections(y + 0.5);
            for pair in xs.chunks(2).filter(|c| c.len() == 2) {
                let start = cmp::max(0, pair[0].floor() as u32);
                let end = cmp::min(pair[1].floor() as u32, self.target.width() - 1);
                let falpha = u2f(self.fill[3]);
                let mut start_fill = self.fill;
                start_fill[3] = f2u(falpha * (1.0 - pair[0].fract()));
                let mut end_fill = self.fill;
                end_fill[3] = f2u(falpha * pair[1].fract());
                for x in start..=end {
                    self.fill_pixel(x, y as u32, if x == start {
                        start_fill
                    } else if x == end {
                        end_fill
                    } else {
                        self.fill
                    });
                }
            }
            y -= 1.0;
        }
    }

    fn fill_pixel(&mut self, x: u32, y: u32, rgba: Color) {
        self.target.set_pixel(x, y, self.blend(self.target.get_pixel(x, y), rgba));
    }

    fn blend(&self, old: Color, new: Color) -> Color {
        fn comp(ca: f32, cb: f32, aa: f32, ab: f32) -> f32 {
            (ca * aa + cb * ab * (1.0 - aa)) / (aa + ab * (1.0 - aa))
        }

        let old: [f32; 4] = [u2f(old[0]), u2f(old[1]), u2f(old[2]), u2f(old[3])];
        let new: [f32; 4] = [u2f(new[0]), u2f(new[1]), u2f(new[2]), u2f(new[3])];

        [
            f2u(comp(new[0], old[0], new[3], old[3])),
            f2u(comp(new[1], old[1], new[3], old[3])),
            f2u(comp(new[2], old[2], new[3], old[3])),
            f2u(new[3] + (1.0 - new[3]) * old[3]),
        ]
    }
}

/// A drawable raster target.
pub trait Target {
    /// The width of the target image.
    fn width(&self) -> u32;
    /// The height of the target image.
    fn height(&self) -> u32;
    /// Returns the color for the given pixel.
    fn get_pixel(&self, x: u32, y: u32) -> Color;
    /// Sets the color for the given pixel.
    fn set_pixel(&mut self, x: u32, y: u32, rgba: Color);
}

/// A 32-bit RGBA color.
///
/// In order, the components of these arrays are red, green, blue, and alpha. Each component ranges
/// from 0 to 255.
pub type Color = [u8; 4];

struct Path {
    start: Point,
    subpaths: Vec<Subpath>,
    is_closed: bool,
}

impl Path {
    fn new() -> Path {
        Path {
            start: Point::new(0.0, 0.0),
            subpaths: Vec::new(),
            is_closed: false,
        }
    }

    fn add(&mut self, subpath: Subpath) {
        self.is_closed = false;
        self.subpaths.push(subpath);
    }

    fn close(&mut self) {
        if !self.is_closed {
            self.is_closed = true;
            self.subpaths.push(Subpath::Line(self.start));
        }
    }

    fn to_lines(&self) -> Lines {
        let mut lines = Vec::new();
        let mut active = self.start;

        for subpath in &self.subpaths {
            match *subpath {
                Subpath::Move(point) => active = point,
                Subpath::Line(point) => lines.push(Line(mem::replace(&mut active, point), point)),
            }
        }

        Lines(lines)
    }
}

#[derive(Debug)]
enum Subpath {
    Move(Point),
    Line(Point),
}

#[derive(Copy, Clone, Debug)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}

#[derive(Debug)]
struct Line(Point, Point);

#[derive(Debug)]
struct Lines(Vec<Line>);

impl Lines {
    fn bounds(&self) -> Rect {
        fn constrain(bounds: &mut Rect, point: Point) {
            bounds.tl.x = bounds.tl.x.min(point.x);
            bounds.tl.y = bounds.tl.y.min(point.y);
            bounds.br.x = bounds.br.x.max(point.x);
            bounds.br.y = bounds.br.y.max(point.y);
        }

        let mut bounds = Rect {
            tl: Point { x: f32::MAX, y: f32::MAX },
            br: Point { x: f32::MIN, y: f32::MIN },
        };

        for line in &self.0 {
            constrain(&mut bounds, line.0);
            constrain(&mut bounds, line.1);
        }

        bounds
    }

    fn x_intersections(&self, y: f32) -> Vec<f32> {
        let mut xs = Vec::new();

        for line in &self.0 {
            if (line.0.y < y && line.1.y >= y) || (line.1.y < y && line.0.y >= y) {
                xs.push(line.0.x + (y - line.0.y) / (line.1.y - line.0.y) * (line.1.x - line.0.x));
            }
        }

        xs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal));
        xs
    }
}

struct Rect {
    tl: Point,
    br: Point,
}

fn u2f(v: u8) -> f32 {
    (v as f32) / 255.0
}

fn f2u(v: f32) -> u8 {
    (v * 255.0).round() as u8
}
