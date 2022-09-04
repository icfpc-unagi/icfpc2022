use std::{
    fmt,
    fs::File,
    io::{self, BufRead},
    iter::empty,
    ops::{Add, Sub},
    panic,
    str::FromStr,
};

pub mod canvas;
pub mod chokudai1;
pub mod chokudai_dev2;
pub mod color;
pub mod initial_json;
pub mod local_optimization;
pub mod optmerge;
pub mod rotate;
pub mod submissions;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
pub mod wata;

pub use canvas::*;
pub use initial_json::InitialJson;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

#[macro_export]
macro_rules! mat {
    ($($e:expr),*) => { vec![$($e),*] };
    ($($e:expr,)*) => { vec![$($e),*] };
    ($e:expr; $d:expr) => { vec![$e; $d] };
    ($e:expr; $d:expr $(; $ds:expr)+) => { vec![mat![$e $(; $ds)*]; $d] };
}

pub fn read_png<P: AsRef<std::path::Path>>(path: P) -> Vec<Vec<[u8; 4]>> {
    let decoder = png::Decoder::new(File::open(path).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let h = info.height as usize;
    let w = info.width as usize;
    let mut out = mat![[0; 4]; h; w];
    for i in 0..h {
        for j in 0..w {
            for k in 0..4 {
                out[h - i - 1][j][k] = buf[(i * w + j) * 4 + k];
            }
        }
    }
    out
}

pub fn write_png<P: AsRef<std::path::Path>>(
    path: P,
    bitmap: Vec<Vec<Color>>,
) -> Result<(), png::EncodingError> {
    let mut encoder = png::Encoder::new(
        io::BufWriter::new(File::create(path)?),
        bitmap[0].len() as u32,
        bitmap.len() as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    let data = flatten_png_data(&bitmap);
    writer.write_image_data(&data)
}

fn flatten_png_data(bitmap: &Vec<Vec<Color>>) -> Vec<u8> {
    Vec::from_iter(bitmap.iter().rev().flatten().flatten().cloned())
}

pub fn write_apng_from_program<P>(
    path: P,
    canvas: &mut Canvas,
    program: Program,
    seconds_per_loop: u16,
) -> Result<(), png::EncodingError>
where
    P: AsRef<std::path::Path>,
{
    let n_frames = 1 + program.iter().filter(|m| m.may_change_bitmap()).count(); // program.len() + 1;

    let mut encoder = png::Encoder::new(
        io::BufWriter::new(File::create(path)?),
        canvas.bitmap[0].len() as u32,
        canvas.bitmap.len() as u32,
    );
    encoder.set_animated(n_frames as u32, 0)?;
    encoder.set_frame_delay(seconds_per_loop, n_frames as u16)?;
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    {
        let data = flatten_png_data(&canvas.bitmap);
        writer.write_image_data(&data)?;
    }

    for m in program.iter() {
        canvas.apply(m);
        if m.may_change_bitmap() {
            let data = flatten_png_data(&canvas.bitmap);
            writer.write_image_data(&data)?;
        }
    }
    Ok(())
}

#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    Hash,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Point(pub i32, pub i32);

impl Add<Point> for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<Point> for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct BlockId(pub Vec<u32>);

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut it = self.0.iter();
        f.write_fmt(format_args!("{}", it.next().unwrap()))?;
        for x in it {
            f.write_fmt(format_args!(".{}", x))?;
        }
        Ok(())
    }
}

impl FromIterator<u32> for BlockId {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl BlockId {
    pub fn extended<I: IntoIterator<Item = u32>>(&self, intoiter: I) -> BlockId {
        BlockId(self.0.iter().cloned().chain(intoiter).collect())
    }
    pub fn cut(&self) -> [BlockId; 2] {
        [self.extended([0]), self.extended([1])]
    }
    pub fn cut4(&self) -> [BlockId; 4] {
        [
            self.extended([0]),
            self.extended([1]),
            self.extended([2]),
            self.extended([3]),
        ]
    }
}

impl FromStr for BlockId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BlockId(s.split(".").map(|x| x.parse().unwrap()).collect()))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Block(pub Point, pub Point);

impl Block {
    pub fn size(&self) -> Point {
        Point(self.1 .0 - self.0 .0, self.1 .1 - self.0 .1)
    }
    pub fn area(&self) -> i32 {
        let size = self.size();
        size.0 * size.1
    }
}

