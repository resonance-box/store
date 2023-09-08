use super::note::{Note, NoteUpdater};
use crate::shared::{id::Id, unit::time::Ticks};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const TS_EVENT_INTERFACES: &'static str = r#"
export type Event = Note;

export type EventUpdater = NoteUpdater;
"#;

#[wasm_bindgen]
#[derive(Debug)]
pub enum EventKind {
    Note = "Note",
}

impl Display for EventKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EventKind::Note => write!(f, "Note"),
            _ => panic!("Unknown event kind: {}", self),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Event {
    Note(Note),
}

impl Event {
    pub(crate) fn clone_with_updater(&self, updater: EventUpdater) -> Self {
        match (self, updater) {
            (Event::Note(note), EventUpdater::Note(note_updater)) => {
                Event::Note(note.clone_with_updater(note_updater))
            }
        }
    }

    pub(crate) fn get_id(&self) -> Id {
        match self {
            Event::Note(note) => note.id,
        }
    }

    pub(crate) fn get_ticks(&self) -> Ticks {
        match self {
            Event::Note(note) => note.ticks,
        }
    }

    pub(crate) fn get_duration(&self) -> Option<Ticks> {
        match self {
            Event::Note(note) => Some(note.duration),
        }
    }

    pub(crate) fn get_track_id(&self) -> Id {
        match self {
            Event::Note(note) => note.track_id,
        }
    }

    pub(crate) fn from_js_object(obj: js_sys::Object) -> Self {
        let kind = js_sys::Reflect::get(&obj, &JsValue::from_str("kind"))
            .unwrap()
            .as_string()
            .unwrap();
        let kind = EventKind::from_str(&kind).unwrap();

        match kind {
            EventKind::Note => Event::Note(Note::from_js_object(obj)),
            _ => panic!("Unknown event kind: {}", kind),
        }
    }

    pub(crate) fn to_js_object(&self) -> js_sys::Object {
        match self {
            Event::Note(note) => note.to_js_object(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum EventUpdater {
    Note(NoteUpdater),
}

impl EventUpdater {
    pub(crate) fn get_id(&self) -> Id {
        match self {
            EventUpdater::Note(note) => note.id,
        }
    }

    pub(crate) fn from_js_object(obj: js_sys::Object) -> Self {
        let kind = js_sys::Reflect::get(&obj, &JsValue::from_str("kind"))
            .unwrap()
            .as_string()
            .unwrap();
        let kind = EventKind::from_str(&kind).unwrap();

        match kind {
            EventKind::Note => EventUpdater::Note(NoteUpdater::from_js_object(obj)),
            _ => panic!("Unknown event kind: {}", kind),
        }
    }
}
