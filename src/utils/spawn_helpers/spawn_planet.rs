use crate::components::{ConstantOrbit, Planet, Sector, SelectableEntity};
use crate::persistence::{PersistentPlanetId, PlanetIdMap};
use crate::simulation::ship_ai::AutoTradeBehavior;
use crate::simulation::transform::simulation_transform::SimulationTransform;
use crate::utils::{PlanetEntity, SectorEntity};
use crate::{constants, SpriteHandles};
use bevy::core::Name;
use bevy::math::Vec2;
use bevy::prelude::{default, Commands, Query, Rot2, SpriteBundle};

#[allow(clippy::too_many_arguments)]
pub fn spawn_planet(
    commands: &mut Commands,
    planet_id_map: &mut PlanetIdMap,
    sprites: &SpriteHandles,
    name: String,
    sectors: &mut Query<&mut Sector>,
    sector_entity: SectorEntity,
    orbit_radius: f32,
    orbit_rotational_fraction: f32,
    mass: u32,
) {
    let mut sector_data = sectors.get_mut(sector_entity.into()).unwrap();

    let planet_id = PersistentPlanetId::next();
    let planet = Planet::new(planet_id, mass);

    // TODO: calculate that from distance and angle
    let local_position = Vec2::ZERO;

    // TODO: Grab that from the sector/star. And figure out what unit we wanna use.
    let star_mass = 100.0;

    // TODO: Instead of using a realistic gravitational constant, we can just adjust this value for our simulation until it "feels" right, that's why this value is bogus
    const GRAVITATIONAL_CONSTANT: f32 = 0.00067;

    let velocity = ((GRAVITATIONAL_CONSTANT * star_mass) / orbit_radius).sqrt();

    let simulation_transform =
        SimulationTransform::new(sector_data.world_pos + local_position, Rot2::IDENTITY, 1.0);

    let entity = commands
        .spawn((
            Name::new(name),
            SelectableEntity::Planet,
            AutoTradeBehavior::default(),
            ConstantOrbit::new(orbit_rotational_fraction, orbit_radius, velocity),
            SpriteBundle {
                texture: sprites.planet.clone(),
                transform: simulation_transform.as_transform(constants::PLANET_AND_STARS_LAYER),
                ..default()
            },
            simulation_transform,
            planet,
        ))
        .id();

    let planet_entity = PlanetEntity::from(entity);
    planet_id_map.insert(planet_id, planet_entity);
    sector_data.add_planet(commands, sector_entity, planet_entity);
}