pub type Color = [u8; 4];
const WHITE: Color = [255; 4];

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Move {
    LineCut(BlockId, char, i32), // orientation, offset (x or y)
    PointCut(BlockId, i32, i32), // offset (x and y)
    Color(BlockId, Color),
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

impl Move {
    pub fn base_cost(&self) -> f64 {
        match self {
            Move::LineCut(_, _, _) => 7.0,
            Move::PointCut(_, _, _) => 10.0,
            Move::Color(_, _) => 5.0,
            Move::Swap(_, _) => 3.0,
            Move::Merge(_, _) => 1.0,
        }
    }

    pub fn may_change_bitmap(&self) -> bool {
        matches!(self, Move::Color(_, _) | Move::Swap(_, _))
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::LineCut(block, ori, offset) => {
                f.write_fmt(format_args!("cut [{}] [{}] [{}]", block, ori, offset))
            }
            Move::PointCut(block, x, y) => {
                f.write_fmt(format_args!("cut [{}] [{},{}]", block, x, y))
            }
            Move::Color(block, c) => f.write_fmt(format_args!(
                "color [{}] [{},{},{},{}]",
                block, c[0], c[1], c[2], c[3]
            )),
            Move::Swap(block1, block2) => {
                f.write_fmt(format_args!("swap [{}] [{}]", block1, block2))
            }
            Move::Merge(block1, block2) => {
                f.write_fmt(format_args!("merge [{}] [{}]", block1, block2))
            }
        }
    }
}

fn remove_spaces_inside_brackets(s: &str) -> String {
    let mut chars = vec![];
    let mut lev: i32 = 0;
    for c in s.chars() {
        if c == '[' {
            lev += 1;
        }
        if c == ']' {
            lev -= 1;
        }
        if c.is_whitespace() && lev > 0 {
            continue;
        }
        chars.push(c);
    }
    return chars.into_iter().collect();
}

fn unwrap_brackets(s: &str) -> &str {
    assert!(s.len() >= 2);
    assert_eq!(s.chars().nth(0).unwrap(), '[');
    assert_eq!(s.chars().last().unwrap(), ']');
    //assert_eq!(s[0 as usize], '[');
    //assert_eq!(s[s.len() - 1], ']');
    &s[1..s.len() - 1]
}

fn parse_numbers<T: FromStr>(s: &str) -> Vec<T>
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    s.split(',').map(|t| t.parse().unwrap()).collect()
}

fn parse_color(s: &str) -> Color {
    let v = parse_numbers(s);
    assert_eq!(v.len(), 4);
    return [v[0], v[1], v[2], v[3]];
}

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = remove_spaces_inside_brackets(s.trim());
        let tokens: Vec<_> = s.split(" ").collect();
        assert!(tokens.len() >= 1);

        let op = tokens[0];
        let args: Vec<_> = tokens[1..].iter().map(|s| unwrap_brackets(s)).collect();

        let mv;
        match &*op {
            "cut" => {
                if args.len() == 2 {
                    let p = parse_numbers::<i32>(args[1]);
                    assert_eq!(p.len(), 2);
                    mv = Move::PointCut(args[0].parse().unwrap(), p[0], p[1]);
                } else if args.len() == 3 {
                    assert_eq!(args[1].len(), 1);
                    mv = Move::LineCut(
                        args[0].parse().unwrap(),
                        args[1].chars().nth(0).unwrap(),
                        args[2].parse().unwrap(),
                    );
                } else {
                    panic!();
                }
            }
            "color" => mv = Move::Color(args[0].parse().unwrap(), parse_color(args[1])),
            "merge" => mv = Move::Merge(args[0].parse().unwrap(), args[1].parse().unwrap()),
            "swap" => mv = Move::Swap(args[0].parse().unwrap(), args[1].parse().unwrap()),
            _ => {
                panic!("Unknown instruction: {:?}", &tokens)
            }
        }
        return Ok(mv);
    }
}

// Instruction Set
pub type Program = Vec<Move>;

pub fn read_isl<R: io::Read>(r: R) -> io::Result<Program> {
    Ok(read_isl_with_comments(r)?.0)
}

pub fn read_isl_with_comments<R: io::Read>(r: R) -> io::Result<(Program, Vec<String>)> {
    let r = io::BufReader::new(r);
    let mut program = Program::new();
    let mut comments = Vec::new();
    for line in r.lines() {
        let line = line?;
        let line = line.trim_start();
        if line.starts_with('#') {
            comments.push(line[1..].trim().into());
            continue;
        }
        if line.is_empty() {
            continue;
        }
        program.push(line.parse().unwrap());
    }
    Ok((program, comments))
}

