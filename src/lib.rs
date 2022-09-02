use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufRead},
    panic,
    str::FromStr,
};

pub mod color;
pub mod rotate;

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
    pub fn cut(&self) -> [BlockId; 2] {
        [
            BlockId(self.0.iter().cloned().chain([0]).collect()),
            BlockId(self.0.iter().cloned().chain([1]).collect()),
        ]
    }
    pub fn cut4(&self) -> [BlockId; 4] {
        [
            BlockId(self.0.iter().cloned().chain([0]).collect()),
            BlockId(self.0.iter().cloned().chain([1]).collect()),
            BlockId(self.0.iter().cloned().chain([2]).collect()),
            BlockId(self.0.iter().cloned().chain([3]).collect()),
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
    pub fn area(&self) -> i32 {
        (self.1 .0 - self.0 .0) * (self.1 .1 - self.0 .1)
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

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    pub bitmap: [[Color; 400]; 400],
    pub blocks: HashMap<BlockId, Block>,
    pub counter: usize,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            bitmap: [[[0u8; 4]; 400]; 400],
            blocks: Default::default(),
            counter: Default::default(),
        }
    }
}

impl Canvas {
    // returns cost
    pub fn apply(&mut self, mov: &Move) -> f64 {
        let block_area = match mov {
            Move::LineCut(b, o, x) => {
                let block = self.blocks.remove(&b).unwrap();
                // NOTE: offset is absolute coordinate
                let [bid0, bid1] = b.cut();
                let block0;
                let block1;
                match o {
                    'x' | 'X' => {
                        block0 = Block(block.0, Point(*x, block.1 .1));
                        block1 = Block(Point(*x, block.0 .1), block.1);
                    }
                    'y' | 'Y' => {
                        block0 = Block(block.0, Point(block.1 .0, *x));
                        block1 = Block(Point(block.0 .0, *x), block.1);
                    }
                    _ => panic!("bad orientation: {}", o),
                }
                assert!(self.blocks.insert(bid0, block0).is_none());
                assert!(self.blocks.insert(bid1, block1).is_none());
                block.area()
            }
            Move::PointCut(b, x, y) => {
                let block = self.blocks.remove(&b).unwrap();
                // NOTE: offset is absolute coordinate
                let bids = b.cut4();
                let blocks = [
                    Block(block.0, Point(*x, *y)),
                    Block(Point(*x, block.0 .1), Point(block.1 .0, *y)),
                    Block(Point(*x, *y), block.1),
                    Block(Point(block.0 .0, *y), Point(*x, block.1 .1)),
                ];
                for (bid, block) in bids.into_iter().zip(blocks) {
                    assert!(self.blocks.insert(bid, block).is_none());
                }
                block.area()
            }
            Move::Color(b, c) => {
                let block = &self.blocks[&b];
                for y in block.1 .1..block.0 .1 {
                    for x in block.1 .0..block.0 .0 {
                        self.bitmap[y as usize][x as usize] = *c;
                    }
                }
                block.area()
            }
            Move::Swap(b1, b2) => todo!(),
            Move::Merge(b1, b2) => todo!(),
        };
        (mov.base_cost() * (400.0 * 400.0) / block_area as f64).round()
    }

    pub fn apply_all<Iter: Iterator<Item = Move>>(&mut self, iter: Iter) -> f64 {
        let mut cost = 0.0;
        for mov in iter {
            cost += self.apply(&mov);
        }
        cost
    }
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
