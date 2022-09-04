use std::{fs::File, io::Write};

use icfpc2022::{read_isl_with_comments, write_isl, write_isl_with_comments, BlockId, Move};
use itertools::chain;

fn main() -> anyhow::Result<()> {
    let n = 400;
    let (mut program_lite, comments1) =
        read_isl_with_comments(File::open("submissions/15033.isl")?)?;
    let (mut program_full, comments2) =
        read_isl_with_comments(File::open("submissions/26658.isl")?)?;
    assert!(program_full[..n - 1]
        .iter()
        .all(|m| matches!(m, Move::Merge(..))));
    program_full.truncate(n - 1);

    let n = (2 * n - 2) as u32;
    for m in program_lite.iter_mut() {
        match m {
            Move::LineCut(BlockId(b), _, _)
            | Move::PointCut(BlockId(b), _, _)
            | Move::Color(BlockId(b), _) => b[0] += n,
            Move::Swap(BlockId(b), BlockId(bb)) | Move::Merge(BlockId(b), BlockId(bb)) => {
                b[0] += n;
                bb[0] += n
            }
        }
    }
    program_full.extend(program_lite);
    let mut w = File::create("out/tos27.isl")?;
    w.write_fmt(format_args!("# tos_2to27"))?;
    write_isl_with_comments(w, program_full, chain!(comments1, comments2))?;
    Ok(())
}
