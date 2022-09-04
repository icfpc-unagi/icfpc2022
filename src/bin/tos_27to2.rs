use std::{fs::File, io::Write};

use icfpc2022::{read_isl_with_comments, write_isl_with_comments, BlockId, Move};

fn main() -> anyhow::Result<()> {
    let n = 400;
    let (mut program, comments) = read_isl_with_comments(File::open("submissions/26658.isl")?)?;
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
    let mut w = File::create("out/tos2.isl")?;
    w.write_fmt(format_args!("# tos_27to2"))?;
    write_isl_with_comments(w, program, &comments)?;
    Ok(())
}
