pub fn flip_png(mut png: Vec<Vec<[u8; 4]>>) -> Vec<Vec<[u8; 4]>> {
    // 左右反転
    for row in png.iter_mut() {
        row.reverse();
    }
    png
}

pub fn flip_program(program: &Vec<super::Move>) -> Vec<super::Move> {
    unimplemented!()
}
