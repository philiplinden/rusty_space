use crate::components::{BuyOrders, Inventory, SellOrders};
use crate::game_data::{GameData, ShipyardModuleId};
use crate::production::production_kind::ProductionKind;
use crate::production::production_started_event::ProductionStartedEvent;
use crate::production::shipyard_component::{OngoingShipConstructionOrder, ShipyardComponent};
use crate::production::ProductionComponent;
use crate::session_data::{SessionData, ShipConfigId};
use crate::utils;
use crate::utils::SimulationTime;
use bevy::log::error;
use bevy::prelude::{Entity, Event, EventReader, EventWriter, Or, Query, Res, With};

/// This event should be sent whenever an entity's inventory is being updated outside the production manager
///
/// More performant than querying with Changed<Inventory> since bevy won't need to iterate
/// through all entities matching the query every frame, plus it won't trigger itself recursively
/// ...the only risk is that we may forget to send it on inventory changes. What could go wrong?
#[derive(Event)]
pub struct InventoryUpdateForProductionEvent {
    entity: Entity,
}

impl InventoryUpdateForProductionEvent {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_inventory_updates(
    simulation_time: Res<SimulationTime>,
    game_data: Res<GameData>,
    session_data: Res<SessionData>,
    mut event_reader: EventReader<InventoryUpdateForProductionEvent>,
    mut production_start_event_writer: EventWriter<ProductionStartedEvent>,
    mut query: Query<
        (
            Option<&mut ProductionComponent>,
            Option<&mut ShipyardComponent>,
            &mut Inventory,
            Option<&mut BuyOrders>,
            Option<&mut SellOrders>,
        ),
        Or<(With<ProductionComponent>, With<ShipyardComponent>)>,
    >,
) {
    let now = simulation_time.now();
    for event in event_reader.read() {
        let Ok((production, shipyard, mut inventory, buy_orders, sell_orders)) =
            query.get_mut(event.entity)
        else {
            continue;
        };

        // Check Item Production Lines
        if let Some(mut production) = production {
            for (id, module) in production.modules.iter_mut() {
                if module.current_run_finished_at.is_some() {
                    continue;
                }

                let recipe = game_data.item_recipes.get(&module.recipe).unwrap();
                if inventory.has_enough_items_in_inventory(&recipe.input, module.amount)
                    && inventory.has_enough_storage_for_items(&recipe.output, module.amount)
                {
                    inventory.remove_items(&recipe.input, module.amount);
                    inventory.reserve_storage_space_for_production_yield(recipe, module.amount);

                    let finish_timestamp = now.add_milliseconds(recipe.duration);
                    module.current_run_finished_at = Some(finish_timestamp);

                    production_start_event_writer.send(ProductionStartedEvent::new(
                        event.entity,
                        ProductionKind::Item(*id),
                        finish_timestamp,
                    ));
                }
            }
        }

        // Check Shipyard production lines
        if let Some(mut shipyard) = shipyard {
            if shipyard.queue.is_empty() {
                continue;
            }

            let mut available_module_ids: Vec<ShipyardModuleId> = shipyard
                .modules
                .iter()
                .filter_map(|(id, module)| {
                    if module.active.len() < module.amount as usize {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();

            if available_module_ids.is_empty() {
                continue;
            }

            let mut affordable_ships_from_queue: Vec<(usize, ShipConfigId)> = shipyard
                .queue
                .iter()
                .enumerate()
                .filter_map(|(index, config_id)| {
                    let Some(configuration) = session_data.ship_configurations.get(config_id)
                    else {
                        error!("Was unable to find a configuration with id {config_id}");
                        return None;
                    };

                    if inventory.has_enough_items_in_inventory(&configuration.materials, 1) {
                        Some((index, *config_id))
                    } else {
                        None
                    }
                })
                .collect();

            if affordable_ships_from_queue.is_empty() {
                continue;
            }

            while !affordable_ships_from_queue.is_empty() && !available_module_ids.is_empty() {
                let (next_index, next_ship_config_id) =
                    *affordable_ships_from_queue.first().unwrap();
                let module_id = *available_module_ids.first().unwrap();
                let module = shipyard.modules.get_mut(&module_id).unwrap();

                let Some(configuration) =
                    session_data.ship_configurations.get(&next_ship_config_id)
                else {
                    error!("Was unable to find a configuration with id {next_ship_config_id}?! This should seriously never happen at this point.");
                    continue;
                };

                let finish_timestamp = now.add_milliseconds(configuration.duration);
                module.active.push(OngoingShipConstructionOrder {
                    ship_config: next_ship_config_id,
                    finished_at: finish_timestamp,
                });

                if module.active.len() >= module.amount as usize {
                    available_module_ids.retain(|x| x != &module_id)
                }

                inventory.remove_items(&configuration.materials, 1);
                affordable_ships_from_queue.retain(|(index, config)| {
                    if index == &next_index {
                        return false;
                    }

                    let config = &session_data.ship_configurations[config];
                    inventory.has_enough_items_in_inventory(&config.materials, 1)
                });
                shipyard.queue.remove(next_index);

                production_start_event_writer.send(ProductionStartedEvent::new(
                    event.entity,
                    ProductionKind::Shipyard(module_id),
                    finish_timestamp,
                ));
            }
        }

        utils::update_orders(&inventory, buy_orders, sell_orders);
    }
}
