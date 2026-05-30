pub const GAME_NAME: &str = "Devil May Cry 2";

pub const EMPTY_COORDINATES: Coordinates = Coordinates { x: 0, y: 0, z: 0 };

#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) z: u32,
}

impl Coordinates {
    pub fn has_coords(&self) -> bool {
        self.x > 0
    }
}

impl PartialEq for Coordinates {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}
