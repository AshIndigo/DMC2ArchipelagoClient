use std::fmt::{Display, Formatter};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;
use crate::constants::Coordinates;

pub(crate) static TX_LOCATION: OnceLock<Sender<Location>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LocationType {
    Standard,
    MissionComplete,
    SSRank,
    PurchaseItem,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Location {
    pub(crate) location_type: LocationType,
    pub(crate) item_id: u32,
    pub(crate) room: i32,
    pub(crate) mission: u32,
    pub coordinates: Coordinates,
    pub to_display: bool,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mission: {:#} Room ID: {:#} Item ID: {:#x}",
            self.mission, self.room, self.item_id
        )
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.coordinates == other.coordinates
            && self.room == other.room
            && self.item_id == other.item_id
    }
}