use crate::game_data::{ItemId, ProductionModuleId, RecipeId, ShipyardModuleId};
use crate::persistence::data::v1::inventory_save_data::InventorySaveData;
use crate::persistence::PersistentStationId;
use crate::session_data::ShipConfigId;
use crate::universe_builder::LocalHexPosition;
use crate::utils::{PriceSetting, SimulationTimestamp};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ProductionSaveData {
    pub modules: Vec<ProductionModuleSaveData>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct ProductionModuleSaveData {
    pub module_id: ProductionModuleId,
    pub amount: u32,
    pub recipe: RecipeId,
    pub finished_at: Option<SimulationTimestamp>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShipyardModuleSaveData {
    pub module_id: ShipyardModuleId,
    pub amount: u32,
    pub active: Vec<ActiveShipyardOrderSaveData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveShipyardOrderSaveData {
    pub finished_at: SimulationTimestamp,
    pub ship_config: ShipConfigId,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShipyardSaveData {
    pub queue: Vec<ShipConfigId>,
    pub modules: Vec<ShipyardModuleSaveData>,
}

#[derive(Serialize, Deserialize)]
pub struct StationSaveData {
    pub id: PersistentStationId,
    pub name: String,
    pub position: LocalHexPosition,
    pub inventory: InventorySaveData,
    pub production_modules: Option<ProductionSaveData>,
    pub shipyard_modules: Option<ShipyardSaveData>,
    pub buy_orders: Option<SerializedBuyOrder>,
    pub sell_orders: Option<SerializedSellOrder>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializedBuyOrder {
    pub orders: Vec<SerializedBuyOrderData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializedSellOrder {
    pub orders: Vec<SerializedSellOrderData>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct SerializedBuyOrderData {
    pub item_id: ItemId,
    pub amount: u32,
    pub price: u32,

    pub buy_up_to: u32,
    pub price_setting: PriceSetting,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct SerializedSellOrderData {
    pub item_id: ItemId,
    pub amount: u32,
    pub price: u32,

    pub keep_at_least: u32,
    pub price_setting: PriceSetting,
}
