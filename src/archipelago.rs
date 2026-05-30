use std::collections::VecDeque;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::OnceLock;
use std::time::Duration;
use archipelago_rs::{Client, Connection, ConnectionOptions, ConnectionState, DeathLinkOptions, Event, ItemHandling};
use randomizer_utilities::archipelago_utilities::{handle_print, DeathLinkData};
use randomizer_utilities::{item_sync, setup_channel_pair};
use randomizer_utilities::ui::font_handler::WHITE;
use randomizer_utilities::ui::overlay_messages;
use randomizer_utilities::ui::overlay_messages::{MessageSegment, MessageType, OverlayMessage};
use crate::{game_manager, hook};
use crate::check_handler::{Location, TX_LOCATION};
use crate::game_manager::{ArchipelagoData, ARCHIPELAGO_DATA};
use crate::mapping::{DeathlinkSetting, Mapping, OverlayInfo, MAPPING, OVERLAY_INFO};

pub(crate) static CONNECTED: AtomicBool = AtomicBool::new(false);
pub static TX_DEATHLINK: OnceLock<Sender<DeathLinkData>> = OnceLock::new();

pub struct ArchipelagoCore {
    pub connection: Connection<Mapping>,
    received_items_queue: VecDeque<usize>,
    hooks_installed: bool,
    hooks_enabled: bool,

    location_receiver: Receiver<Location>,
    deathlink_receiver: Receiver<DeathLinkData>,
}

