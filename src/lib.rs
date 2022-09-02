use std::{
    fmt,
    fs::File,
    io::{self, BufRead},
    panic,
    str::FromStr,
};

pub mod canvas;
pub mod color;
pub mod rotate;
pub mod wata;

pub use canvas::*;

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

pub fn read_png(path: &str) -> Vec<Vec<[u8; 4]>> {
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

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Point(pub i32, pub i32);

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

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Default)]
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
        let s = remove_spaces_inside_brackets(s);
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
    let r = io::BufReader::new(r);
    let mut program = Program::new();
    for line in r.lines() {
        program.push(line?.parse().unwrap());
    }
    Ok(program)
}

pub fn write_isl<W: io::Write>(mut w: W, program: Program) -> io::Result<()> {
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

pub fn similarity(a: &[[Color; 400]; 400], b: &[[Color; 400]; 400]) -> f64 {
    let pixel_pairs = a.iter().zip(b).flat_map(|(a, b)| a.iter().zip(b));
    pixel_pairs.map(|(a, b)| pixel_distance(a, b)).sum()
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
}
