use serde::{Deserialize, Serialize};

use crate::shared::{id::Id, unit::time::Ticks};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Velocity(u8);

impl Velocity {
    pub fn new(value: u8) -> Self {
        Velocity(value)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NoteNumber(u8);

impl NoteNumber {
    pub fn new(value: u8) -> Self {
        NoteNumber(value)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Note {
    pub(crate) id: Id,
    pub(crate) ticks: Ticks,
    pub(crate) duration: Ticks,
    pub(crate) velocity: Velocity,
    pub(crate) note_number: NoteNumber,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct NoteInput {
    pub(crate) ticks: Ticks,
    pub(crate) duration: Ticks,
    pub(crate) velocity: Velocity,
    pub(crate) note_number: NoteNumber,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct NoteUpdater {
    pub(crate) id: Id,
    pub(crate) ticks: Option<Ticks>,
    pub(crate) duration: Option<Ticks>,
    pub(crate) velocity: Option<Velocity>,
    pub(crate) note_number: Option<NoteNumber>,
}
