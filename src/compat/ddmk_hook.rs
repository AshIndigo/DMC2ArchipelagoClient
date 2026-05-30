use crate::archipelago::CONNECTED;
use crate::{config, game_manager};
use imgui_sys::{ImGuiCond, ImGuiCond_Appearing, ImGuiWindowFlags, ImVec2};
use randomizer_utilities::dmc::common_ddmk;
use randomizer_utilities::dmc::common_ddmk::{SETUP, get_orig_render_func, run_common_ddmk_code};
use randomizer_utilities::dmc::dmc_helpers::DDMKHandler;
use randomizer_utilities::{get_base_address, read_data_from_address};
use std::os::raw::c_char;
use std::sync::atomic::Ordering;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::thread;

pub static LUCIA_ADDRESS: LazyLock<usize> = LazyLock::new(|| get_base_address("Lucia.dll"));
const DDMK_UI_ENABLED: usize = 0x7119A;

#[derive(Debug, Default)]
struct CustomDataHolder {
    category: String,
    id: String,
    count: String,
    hp_to_give: String,
}

impl CustomDataHolder {
    // pub fn convert_to_data(&self) -> ItemData {
    //     ItemData {
    //         category: self.category.parse().unwrap_or(0),
    //         id: self.id.parse().unwrap_or(0),
    //         count: self.count.parse().unwrap_or(0),
    //     }
    // }
}

static CUSTOM_ITEM: Mutex<CustomDataHolder> = Mutex::new(CustomDataHolder {
    category: String::new(),
    id: String::new(),
    count: String::new(),
    hp_to_give: String::new(),
});
unsafe extern "C" fn hooked_render() {
    unsafe {
        if !SETUP.load(Ordering::SeqCst) {
            return;
        }

        if !read_data_from_address::<bool>(DDMK_UI_ENABLED + *LUCIA_ADDRESS) {
            return;
        }

        archipelago_window(CUSTOM_ITEM.lock().unwrap()); // For the archipelago window
        tracking_window();
        match get_orig_render_func() {
            None => {}
            Some(fnc) => {
                fnc();
            }
        }
    }
}

unsafe fn tracking_window() {
    unsafe {
        let flag = &mut true;
        common_ddmk::get_imgui_next_pos()(
            &ImVec2 { x: 800.0, y: 320.0 }, // 300
            ImGuiCond_Appearing as ImGuiCond,
            &ImVec2 { x: 0.0, y: 0.0 },
        );
        common_ddmk::get_imgui_begin()(
            c"Tracker".as_ptr() as *const c_char,
            flag as *mut bool,
            imgui_sys::ImGuiWindowFlags_AlwaysAutoResize as ImGuiWindowFlags,
        );

        match game_manager::ARCHIPELAGO_DATA.read() {
            Ok(data) => {
                // for chunk in constants::get_items_by_category(ItemCategory::Key).chunks(3) {
                //     let row_text = chunk
                //         .iter()
                //         .map(|&item| checkbox_text(&item.to_string(), &data.items))
                //         .collect::<Vec<String>>()
                //         .join("  ");
                //     common_ddmk::text(format!("{}\0", row_text));
                // }
                common_ddmk::text(format!("Blue Orbs: {}\0", data.blue_orbs));
                common_ddmk::text(format!("Purple Orbs: {}\0", data.purple_orbs));
                // HP Trackers
                // with_session_read(|s| {
                //     common_ddmk::text(format!("S HP: {}\0", s.hp));
                // })
                // .unwrap();
                // with_active_player_data_read(|s| {
                //     common_ddmk::text(format!("D Max HP: {}\0", s.max_hp));
                //     common_ddmk::text(format!("D HP: {}\0", s.hp));
                // })
                // .unwrap();
            }
            Err(err) => {
                log::error!("Failed to read ArchipelagoData: {:?}", err);
            }
        }

        common_ddmk::get_imgui_end()();
    }
}

unsafe fn archipelago_window(mut custom_item_data: MutexGuard<CustomDataHolder>) {
    unsafe {
        let flag = &mut true;
        common_ddmk::get_imgui_next_pos()(
            &ImVec2 { x: 800.0, y: 100.0 },
            ImGuiCond_Appearing as ImGuiCond,
            &ImVec2 { x: 0.0, y: 0.0 },
        );
        common_ddmk::get_imgui_begin()(
            c"Archipelago".as_ptr() as *const c_char,
            flag as *mut bool,
            imgui_sys::ImGuiWindowFlags_AlwaysAutoResize as ImGuiWindowFlags,
        );
        common_ddmk::text(format!(
            "Status: {}\0",
            if CONNECTED.load(Ordering::SeqCst) {
                "Connected"
            } else {
                "Disconnected"
            }
        ));
        //const DEBUG: bool = true;
        //if DEBUG {
        if common_ddmk::get_imgui_button()(
            c"Call function".as_ptr() as *const c_char,
            &ImVec2 { x: 0.0, y: 0.0 },
        ) {
            thread::spawn(move || {
                log::debug!("Calling function");
            });
        }
        //}
        common_ddmk::get_imgui_end()();
    }
}

pub fn setup_ddmk_hook() {
    if !config::CONFIG.mods.disable_ddmk_hooks {
        log::info!("Starting up DDMK hook");
        log::info!("Lucia base ADDR: {:X}", *LUCIA_ADDRESS);
        if common_ddmk::DDMK_INFO
            .set(DDMKHandler {
                ddmk_address: LazyLock::new(|| get_base_address("Lucia.dll")),
                main_func_addr: 0x530C0,
                timestep_func_addr: 0x9440,
                ddmk_ui_enabled: DDMK_UI_ENABLED,
                hooked_render: hooked_render as *const () as _,
                text_addr: 0x487C0,
                end_addr: 0x104B0,
                begin_addr: 0xAE20,
                button_addr: 0x46A0,
                next_pos: 0x20340,
            })
            .is_err()
        {
            log::error!("Failed to set DDMK info");
        }
        run_common_ddmk_code();
        log::info!("DDMK hook initialized");
    } else {
        log::info!("DDMK is detected but hooks will not be enabled")
    }
}
