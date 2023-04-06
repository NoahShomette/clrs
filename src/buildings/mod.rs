pub mod pulser;

use bevy::prelude::{Component, FromReflect, Reflect};

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Building<T> {
    pub building_type: T,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Pulser{
    pub strength: u32,
}
