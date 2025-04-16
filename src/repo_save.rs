//! Structs representing a REPO save file's JSON.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Representation of a Save file in Rust
/// Field naming to most closely match REPO save file field names.
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveGame {
    #[serde(rename = "dictionaryOfDictionaries")]
    pub dictionary_of_dictionaries: Dictionary,
    #[serde(rename = "playerNames")]
    pub player_names: PlayerNames,
    #[serde(rename = "timePlayed")]
    pub time_played: TimePlayedValue,
    #[serde(rename = "dateAndTime")]
    pub date_and_time: StringValue,
    #[serde(rename = "teamName")]
    pub team_name: StringValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dictionary {
    #[serde(rename = "__type")]
    pub _type: String,
    pub value: DictionaryValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DictionaryValue {
    #[serde(rename = "runStats")]
    pub run_stats: HashMap<String, i32>,
    #[serde(rename = "itemsPurchased")]
    pub items_purchased: HashMap<String, i32>,
    #[serde(rename = "itemsPurchasedTotal")]
    pub items_purchased_total: HashMap<String, i32>,
    #[serde(rename = "itemsUpgradesPurchased")]
    pub items_upgrades_purchased: HashMap<String, i32>,
    #[serde(rename = "itemBatteryUpgrades")]
    pub item_battery_upgrades: HashMap<String, i32>,
    #[serde(rename = "playerHealth")]
    pub player_health: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeHealth")]
    pub player_upgrade_health: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeStamina")]
    pub player_upgrade_stamina: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeExtraJump")]
    pub player_upgrade_extra_jump: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeLaunch")]
    pub player_upgrade_launch: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeMapPlayerCount")]
    pub player_upgrade_map_player_count: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeSpeed")]
    pub player_upgrade_speed: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeStrength")]
    pub player_upgrade_strength: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeRange")]
    pub player_upgrade_range: HashMap<String, i32>,
    #[serde(rename = "playerUpgradeThrow")]
    pub player_upgrade_throw: HashMap<String, i32>,
    #[serde(rename = "playerHasCrown")]
    pub player_has_crown: HashMap<String, i32>,
    pub item: HashMap<String, i32>,
    #[serde(rename = "itemStatBattery")]
    pub item_stat_battery: HashMap<String, i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerNames {
    #[serde(rename = "__type")]
    pub _type: String,
    pub value: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimePlayedValue {
    #[serde(rename = "__type")]
    pub _type: String,
    pub value: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StringValue {
    #[serde(rename = "__type")]
    pub _type: String,
    pub value: String,
}
