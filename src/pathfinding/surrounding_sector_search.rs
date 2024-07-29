use crate::components::{Sector, SectorAsteroidComponent};
use crate::utils::SectorEntity;
use bevy::prelude::Query;
use bevy::utils::HashSet;
use std::cmp::Ordering;

#[derive(PartialEq, Eq)]
pub struct SearchResult {
    pub distance: u8,
    pub sector: SectorEntity,
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance
            .cmp(&other.distance)
            .then_with(|| self.sector.cmp(&other.sector))
    }
}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Performs a breadth-first search on the sectors surrounding `from`.
///
/// # Returns
/// A Vec containing the [`SearchResult`]s which match the provided `search_fn`, ordered by their distance to `from`.
pub fn surrounding_sector_search<TSearchFunction>(
    all_sectors: &Query<&Sector>,
    from: SectorEntity,
    max_range: u8,
    sector_search_query: &Query<&SectorAsteroidComponent>, // TODO: Find a way to replace this with a generic search query + function
    search_fn: TSearchFunction,
) -> Vec<SearchResult>
where
    TSearchFunction: Fn(&SectorAsteroidComponent) -> bool,
{
    let mut visited = HashSet::default();
    let mut next = vec![&from];
    let mut result = Vec::new();

    let mut current_depth = 0;
    while current_depth <= max_range {
        let mut next_next = Vec::new();
        for sector_entity in next.into_iter() {
            visited.insert(sector_entity);

            let sector = all_sectors.get(sector_entity.into()).unwrap();
            if let Ok(search_components) = sector_search_query.get(sector_entity.into()) {
                if search_fn(search_components) {
                    result.push(SearchResult {
                        distance: current_depth,
                        sector: *sector_entity,
                    });
                }
            }

            for (to_sector, _) in &sector.gates {
                if !visited.contains(to_sector) {
                    next_next.push(to_sector);
                }
            }
        }

        next = next_next;
        current_depth += 1;
    }

    result
}

#[cfg(test)]
mod test {
    use crate::components::{Sector, SectorAsteroidComponent};
    use crate::pathfinding::surrounding_sector_search::surrounding_sector_search;
    use crate::persistence::local_hex_position::LocalHexPosition;
    use crate::persistence::{SectorAsteroidSaveData, SectorIdMap, UniverseSaveData};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Query, Res, Vec2, World};
    use hexx::Hex;

    const LEFT: Hex = Hex::new(-1, 0);
    const CENTER: Hex = Hex::new(0, 0);
    const RIGHT: Hex = Hex::new(1, 0);

    fn has_asteroids(_asteroids: &SectorAsteroidComponent) -> bool {
        true
    }

    fn test_breadth_search(
        world: &mut World,
        from_sector: Hex,
        max_range: u8,
        expected_result: Vec<(u8, Hex)>,
    ) {
        world.run_system_once(
            move |sectors: Query<&Sector>,
                  sector_id_map: Res<SectorIdMap>,
                  search_query: Query<&SectorAsteroidComponent>| {
                let from_entity = sector_id_map.id_to_entity()[&from_sector];

                let result = surrounding_sector_search(
                    &sectors,
                    from_entity,
                    max_range,
                    &search_query,
                    has_asteroids,
                );

                let transformed_result: Vec<(u8, Hex)> = result
                    .iter()
                    .map(|x| (x.distance, sector_id_map.entity_to_id()[&x.sector]))
                    .collect();

                assert_eq!(expected_result, transformed_result);
            },
        );
    }

    #[test]
    fn find_neighbor_with_asteroids() {
        let mut universe = UniverseSaveData::default();
        universe.sectors.add(LEFT);
        universe.sectors.add(CENTER);
        universe
            .sectors
            .add(RIGHT)
            .with_asteroids(SectorAsteroidSaveData {
                // TODO: That probably also needs a builder
                average_velocity: Vec2::ONE,
                live_asteroids: Vec::new(),
                respawning_asteroids: Vec::new(),
            });
        universe.gate_pairs.add(
            LocalHexPosition::new(LEFT, Vec2::X),
            LocalHexPosition::new(CENTER, Vec2::NEG_X),
        );
        universe.gate_pairs.add(
            LocalHexPosition::new(CENTER, Vec2::X),
            LocalHexPosition::new(RIGHT, Vec2::NEG_X),
        );

        let mut app = universe.build_test_app();
        let world = app.world_mut();

        test_breadth_search(world, CENTER, 5, vec![(1, RIGHT)]);
    }

    #[test]
    fn find_multiple_neighbor_with_asteroids() {
        let mut universe = UniverseSaveData::default();
        universe.sectors.add(LEFT);
        universe
            .sectors
            .add(CENTER)
            .with_asteroids(SectorAsteroidSaveData {
                // TODO: That probably also needs a builder
                average_velocity: Vec2::ONE,
                live_asteroids: Vec::new(),
                respawning_asteroids: Vec::new(),
            });
        universe
            .sectors
            .add(RIGHT)
            .with_asteroids(SectorAsteroidSaveData {
                // TODO: That probably also needs a builder
                average_velocity: Vec2::ONE,
                live_asteroids: Vec::new(),
                respawning_asteroids: Vec::new(),
            });
        universe.gate_pairs.add(
            LocalHexPosition::new(LEFT, Vec2::X),
            LocalHexPosition::new(CENTER, Vec2::NEG_X),
        );
        universe.gate_pairs.add(
            LocalHexPosition::new(CENTER, Vec2::X),
            LocalHexPosition::new(RIGHT, Vec2::NEG_X),
        );

        let mut app = universe.build_test_app();
        let world = app.world_mut();

        test_breadth_search(world, LEFT, 2, vec![(1, CENTER), (2, RIGHT)]);
    }

    #[test]
    fn find_neighbor_with_asteroids_does_not_find_anything_outside_of_max_range() {
        let mut universe = UniverseSaveData::default();
        universe.sectors.add(LEFT);
        universe.sectors.add(CENTER);
        universe
            .sectors
            .add(RIGHT)
            .with_asteroids(SectorAsteroidSaveData {
                // TODO: That probably also needs a builder
                average_velocity: Vec2::ONE,
                live_asteroids: Vec::new(),
                respawning_asteroids: Vec::new(),
            });
        universe.gate_pairs.add(
            LocalHexPosition::new(LEFT, Vec2::X),
            LocalHexPosition::new(CENTER, Vec2::NEG_X),
        );
        universe.gate_pairs.add(
            LocalHexPosition::new(CENTER, Vec2::X),
            LocalHexPosition::new(RIGHT, Vec2::NEG_X),
        );

        let mut app = universe.build_test_app();
        let world = app.world_mut();

        test_breadth_search(world, LEFT, 1, vec![]);
    }
}