pub fn write_isl<W: io::Write>(w: W, program: Program) -> io::Result<()> {
    write_isl_with_comments(w, program, std::iter::empty::<&str>())
}

pub fn write_isl_with_comments<W: io::Write, I: IntoIterator>(
    mut w: W,
    program: Program,
    comments: I,
) -> io::Result<()>
where
    I::Item: fmt::Display,
{
    for comment in comments {
        w.write_fmt(format_args!("# {}\n", comment))?
    }
    for mov in program {
        w.write_fmt(format_args!("{}\n", mov))?
    }
    Ok(())
}

pub fn pixel_distance(a: &Color, b: &Color) -> f64 {
    (a.into_iter()
        .zip(b)
        .map(|(&a, &b)| a as i32 - b as i32)
        .map(|x| x * x)
        .sum::<i32>() as f64)
        .sqrt()
}

pub fn similarity(a: &Vec<Vec<Color>>, b: &Vec<Vec<Color>>) -> f64 {
    let pixel_pairs = a.iter().zip(b).flat_map(|(a, b)| a.iter().zip(b));
    (pixel_pairs.map(|(a, b)| pixel_distance(a, b)).sum::<f64>() * 0.005).round()
}

// individual color difference
pub fn color_diff_bitmap(a: &Vec<Vec<Color>>, b: &Vec<Vec<Color>>) -> Vec<Vec<Color>> {
    a.iter()
        .zip(b)
        .map(|(a, b)| {
            a.iter()
                .zip(b)
                .map(|(a, b)| {
                    a.iter()
                        .zip(b)
                        .map(|(a, b)| a.abs_diff(*b))
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                })
                .collect()
        })
        .collect()
}

// grayscale pixel_distance rounded, clamped
pub fn pixel_distance_bitmap(a: &Vec<Vec<Color>>, b: &Vec<Vec<Color>>) -> Vec<Vec<Color>> {
    a.iter()
        .zip(b)
        .map(|(a, b)| {
            a.iter()
                .zip(b)
                .map(|(a, b)| {
                    let d = pixel_distance(a, b).round().clamp(0.0, 255.0) as u8;
                    [d, d, d, 255]
                })
                .collect()
        })
        .collect()
}

pub fn load_problem(problem_id: u32) -> (Canvas, Vec<Vec<Color>>) {
    let target_png = read_png(format!("problems/{problem_id}.png"));
    let canvas = if problem_id <= 25 {
        Canvas::new(target_png[0].len(), target_png.len())
    } else {
        Canvas::from_initial_json(format!("problems/{problem_id}.initial.json"))
    };
    (canvas, target_png)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Submission {
    pub id: u32,
    pub problem_id: u32,
    pub status: String,
    pub cost: u32,
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn move_to_string() {
        assert_eq!(
            Move::LineCut(BlockId(vec![1]), 'x', 2).to_string(),
            "cut [1] [x] [2]"
        );
        assert_eq!(
            Move::PointCut(BlockId(vec![1]), 2, 3).to_string(),
            "cut [1] [2,3]"
        );
        assert_eq!(
            Move::Color(BlockId(vec![1]), [2, 3, 4, 5]).to_string(),
            "color [1] [2,3,4,5]"
        );
        assert_eq!(
            Move::Swap(BlockId(vec![1]), BlockId(vec![2])).to_string(),
            "swap [1] [2]"
        );
        assert_eq!(
            Move::Merge(BlockId(vec![1]), BlockId(vec![2])).to_string(),
            "merge [1] [2]"
        );
    }

    #[test]
    fn block_id() {
        assert_eq!("1.2.3".parse(), Ok(BlockId(vec![1, 2, 3])));
        assert_eq!(BlockId(vec![1]).extended([2]), BlockId(vec![1, 2]));
        assert_eq!(
            BlockId(vec![1]).cut(),
            [BlockId(vec![1, 0]), BlockId(vec![1, 1])]
        );
    }

    #[test]
    fn test_load_problem() {
        let (canvas, _) = load_problem(25);
        assert_eq!(canvas.blocks.len(), 1);

        let (canvas, _) = load_problem(26);
        assert_eq!(canvas.blocks.len(), 100);
    }
}
