use crate::persistence::persistent_entity_id::{
    PersistentAsteroidId, PersistentGateId, PersistentShipId, PersistentStationId,
};
use crate::utils::{AsteroidEntity, GateEntity, SectorEntity, ShipEntity, StationEntity};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Res, Resource};
use bevy::utils::HashMap;
use hexx::Hex;
use std::hash::Hash;

#[derive(SystemParam)]
pub struct AllEntityIdMaps<'w> {
    asteroids: Res<'w, AsteroidIdMap>,
    gates: Res<'w, GateIdMap>,
    sectors: Res<'w, SectorIdMap>,
    ships: Res<'w, ShipIdMap>,
    stations: Res<'w, StationIdMap>,
}

pub type AsteroidIdMap = EntityIdMap<PersistentAsteroidId, AsteroidEntity>;
pub type GateIdMap = EntityIdMap<PersistentGateId, GateEntity>;
pub type ShipIdMap = EntityIdMap<PersistentShipId, ShipEntity>;
pub type StationIdMap = EntityIdMap<PersistentStationId, StationEntity>;
pub type SectorIdMap = EntityIdMap<Hex, SectorEntity>;

/// A simple Bidirectional Map.
///
/// For anything more complex than numeric id values, use the bimap crate.
#[derive(Resource)]
pub struct EntityIdMap<TId, TEntity>
where
    TId: Eq + Hash + Copy,
    TEntity: Eq + Hash + Copy,
{
    id_to_entity: HashMap<TId, TEntity>,
    entity_to_id: HashMap<TEntity, TId>,
}

impl<TId, TEntity> EntityIdMap<TId, TEntity>
where
    TId: Eq + Hash + Copy,
    TEntity: Eq + Hash + Copy,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            id_to_entity: HashMap::new(),
            entity_to_id: HashMap::new(),
        }
    }

    /// Read-Only access to the underlying id_to_entity HashMap.
    #[inline]
    pub fn entity_to_id(&self) -> &HashMap<TEntity, TId> {
        &self.entity_to_id
    }

    /// Read-Only access to the underlying entity_to_id HashMap.
    #[inline]
    pub fn id_to_entity(&self) -> &HashMap<TId, TEntity> {
        &self.id_to_entity
    }

    #[inline]
    pub fn insert(&mut self, id: TId, entity: TEntity) {
        self.id_to_entity.insert(id, entity);
        self.entity_to_id.insert(entity, id);
    }

    #[inline]
    pub fn get_entity(&self, id: &TId) -> Option<&TEntity> {
        self.id_to_entity.get(id)
    }

    #[inline]
    pub fn get_id(&self, entity: &TEntity) -> Option<&TId> {
        self.entity_to_id.get(entity)
    }
}
