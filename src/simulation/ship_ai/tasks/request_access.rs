use crate::components::{GasGiant, InteractionQueue};
use crate::simulation::prelude::{SimulationTime, TaskInsideQueue, TaskQueue};
use crate::simulation::ship_ai::{tasks, TaskFinishedEvent};
use crate::utils::TypedEntity;
use bevy::prelude::{Commands, Component, Entity, EventWriter, Query, Res};

/// Intermediate task to reserve a spot inside an [`InteractionQueue`] attached to the [`target`].
///
/// Will always be immediately removed on execution, with two possible results depending on the queue's state:
///  - free: proceeding with the next task in this entity's local [`TaskQueue`]
///  - busy: spawning an [`AwaitingSignal`] Task
#[derive(Component)]
pub struct RequestAccess {
    target: TypedEntity,
}

impl RequestAccess {
    pub fn new(target: TypedEntity) -> Self {
        Self { target }
    }

    pub fn run_tasks(
        mut commands: Commands,
        mut all_ships_with_task: Query<(Entity, &Self, &mut TaskQueue)>,
        mut all_interaction_queues: Query<&mut InteractionQueue>,
        simulation_time: Res<SimulationTime>,
        mut finished_events: EventWriter<TaskFinishedEvent<RequestAccess>>, // TODO: Remove once Docking Task exists
    ) {
        let now = simulation_time.now();

        for (entity, task, mut task_queue) in all_ships_with_task.iter_mut() {
            let mut interaction_queue = all_interaction_queues.get_mut(task.target.into()).unwrap();

            if interaction_queue
                .try_start_interaction(&now, entity.into())
                .is_err()
            {
                task_queue.insert(1, TaskInsideQueue::AwaitingSignal);
            }

            tasks::remove_task_and_add_next_in_queue::<Self>(
                &mut commands,
                entity,
                &mut task_queue,
                now,
            );

            finished_events.send(TaskFinishedEvent::new(entity)); // TODO: Remove once Docking Task exists
        }
    }
}
