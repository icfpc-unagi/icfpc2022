use crate::*;
use std::{collections::HashMap, panic};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    pub bitmap: Vec<Vec<Color>>,
    pub blocks: HashMap<BlockId, Block>,
    pub counter: u32,
}

fn check_merge_compatibility(b0: &Block, b1: &Block) -> anyhow::Result<()> {
    if b0.0 .0 > b1.0 .0 || b0.0 .1 > b1.0 .1 {
        return check_merge_compatibility(b1, b0);
    }
    // b0のほうがb1より左(x座標小)か、下(y座標小)にある

    if b0.0 .0 == b1.0 .0 {
        // x座標一致、y座標方向のマージ
        if b0.1 .0 != b1.1 .0 {
            anyhow::bail!(
                "Merge compatibility: x0 matches, but x1 differs: {:?} {:?}",
                b0,
                b1
            );
        }
        if b0.1 .1 != b1.0 .1 {
            anyhow::bail!(
                "Merge compatibility: x0 and x1 match, but y1 and y0 differ: {:?} {:?}",
                b0,
                b1
            );
        }
    } else if b0.0 .1 == b1.0 .1 {
        // y座標一致、x座標方向のマージ
        if b0.1 .1 != b1.1 .1 {
            anyhow::bail!(
                "Merge compatibility: y0 matches, but y1 differs: {:?} {:?}",
                b0,
                b1
            );
        }
        if b0.1 .0 != b1.0 .0 {
            anyhow::bail!(
                "Merge compatibility: y0 and y1 match, but x1 and x0 differ: {:?} {:?}",
                b0,
                b1
            );
        }
    } else {
        anyhow::bail!(
            "Merge compatibility: neither of x nor y coordinate matches: {:?} {:?}",
            b0,
            b1
        );
    }

    anyhow::Ok(())
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            bitmap: vec![vec![Color::default(); w]; h],
            blocks: HashMap::from([(
                BlockId(vec![0]),
                Block(Point(0, 0), Point(w as i32, h as i32)),
            )]),
            counter: Default::default(),
        }
    }
    pub fn new400() -> Self {
        Self::new(400, 400)
    }

    // returns cost
    pub fn apply(&mut self, mov: &Move) -> f64 {
        self.apply_safe(mov).unwrap()
    }

    // returns cost
    pub fn apply_safe(&mut self, mov: &Move) -> anyhow::Result<f64> {
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
                if block0.area() == 0 {
                    anyhow::bail!("Area of block {} is zero", bid0);
                }
                if block1.area() == 0 {
                    anyhow::bail!("Area of block {} is zero", bid1);
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
                for y in block.0 .1..block.1 .1 {
                    for x in block.0 .0..block.1 .0 {
                        self.bitmap[y as usize][x as usize] = *c;
                    }
                }
                block.area()
            }
            Move::Swap(b0, b1) => {
                let block0 = &self.blocks[&b0];
                let block1 = &self.blocks[&b1];
                let size = block0.size();

                // Check!
                //assert_eq!(size, block1.size());
                let size1 = block1.size();
                if size != size1 {
                    anyhow::bail!(
                        "Swaped blocks have different sizes: {:?} vs {:?} ({}, {})",
                        size,
                        size1,
                        *b0,
                        *b1,
                    )
                }

                for y in 0..size.1 {
                    for x in 0..size.0 {
                        let y0 = (block0.0 .1 + y) as usize;
                        let x0 = (block0.0 .0 + x) as usize;
                        let y1 = (block1.0 .1 + y) as usize;
                        let x1 = (block1.0 .0 + x) as usize;
                        let tmp = self.bitmap[y0][x0];
                        self.bitmap[y0][x0] = self.bitmap[y1][x1];
                        self.bitmap[y1][x1] = tmp;
                    }
                }
                block0.area()
            }
            Move::Merge(b0, b1) => {
                let block0 = self.blocks.remove(&b0).unwrap();
                let block1 = self.blocks.remove(&b1).unwrap();

                check_merge_compatibility(&block0, &block1)?;

                self.counter += 1;
                let bid = BlockId(vec![self.counter]);
                let block = Block(block0.0.min(block1.0), block0.1.max(block1.1));
                assert!(self.blocks.insert(bid, block).is_none());
                // cost the larger area; not the union of both
                block0.area().max(block1.area())
            }
        };

        anyhow::Ok(
            (mov.base_cost() * (self.bitmap.len() * self.bitmap[0].len()) as f64
                / block_area as f64)
                .round(),
        )
    }

    pub fn apply_all<I: IntoIterator<Item = Move>>(&mut self, iter: I) -> f64 {
        self.apply_all_safe(iter).unwrap()
    }

    pub fn apply_all_safe<I: IntoIterator<Item = Move>>(&mut self, iter: I) -> anyhow::Result<f64> {
        let mut cost = 0.0;
        for mov in iter {
            cost += self.apply_safe(&mov)?;
        }
        anyhow::Ok(cost)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde::*;
    use serde_json;

    #[derive(Serialize, Deserialize)]
    struct Submission {
        id: u32,
        problem_id: u32,
        status: String,
        score: u32,
    }

    #[test]
    fn test_1677() {
        for id in [270, 1677, 2796] {
            let sub: Submission =
                serde_json::from_reader(File::open(format!("submissions/{}.json", id)).unwrap())
                    .unwrap();
            assert_eq!(sub.status, "SUCCEEDED");
            let isl = read_isl(File::open(format!("submissions/{}.isl", id)).unwrap()).unwrap();
            let png = read_png(&format!("problems/{}.png", sub.problem_id));
            let mut canvas = Canvas::new400();
            let cost = canvas.apply_all(isl);
            let sim = similarity(&png, &canvas.bitmap);
            assert_eq!(cost as u32 + sim as u32, sub.score);
            // write_png(&format!("submissions/{}_target.png", id), png).unwrap();
            // write_png(&format!("submissions/{}_painted.png", id), canvas.bitmap).unwrap();
        }
    }

    #[test]
    fn test_error_line_cut() {
        let mut canvas = Canvas::new400();
        assert!(canvas
            .apply_safe(&Move::LineCut(BlockId(vec![0]), 'x', 0,))
            .is_err());
    }

    #[test]
    fn test_error_merge() {
        let mut canvas = Canvas::new400();
        assert!(canvas
            .apply_safe(&Move::LineCut(BlockId(vec![0]), 'x', 10,))
            .is_ok());
        assert!(canvas
            .apply_safe(&Move::LineCut(BlockId(vec![0, 0]), 'y', 20,))
            .is_ok());
        assert!(canvas
            .apply_safe(&Move::LineCut(BlockId(vec![0, 1]), 'y', 21,))
            .is_ok());
        assert!(canvas
            .apply_safe(&Move::Merge(BlockId(vec![0, 0, 0]), BlockId(vec![0, 1, 0])))
            .is_err());
    }
}
