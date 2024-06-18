use bevy::prelude::{Component, Entity};
use std::collections::VecDeque;

/// A single Task which can be scheduled for individual ships.
pub enum ShipTask {
    DoNothing,
    MoveTo(Entity),
}

/// A queue of [ShipTasks].
#[derive(Component)]
pub struct TaskQueue {
    pub queue: VecDeque<ShipTask>,
}
