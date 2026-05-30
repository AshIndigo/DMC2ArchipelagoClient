use std::collections::HashSet;
use std::sync::{LazyLock, RwLock};
use crate::mapping::MAPPING;

#[derive(Debug, Default)]
pub(crate) struct ArchipelagoData {
    pub(crate) blue_orbs: i32,
    pub(crate) purple_orbs: i32,
    pub(crate) dt_unlocked: bool,
    gun_levels: [u32; 5],
    pub(crate) items: HashSet<String>,
}

pub static ARCHIPELAGO_DATA: LazyLock<RwLock<ArchipelagoData>> =
    LazyLock::new(|| RwLock::new(ArchipelagoData::default()));

impl ArchipelagoData {
    pub fn add_item(&mut self, item: String) {
        self.items.insert(item);
    }

    pub(crate) fn add_blue_orb(&mut self) {
        self.blue_orbs = (self.blue_orbs + 1).min(14);
    }

    pub(crate) fn add_purple_orb(&mut self) {
        self.purple_orbs = (self.purple_orbs + 1).min(10);
        if let Some(mappings) = MAPPING.read().unwrap().as_ref()
            && !mappings.devil_trigger_mode
        {
            self.dt_unlocked = true;
        }
    }

    pub(crate) fn add_dt(&mut self) {
        if let Some(mappings) = MAPPING.read().unwrap().as_ref() {
            if mappings.devil_trigger_mode {
                self.dt_unlocked = true;
            }
            if !mappings.purple_orb_mode {
                self.purple_orbs = (self.purple_orbs + 3).min(10);
            }
        }
    }

    pub(crate) fn add_gun_level(&mut self, gun_index: usize) {
        self.gun_levels[gun_index] = (self.gun_levels[gun_index] + 1).min(2);
    }

    pub(crate) fn reset_gun_levels(&mut self) {
        self.gun_levels = [0; 5];
    }
}

pub(crate) fn kill_dante() {
    todo!()
}

pub(crate) fn hurt_dante() {
    todo!()
}