use crate::shared::{id::Id, unit::time::Ticks};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const TS_NOTE_INTERFACES: &'static str = r#"
export interface Note {
  id: string;
  kind: "Note";
  ticks: number;
  duration: number;
  velocity: number;
  noteNumber: number;
  trackId: string;
}

export interface NoteUpdater {
  id: string;
  kind: "Note";
  ticks?: number;
  duration?: number;
  velocity?: number;
  noteNumber?: number;
  trackId?: string;
}
"#;

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
    pub(crate) track_id: Id,
}

impl Note {
    pub(crate) fn clone_with_updater(&self, updater: NoteUpdater) -> Self {
        Note {
            id: self.id,
            ticks: updater.ticks.unwrap_or(self.ticks),
            duration: updater.duration.unwrap_or(self.duration),
            velocity: updater.velocity.unwrap_or(self.velocity),
            note_number: updater.note_number.unwrap_or(self.note_number),
            track_id: updater.track_id.unwrap_or(self.track_id),
        }
    }

    pub(crate) fn from_js_object(obj: js_sys::Object) -> Self {
        let id = js_sys::Reflect::get(&obj, &JsValue::from_str("id"))
            .unwrap()
            .as_string()
            .unwrap();

        let ticks = js_sys::Reflect::get(&obj, &JsValue::from_str("ticks"))
            .unwrap()
            .as_f64()
            .unwrap();

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

        let track_id = js_sys::Reflect::get(&obj, &JsValue::from_str("trackId"))
            .unwrap()
            .as_string()
            .unwrap();

        Note {
            id: Id::try_from(id.as_str()).unwrap(),
            ticks: Ticks::new(ticks as u32),
            duration: Ticks::new(duration as u32),
            velocity: Velocity::new(velocity as u8),
            note_number: NoteNumber::new(note_number as u8),
            track_id: Id::try_from(track_id.as_str()).unwrap(),
        }
    }

    pub(crate) fn to_js_object(&self) -> js_sys::Object {
        let js_event = js_sys::Object::new();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("id"),
            &JsValue::from_str(self.id.to_string().as_str()),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("kind"),
            &JsValue::from_str("Note"),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("ticks"),
            &JsValue::from_f64(self.ticks.as_u32() as f64),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("duration"),
            &JsValue::from_f64(self.duration.as_u32() as f64),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("velocity"),
            &JsValue::from_f64(self.velocity.as_u8() as f64),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("noteNumber"),
            &JsValue::from_f64(self.note_number.as_u8() as f64),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("trackId"),
            &JsValue::from_str(self.track_id.to_string().as_str()),
        )
        .unwrap();

        js_event
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct NoteUpdater {
    pub(crate) id: Id,
    pub(crate) ticks: Option<Ticks>,
    pub(crate) duration: Option<Ticks>,
    pub(crate) velocity: Option<Velocity>,
    pub(crate) note_number: Option<NoteNumber>,
    pub(crate) track_id: Option<Id>,
}

impl NoteUpdater {
    pub(crate) fn from_js_object(obj: js_sys::Object) -> Self {
        let id = js_sys::Reflect::get(&obj, &JsValue::from_str("id"))
            .unwrap()
            .as_string()
            .unwrap();

        let ticks = js_sys::Reflect::get(&obj, &JsValue::from_str("ticks"))
            .unwrap()
            .as_f64();

        let duration = js_sys::Reflect::get(&obj, &JsValue::from_str("duration"))
            .unwrap()
            .as_f64();

        let velocity = js_sys::Reflect::get(&obj, &JsValue::from_str("velocity"))
            .unwrap()
            .as_f64();

        let note_number = js_sys::Reflect::get(&obj, &JsValue::from_str("noteNumber"))
            .unwrap()
            .as_f64();

        let track_id = js_sys::Reflect::get(&obj, &JsValue::from_str("trackId"))
            .unwrap()
            .as_string();

        NoteUpdater {
            id: Id::try_from(id.as_str()).unwrap(),
            ticks: ticks.map(|t| Ticks::new(t as u32)),
            duration: duration.map(|d| Ticks::new(d as u32)),
            velocity: velocity.map(|v| Velocity::new(v as u8)),
            note_number: note_number.map(|n| NoteNumber::new(n as u8)),
            track_id: track_id.map(|t| Id::try_from(t.as_str()).unwrap()),
        }
    }
}
