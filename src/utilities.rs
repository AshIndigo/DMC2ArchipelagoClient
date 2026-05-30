use randomizer_utilities::get_base_address;
use std::sync::LazyLock;

pub static DMC2_ADDRESS: LazyLock<usize> = LazyLock::new(|| get_base_address("dmc2.exe"));

/// Checks to see if DDMK is loaded
pub fn is_ddmk_loaded() -> bool {
    randomizer_utilities::is_library_loaded("Lucia.dll")
}

pub(crate) fn is_on_main_menu() -> bool {
    true
}
