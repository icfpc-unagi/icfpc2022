use std::fs::File;

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

#[derive(Debug)]
struct BlockId(Vec<u32>);

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
enum Move {
    LineCut(BlockId, char, u32), // orientation, offset (x or y)
    PointCut(BlockId, u32, u32), // offset (x and y)
    Color(BlockId, u32),         // TODO: color type?
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

impl ToString for Move {
    fn to_string(&self) -> String {
        match self {
            Move::LineCut(block, ori, offset) => format!("cut [{}] [{}] [{}]", block, ori, offset),
            Move::PointCut(block, x, y) => format!("cut [{}] [{},{}]", block, x, y),
            Move::Color(block, color) => format!("color [{}] [{}]", block, color), // TODO
            Move::Swap(block1, block2) => format!("swap [{}] [{}]", block1, block2),
            Move::Merge(block1, block2) => format!("merge [{}] [{}]", block1, block2),
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
        // TODO: Color
        // assert_eq!(
        //     Move::Color(BlockId(vec![1]), 2).to_string(),
        //     "color [1] [2,3,4,5]"
        // );
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
