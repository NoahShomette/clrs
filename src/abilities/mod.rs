mod nuke;

use bevy::prelude::{FromReflect, Reflect};

#[derive(Default, Clone, Copy, Eq, Hash, Debug, PartialEq, Reflect, FromReflect)]
pub enum Abilities{
    #[default]
    Nuke,
    Sacrifice,
    Boost,
}