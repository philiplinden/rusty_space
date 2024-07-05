use crate::components::{Asteroid, Inventory};
use crate::game_data::DEBUG_ITEM_ID_ORE;
use crate::ship_ai::task_finished_event::TaskFinishedEvent;
use crate::ship_ai::task_queue::TaskQueue;
use crate::ship_ai::tasks;
use crate::ship_ai::tasks::send_completion_events;
use crate::utils::{CurrentSimulationTimestamp, Milliseconds, SimulationTime, SimulationTimestamp};
use bevy::log::error;
use bevy::prelude::{
    Commands, Component, Entity, EulerRot, EventReader, EventWriter, Query, Res, Time, Transform,
    With,
};
use std::cmp::min;
use std::sync::{Arc, Mutex};

pub const TIME_BETWEEN_MINING_UPDATES: Milliseconds = 1000;
pub const MINED_AMOUNT_PER_UPDATE: u32 = 10;

enum TaskResult {
    Skip,
    Ongoing { mined_amount: u32 },
    Finished { mined_amount: u32 },
}

#[derive(Component)]
pub struct MineAsteroid {
    pub target: Entity,
    pub next_update: SimulationTimestamp,
}

impl MineAsteroid {
    fn run(
        &mut self,
        inventory: &mut Inventory,
        asteroids: &Query<&mut Asteroid>,
        now: CurrentSimulationTimestamp,
    ) -> TaskResult {
        if now.has_not_passed(self.next_update) {
            return TaskResult::Skip;
        }

        let asteroid = asteroids.get(self.target).unwrap();
        let mined_amount = MINED_AMOUNT_PER_UPDATE
            .min(inventory.capacity - inventory.used())
            .min(asteroid.ore);

        inventory.add_item(DEBUG_ITEM_ID_ORE, mined_amount);

        // TODO: Test if we stripped this asteroid empty
        if inventory.used() == inventory.capacity {
            TaskResult::Finished { mined_amount }
        } else {
            self.next_update
                .add_milliseconds(TIME_BETWEEN_MINING_UPDATES);
            TaskResult::Ongoing { mined_amount }
        }
    }

    pub fn run_tasks(
        event_writer: EventWriter<TaskFinishedEvent<Self>>,
        simulation_time: Res<SimulationTime>,
        mut ships: Query<(Entity, &mut Self, &mut Inventory)>,
        mut all_asteroids: Query<&mut Asteroid>,
    ) {
        let task_completions = Arc::new(Mutex::new(Vec::<TaskFinishedEvent<Self>>::new()));
        let mined_asteroids = Arc::new(Mutex::new(Vec::<(Entity, u32)>::new()));
        let now = simulation_time.now();

        ships
            .par_iter_mut()
            .for_each(|(entity, mut task, mut inventory)| {
                match task.run(&mut inventory, &all_asteroids, now) {
                    TaskResult::Skip => {}
                    TaskResult::Ongoing { mined_amount } => {
                        mined_asteroids
                            .lock()
                            .unwrap()
                            .push((task.target, mined_amount));
                    }
                    TaskResult::Finished { mined_amount } => {
                        mined_asteroids
                            .lock()
                            .unwrap()
                            .push((task.target, mined_amount));
                        task_completions
                            .lock()
                            .unwrap()
                            .push(TaskFinishedEvent::<Self>::new(entity))
                    }
                }
            });

        match Arc::try_unwrap(mined_asteroids) {
            Ok(mined_asteroids) => {
                let batch = mined_asteroids.into_inner().unwrap();
                if !batch.is_empty() {
                    for (entity, mined_amount) in batch {
                        all_asteroids.get_mut(entity).unwrap().ore -= mined_amount;
                        // TODO: if empty, trigger despawn event!
                    }
                }
            }
            Err(_) => {
                todo!()
            }
        }

        send_completion_events(event_writer, task_completions);
    }

    pub fn complete_tasks(
        mut commands: Commands,
        mut event_reader: EventReader<TaskFinishedEvent<Self>>,
        mut all_ships_with_task: Query<&mut TaskQueue, With<Self>>,
    ) {
        for event in event_reader.read() {
            if let Ok(mut queue) = all_ships_with_task.get_mut(event.entity) {
                tasks::remove_task_and_add_new_one::<Self>(&mut commands, event.entity, &mut queue);
            } else {
                error!(
                    "Unable to find entity for task completion: {}",
                    event.entity
                );
            }
        }
    }
}