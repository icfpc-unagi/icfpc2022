use std::fs::File;

use icfpc2022::{read_isl, write_isl, BlockId, Move};

fn main() -> anyhow::Result<()> {
    let n = 400;
    let mut program = read_isl(File::open("submissions/26658.isl")?)?;
    dbg!(program.len());
    let ok = program.drain(..n - 1).all(|m| matches!(m, Move::Merge(..)));
    assert!(ok);
    dbg!(program.len());

    let n = (2 * n - 2) as u32;
    for m in program.iter_mut() {
        match m {
            Move::LineCut(BlockId(b), _, _)
            | Move::PointCut(BlockId(b), _, _)
            | Move::Color(BlockId(b), _) => b[0] -= n,
            Move::Swap(BlockId(b), BlockId(bb)) | Move::Merge(BlockId(b), BlockId(bb)) => {
                b[0] -= n;
                bb[0] -= n
            }
        }
    }
    write_isl(File::create("out/tos2.isl")?, program)?;
    Ok(())
}
