use crate::poison::OrPoisoned;
use crate::slots::{new_key_type, SlotMap};

use std::sync::OnceLock;

use std::{
    any::Any,
    hash::Hash,
    sync::{Arc, RwLock},
};

new_key_type! {
    /// Unique identifier for an item stored in the arena.
    pub struct NodeId;
}

pub struct Arena;

pub type ArenaMap = SlotMap<NodeId, Box<dyn Any + Send + Sync>>;

static MAP: OnceLock<RwLock<ArenaMap>> = OnceLock::new();

impl Arena {
    #[inline(always)]
    #[allow(unused)]
    pub fn set(arena: &Arc<RwLock<ArenaMap>>) {}

    #[track_caller]
    pub fn with<U>(fun: impl FnOnce(&ArenaMap) -> U) -> U {
        {
            fun(&MAP.get_or_init(Default::default).read().or_poisoned())
        }
    }

    #[track_caller]
    pub fn try_with<U>(fun: impl FnOnce(&ArenaMap) -> U) -> Option<U> {
        {
            Some(fun(&MAP.get_or_init(Default::default).read().or_poisoned()))
        }
    }

    #[track_caller]
    pub fn with_mut<U>(fun: impl FnOnce(&mut ArenaMap) -> U) -> U {
        {
            fun(&mut MAP.get_or_init(Default::default).write().or_poisoned())
        }
    }

    #[track_caller]
    pub fn try_with_mut<U>(fun: impl FnOnce(&mut ArenaMap) -> U) -> Option<U> {
        {
            Some(fun(&mut MAP
                .get_or_init(Default::default)
                .write()
                .or_poisoned()))
        }
    }
}
