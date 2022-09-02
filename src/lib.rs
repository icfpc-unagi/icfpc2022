use std::{collections::HashMap, fmt, fs::File};

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
                out[i][j][k] = buf[(i * w + j) * 4 + k];
            }
        }
    }
    out
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockId(Vec<u32>);

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

#[derive(Debug)]
struct Block {
    // TODO
}

#[derive(Debug)]
struct Canvas {
    blocks: HashMap<BlockId, Block>,
    bitmap: [[Color; 400]; 400],
}

type Color = [u8; 4];

#[derive(Debug)]
pub enum Move {
    LineCut(BlockId, char, u32), // orientation, offset (x or y)
    PointCut(BlockId, u32, u32), // offset (x and y)
    Color(BlockId, Color),
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

// Instruction Set
pub type Program = Vec<Move>;

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
