use crate::*;
use std::{collections::HashMap, panic};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    pub bitmap: [[Color; 400]; 400],
    pub blocks: HashMap<BlockId, Block>,
    pub counter: u32,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            bitmap: [[[0u8; 4]; 400]; 400],
            blocks: HashMap::from([(BlockId(vec![0]), Block(Point(0, 0), Point(400, 400)))]),
            counter: 0,
        }
    }
}

impl Canvas {
    pub fn new() -> Self {
        Canvas::default()
    }
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
            Move::Swap(b0, b1) => {
                let block0 = &self.blocks[&b0];
                let block1 = &self.blocks[&b1];
                let size = block0.size();
                assert_eq!(size, block1.size());
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
                // TODO: validate compatibility
                self.counter += 1;
                let bid = BlockId(vec![self.counter]);
                let block = Block(block0.0.min(block1.0), block0.1.max(block1.1));
                assert!(self.blocks.insert(bid, block).is_none());
                // cost the larger area; not the union of both
                block0.area().max(block1.area())
            }
        };
        (mov.base_cost() * (400.0 * 400.0) / block_area as f64).round()
    }

    pub fn apply_all<I: IntoIterator<Item = Move>>(&mut self, iter: I) -> f64 {
        let mut cost = 0.0;
        for mov in iter {
            cost += self.apply(&mov);
        }
        cost
    }
}
