use super::note::{Note, NoteInput, NoteNumber, NoteUpdater, Velocity};
use crate::shared::{id::Id, unit::time::Ticks};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const TS_EVENT_INTERFACES: &'static str = r#"
export interface Note {
  id: string;
  ticks: number;
  duration: number;
  velocity: number;
  noteNumber: number;    
}

export interface NoteInput {
  kind: "Note";
  ticks: number;
  duration: number;
  velocity: number;
  noteNumber: number;    
}

export interface NoteUpdater {
  id: string;
  kind: "Note";
  ticks?: number;
  duration?: number;
  velocity?: number;
  noteNumber?: number;    
}

export type Event = Note;

export type EventInput = NoteInput;

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
    pub(crate) fn from_event_input(event: EventInput) -> Self {
        let id = Id::new();

        match event {
            EventInput::Note(note) => Event::Note(Note {
                id,
                ticks: note.ticks,
                duration: note.duration,
                velocity: note.velocity,
                note_number: note.note_number,
            }),
        }
    }

    pub(crate) fn clone_with_updater(&self, updater: EventUpdater) -> Self {
        match (self, updater) {
            (Event::Note(note), EventUpdater::Note(note_updater)) => Event::Note(Note {
                id: note.id,
                ticks: note_updater.ticks.unwrap_or(note.ticks),
                duration: note_updater.duration.unwrap_or(note.duration),
                velocity: note_updater.velocity.unwrap_or(note.velocity),
                note_number: note_updater.note_number.unwrap_or(note.note_number),
            }),
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

    pub(crate) fn to_js_object(&self) -> js_sys::Object {
        match self {
            Event::Note(note) => {
                let js_event = js_sys::Object::new();
                js_sys::Reflect::set(
                    &js_event,
                    &JsValue::from_str("id"),
                    &JsValue::from_str(note.id.to_string().as_str()),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_event,
                    &JsValue::from_str("ticks"),
                    &JsValue::from_f64(note.ticks.as_u32() as f64),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_event,
                    &JsValue::from_str("duration"),
                    &JsValue::from_f64(note.duration.as_u32() as f64),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_event,
                    &JsValue::from_str("velocity"),
                    &JsValue::from_f64(note.velocity.as_u8() as f64),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_event,
                    &JsValue::from_str("noteNumber"),
                    &JsValue::from_f64(note.note_number.as_u8() as f64),
                )
                .unwrap();
                js_event
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum EventInput {
    Note(NoteInput),
}

impl EventInput {
    pub(crate) fn from_js_object(obj: js_sys::Object) -> Self {
        let ticks = js_sys::Reflect::get(&obj, &JsValue::from_str("ticks"))
            .unwrap()
            .as_f64()
            .unwrap();

        let kind = js_sys::Reflect::get(&obj, &JsValue::from_str("kind"))
            .unwrap()
            .as_string()
            .unwrap();
        let kind = EventKind::from_str(&kind).unwrap();

        match kind {
            EventKind::Note => {
                let duration = js_sys::Reflect::get(&obj, &JsValue::from_str("duration"))
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let velocity = js_sys::Reflect::get(&obj, &JsValue::from_str("velocity"))
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let note_number = js_sys::Reflect::get(&obj, &JsValue::from_str("noteNumber"))
                    .unwrap()
                    .as_f64()
                    .unwrap();

                let note = NoteInput {
                    ticks: Ticks::new(ticks as u32),
                    duration: Ticks::new(duration as u32),
                    velocity: Velocity::new(velocity as u8),
                    note_number: NoteNumber::new(note_number as u8),
                };

                EventInput::Note(note)
            }
            _ => panic!("Unknown event kind: {}", kind),
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
        let id = js_sys::Reflect::get(&obj, &JsValue::from_str("id"))
            .unwrap()
            .as_string()
            .unwrap();
        let ticks = js_sys::Reflect::get(&obj, &JsValue::from_str("ticks"))
            .unwrap()
            .as_f64();

        let kind = js_sys::Reflect::get(&obj, &JsValue::from_str("kind"))
            .unwrap()
            .as_string()
            .unwrap();
        let kind = EventKind::from_str(&kind).unwrap();

        match kind {
            EventKind::Note => {
                let duration = js_sys::Reflect::get(&obj, &JsValue::from_str("duration"))
                    .unwrap()
                    .as_f64();
                let velocity = js_sys::Reflect::get(&obj, &JsValue::from_str("velocity"))
                    .unwrap()
                    .as_f64();
                let note_number = js_sys::Reflect::get(&obj, &JsValue::from_str("noteNumber"))
                    .unwrap()
                    .as_f64();

                let note = NoteUpdater {
                    id: Id::try_from(id.as_str()).unwrap(),
                    ticks: ticks.map(|t| Ticks::new(t as u32)),
                    duration: duration.map(|d| Ticks::new(d as u32)),
                    velocity: velocity.map(|v| Velocity::new(v as u8)),
                    note_number: note_number.map(|n| NoteNumber::new(n as u8)),
                };

                EventUpdater::Note(note)
            }
            _ => panic!("Unknown event kind: {}", kind),
        }
    }
}
