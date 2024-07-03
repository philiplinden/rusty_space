use crate::sectors::{GatePair, InSector};
use bevy::core::Name;
use bevy::prelude::{Commands, Component, Entity, SpatialBundle, Transform, Vec2, Vec3};
use bevy::utils::{HashMap, HashSet};
use hexx::{Hex, HexLayout};

pub struct SectorEntity(Entity);

/// Marker Component for Sectors
#[derive(Component)]
pub struct Sector {
    pub coordinate: Hex,
    pub world_pos: Vec2,

    /// HashMap<Sector, (FromGate, ToGate)>. TODO: This is terrible
    pub gates: HashMap<Entity, GatePair>,
    ships: HashSet<Entity>,
    stations: HashSet<Entity>,
}

impl Sector {
    pub fn new(coordinate: Hex, world_pos: Vec2) -> Self {
        Sector {
            coordinate,
            world_pos,
            gates: HashMap::new(),
            ships: HashSet::new(),
            stations: HashSet::new(),
        }
    }

    /// Adds ship to this sector and inserts the [InSector] component to it.
    pub fn add_ship(&mut self, commands: &mut Commands, sector_entity: Entity, entity: Entity) {
        self.ships.insert(entity);
        self.in_sector(commands, sector_entity, entity);
    }

    /// Removes ship from this sector whilst also deleting its [InSector] component.
    pub fn remove_ship(&mut self, commands: &mut Commands, entity: Entity) {
        let result = self.ships.remove(&entity);
        debug_assert!(result, "removed ships should always be in sector!");
        commands.entity(entity).remove::<InSector>();
    }

    /// Adds the station to this sector and inserts the [InSector] component to it.
    pub fn add_station(&mut self, commands: &mut Commands, sector_entity: Entity, entity: Entity) {
        self.stations.insert(entity);
        self.in_sector(commands, sector_entity, entity);
    }

    /// Adds the gate to this sector and inserts the [InSector] component to it.
    pub fn add_gate(
        &mut self,
        commands: &mut Commands,
        this_sector: Entity,
        this_gate: Entity,
        destination_sector: Entity,
        destination_gate: Entity,
    ) {
        self.gates
            .insert(destination_sector, (this_gate, destination_gate));
        self.in_sector(commands, this_sector, this_gate);
    }

    /// Adds the [InSector] component linking to `self` to the provided Entity.
    fn in_sector(&self, commands: &mut Commands, sector_entity: Entity, entity: Entity) {
        commands.entity(entity).insert(InSector {
            sector: sector_entity,
        });
    }
}

pub fn spawn_sector(commands: &mut Commands, layout: &HexLayout, coordinate: Hex) -> Entity {
    let position = layout.hex_to_world_pos(coordinate);
    // TODO: remove this once hexx is updated to same glam crate as bevy 0.14
    let position = Vec2::new(position.x, position.y);

    commands
        .spawn((
            Name::new(format!("[{},{}]", coordinate.x, coordinate.y)),
            Sector::new(coordinate, position),
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id()
}
