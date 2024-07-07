use bevy::prelude::{Component, Entity};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Wrapper around [Entity] to guarantee type safety when storing Entities for specific component combinations.
pub struct TypedEntity<T: Component>(Entity, PhantomData<T>);

impl<T: Component> Display for TypedEntity<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", std::any::type_name::<T>(), self.0)
    }
}

impl<T: Component> From<Entity> for TypedEntity<T> {
    fn from(value: Entity) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: Component> From<TypedEntity<T>> for Entity {
    fn from(value: TypedEntity<T>) -> Self {
        value.0
    }
}
impl<T: Component> From<&TypedEntity<T>> for Entity {
    fn from(value: &TypedEntity<T>) -> Self {
        value.0
    }
}

impl<T: Component> Copy for TypedEntity<T> {}
impl<T: Component> Clone for TypedEntity<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Component> Eq for TypedEntity<T> {}
impl<T: Component> PartialEq<Self> for TypedEntity<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: Component> PartialEq<Entity> for TypedEntity<T> {
    fn eq(&self, other: &Entity) -> bool {
        &self.0 == other
    }
}

impl<T: Component> PartialEq<TypedEntity<T>> for Entity {
    fn eq(&self, other: &TypedEntity<T>) -> bool {
        self == &other.0
    }
}

impl<T: Component> Ord for TypedEntity<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Component> PartialOrd<Self> for TypedEntity<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Component> Hash for TypedEntity<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
