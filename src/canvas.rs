use crate::*;
use std::{collections::HashMap, panic};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    pub bitmap: Vec<Vec<Color>>,
    pub blocks: HashMap<BlockId, Block>,
    pub counter: u32,
}

fn check_valid_block(b: &Block) -> anyhow::Result<()> {
    if b.area() == 0 {
        anyhow::bail!("New block: area zero: {:?}", &b);
    }
    if b.0 .0 >= b.1 .0 {
        anyhow::bail!("New block: x0 > x1: {:?}", &b);
    }
    if b.0 .1 >= b.1 .1 {
        anyhow::bail!("New block: y0 > y1: {:?}", &b);
    }
    anyhow::Ok(())
}

pub fn check_merge_compatibility(b0: &Block, b1: &Block) -> anyhow::Result<()> {
    if b0.0 .0 == b1.0 .0 {
        // x座標一致、y座標方向のマージ

        if b0.1 .0 != b1.1 .0 {
            anyhow::bail!(
                "Merge compatibility: x0 matches, but x1 differs: {:?} {:?}",
                b0,
                b1
            );
        }

        if !(b0.1 .1 == b1.0 .1 || b0.0 .1 == b1.1 .1) {
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
        if !(b0.1 .0 == b1.0 .0 || b0.0 .0 == b1.1 .0) {
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

impl TryFrom<InitialJson> for Canvas {
    type Error = anyhow::Error;

    fn try_from(ini: InitialJson) -> anyhow::Result<Self> {
        let w = ini.width;
        let h = ini.height;
        let mut bitmap = vec![vec![WHITE; w]; h];
        let mut blocks = HashMap::new();
        for (i, block) in ini.blocks.iter().enumerate() {
            let id = block.block_id.parse::<BlockId>().unwrap(); // TODO: use `?`
            if id != BlockId(vec![i as u32]) {
                anyhow::bail!("id is not sorted. please fix me and fix `counter`")
            }
            let rect = Block(block.bottom_left, block.top_right);
            blocks.insert(id, rect);
            if let Some(color) = block.color {
                for y in rect.0 .1..rect.1 .1 {
                    for x in rect.0 .0..rect.1 .0 {
                        bitmap[y as usize][x as usize] = color;
                    }
                }
            } else {
                todo!()
            }
        }
        // let blocks = .map(|(i, block)| {
        //     if id.to_string() != block.block_id {
        //         anyhow::bail!("id is not sorted. please fix me and fix `counter`")
        //     }
        //     // TODO: edit bitmap
        //     (BlockId(vec![id]), )
        // }).collect::<HashMap<_, _>>();
        Ok(Self {
            bitmap,
            blocks,
            counter: ini.blocks.len() as u32 - 1,
        })
    }
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            bitmap: vec![vec![WHITE; w]; h],
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
    pub fn from_initial_json<P: AsRef<std::path::Path>>(path: P) -> Self {
        crate::InitialJson::from_path(path).try_into().unwrap()
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
                check_valid_block(&block0)?;
                check_valid_block(&block1)?;
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
                let block0 = self.blocks[&b0];
                let block1 = self.blocks.insert(b1.clone(), block0).unwrap();
                self.blocks.insert(b0.clone(), block1).unwrap();
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
                if !self.blocks.contains_key(&b0) {
                    anyhow::bail!("block {} does not exist", *b0,)
                }
                if !self.blocks.contains_key(&b1) {
                    anyhow::bail!("block {} does not exist", *b1,)
                }
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

    pub fn apply_all_and_score<I: IntoIterator<Item = Move>>(
        &mut self,
        iter: I,
        answer: &Vec<Vec<Color>>,
    ) -> anyhow::Result<f64> {
        let cost = self.apply_all_safe(iter)?;
        let sim = similarity(&answer, &self.bitmap);
        anyhow::Ok(cost + sim)
    }
}

// initial canvas
impl From<Vec<Vec<Color>>> for Canvas {
    fn from(bitmap: Vec<Vec<Color>>) -> Self {
        let w = bitmap[0].len() as i32;
        let h = bitmap.len() as i32;
        Self {
            bitmap,
            blocks: HashMap::from([(BlockId(vec![0]), Block(Point(0, 0), Point(w, h)))]),
            counter: Default::default(),
        }
    }
}

pub fn score(
    program: &Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
) -> anyhow::Result<f64> {
    initial_canvas
        .clone()
        .apply_all_and_score(program.clone(), image)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json;

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
            assert_eq!(cost as u32 + sim as u32, sub.cost);
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

    #[test]
    fn test_initial_27() {
        let canvas = Canvas::from_initial_json("problems/27.initial.json");
        let initial_png = read_png("problems/27.initial.png");
        assert_eq!(&canvas.bitmap, &initial_png);
    }
}
