use std::fs::File;

use crate::{Color, Point};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InitialJson {
    pub width: usize,
    pub height: usize,
    pub blocks: Vec<Block>,
}

impl InitialJson {
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Self {
        let file = File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub block_id: String, // like "0", not usize
    pub bottom_left: Point,
    pub top_right: Point,
    pub color: Color,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_27() {
        let id = 27;
        // let ini: InitialJson =
        //     serde_json::from_reader(File::open(format!("problems/{}.initial.json", id)).unwrap())
        //         .unwrap();
        let ini = InitialJson::from_path(format!("problems/{}.initial.json", id));
        // dbg!(ini);
        assert_eq!(ini.width, 400);
        let b3 = &ini.blocks[3];
        assert_eq!(b3.block_id, "3");
        assert_eq!(b3.bottom_left, Point(0, 60));
        assert_eq!(b3.top_right, Point(20, 80));
        assert_eq!(b3.color, [56, 182, 255, 255]);
    }
}
