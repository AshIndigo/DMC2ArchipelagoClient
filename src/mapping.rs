use randomizer_utilities::APVersion;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::sync::{LazyLock, RwLock};

pub static MAPPING: LazyLock<RwLock<Option<Mapping>>> = LazyLock::new(|| RwLock::new(None));

pub static OVERLAY_INFO: LazyLock<RwLock<OverlayInfo>> =
    LazyLock::new(|| RwLock::new(OverlayInfo::default()));

#[derive(Debug, Default)]
pub struct OverlayInfo {
    pub client_version: Option<APVersion>,
    pub generated_version: Option<APVersion>,
}

/// Figure out which DL setting were on
fn parse_death_link<'de, D>(deserializer: D) -> Result<DeathlinkSetting, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Value::deserialize(deserializer)?;
    match val {
        Value::Number(n) => match n.as_i64().unwrap_or_default() {
            0 => Ok(DeathlinkSetting::Off),
            1 => Ok(DeathlinkSetting::DeathLink),
            2 => Ok(DeathlinkSetting::HurtLink),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid DL option: {}",
                n
            ))),
        },
        other => Err(serde::de::Error::custom(format!(
            "Unexpected type: {:?}",
            other
        ))),
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DeathlinkSetting {
    DeathLink, // Normal DeathLink Behavior
    HurtLink,  // Sends out DeathLink messages when you die. But only hurts you if you receive one
    Off,       // Don't send/receive DL related messages
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Mapping {
    pub generated_version: Option<APVersion>,
    pub client_version: Option<APVersion>,

    #[serde(deserialize_with = "parse_death_link")]
    pub death_link: DeathlinkSetting,

    pub purple_orb_mode: bool,
    pub devil_trigger_mode: bool,
}