impl ArchipelagoCore {
    pub fn new(url: String, game_name: String) -> anyhow::Result<Self> {
        Ok(Self {
            connection: Connection::new(
                url,
                "",
                Some(game_name),
                ConnectionOptions::new().receive_items(ItemHandling::OtherWorlds {
                    own_world: true,
                    starting_inventory: true,
                }),
            ),
            received_items_queue: VecDeque::new(),
            hooks_installed: false,
            hooks_enabled: false,
            location_receiver: setup_channel_pair(&TX_LOCATION),
            deathlink_receiver: setup_channel_pair(&TX_DEATHLINK),
        })
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        for event in self.connection.update() {
            match event {
                Event::Connected => {
                    log::info!("Connected!");
                    log::debug!("Mod version: {}", env!("CARGO_PKG_VERSION"));
                    let mapping = self.connection.client().unwrap().slot_data();
                    let mut overlay_info = OVERLAY_INFO.write()?;
                    log::info!("Running in randomizer mode");
                    overlay_info.generated_version = mapping.generated_version;
                    overlay_info.client_version = mapping.client_version;
                    MAPPING.write()?.replace(mapping.clone());
                    self.received_items_queue.clear();
                    item_sync::send_offline_checks(self.connection.client_mut().unwrap())?;
                    if !self.hooks_installed {
                        // Hooks needed to modify the game
                        unsafe {
                            match hook::create_hooks() {
                                Ok(_) => {
                                    log::debug!("Created DMC2 Hooks");
                                    self.hooks_installed = true;
                                }
                                Err(err) => {
                                    log::error!("Failed to create hooks: {:?}", err);
                                }
                            }
                        }
                    }
                    if self.hooks_installed && !self.hooks_enabled {
                        hook::enable_hooks();
                        self.hooks_enabled = true;
                    }
                    run_setup(self.connection.client_mut().unwrap())?;

                    // Print out version info
                    log::debug!(
                        "Client version: {}",
                        if let Some(cv) = overlay_info.client_version {
                            cv.to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    );

                    log::debug!(
                        "Generated version: {}",
                        if let Some(gv) = overlay_info.generated_version {
                            gv.to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    );
                }
                Event::Updated(_) => {}
                Event::Print(print) => {
                    let str = handle_print(print);
                    log::info!("Print from server: {}", str);
                }
                Event::ReceivedItems(idx) => {
                    self.received_items_queue.push_back(idx);
                }
                Event::Error(err) => log::error!("{}", err),
                Event::Bounce {
                    games: _,
                    slots: _,
                    tags: _,
                    data: _,
                } => {}
                Event::DeathLink { cause, source, .. } => {
                    overlay_messages::add_message(OverlayMessage::new(
                        vec![MessageSegment::new(
                            format!("{}: {}", source, cause.unwrap_or_default()),
                            WHITE,
                        )],
                        Duration::from_secs(3),
                        // TODO May want to adjust position, currently added to the 'notification list' so it's in the upper right queue
                        0.0,
                        0.0,
                        MessageType::Notification,
                    ));

                    match self.connection.client().unwrap().slot_data().death_link {
                        DeathlinkSetting::DeathLink => {
                            game_manager::kill_dante();
                        }
                        DeathlinkSetting::HurtLink => {
                            game_manager::hurt_dante();
                        }
                        DeathlinkSetting::Off => {}
                    }
                }
                Event::KeyChanged {
                    key: _,
                    old_value: _,
                    new_value: _,
                    player: _,
                } => {}
            }
        }
        match self.connection.state() {
            ConnectionState::Connecting(_) => {}
            ConnectionState::Connected(_) => {
                CONNECTED.store(true, Ordering::SeqCst);
            }
            ConnectionState::Disconnected(state) => {
                CONNECTED.store(false, Ordering::SeqCst);
                *OVERLAY_INFO.write()? = OverlayInfo::default();
                disconnect(&mut self.hooks_enabled);
                return Err(format!("Disconnected from server: {:?}", state).into());
            }
        }
        self.handle_channels()?;
        // let _ = with_session_read(|s| { // TODO Session handler
        //     if s.event.contains(constants::Event::Ingame)
        //         && let Some(idx) = self.received_items_queue.pop_front()
        //         && let Err(e) =
        //         handle_received_items_packet(idx, self.connection.client_mut().unwrap())
        //     {
        //         log::error!("Failed to receive items: {:?}", e);
        //     }
        // });
        Ok(())
    }

    pub fn handle_channels(&mut self) -> Result<(), Box<dyn Error>> {
        match self.location_receiver.try_recv() {
            Ok(location) => {
                if let Some(client) = self.connection.client_mut() {
                    handle_item_receive(client, location)?;
                } else {
                    log::error!(
                        "Received location check while client was None: {}",
                        location
                    );
                }
            }
            Err(err) => {
                if err == TryRecvError::Disconnected {
                    return Err("Disconnected from location receiver".into());
                }
            }
        }

        match self.deathlink_receiver.try_recv() {
            Ok(dl_data) => self
                .connection
                .client_mut()
                .unwrap()
                .death_link(DeathLinkOptions::new().cause(dl_data.cause))?,
            Err(err) => {
                if err == TryRecvError::Disconnected {
                    return Err("Disconnected from DeathLink receiver".into());
                }
            }
        }
        Ok(())
    }
}

fn handle_received_items_packet(index: usize, client: &mut Client<Mapping>) -> Result<(), Box<dyn Error>> {
    todo!()
}

fn handle_item_receive(client: &mut Client<Mapping>, idx: Location) -> Result<(), Box<dyn Error>>  {
    todo!()
}

fn run_setup(client: &mut Client<Mapping>) -> Result<(), Box<dyn Error>> {
    todo!()
}

fn disconnect(hooks_enabled: &mut bool) {
    log::info!("Disconnecting and restoring game");
    if *hooks_enabled {
        match hook::disable_hooks() {
            Ok(_) => {
                log::debug!("Disabled hooks");
                *hooks_enabled = false;
            }
            Err(e) => {
                log::error!("Failed to disable hooks: {:?}", e);
            }
        }
    }
    
    MAPPING.write().unwrap().take(); // Clear mappings
    *ARCHIPELAGO_DATA.write().unwrap() = ArchipelagoData::default(); // Reset Data (Probably not needed)
    log::info!("Game restored to default state");
}