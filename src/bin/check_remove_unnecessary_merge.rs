use icfpc2022::*;

fn main() -> anyhow::Result<()> {
    let program = read_isl(std::fs::File::open(
        "/home/takiba/Dropbox/ICFPC2022/wata/out11.txt",
    )?)?;

    let program2 = local_optimization::remove_unnecessary_operations(&program);
    dbg!(program.len(), program2.len());
    local_optimization::remove_unnecessary_merge(&program2);

    Ok(())
}
