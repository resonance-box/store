use crate::{event::event::Event, shared::id::Id};
use std::ops::Deref;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_TRACK_INTERFACE: &'static str = r#"
export interface Track {
  id: string;
  events: Event[];
}
"#;

pub struct Track<'a> {
    pub(crate) id: Id,
    pub(crate) events: Vec<&'a Event>,
}

impl<'a> Track<'a> {
    pub(crate) fn new(id: Id, events: Vec<&'a Event>) -> Self {
        Track { id, events }
    }

    pub(crate) fn to_js_object(&self) -> js_sys::Object {
        let js_track = js_sys::Object::new();

        js_sys::Reflect::set(
            &js_track,
            &JsValue::from_str("id"),
            &JsValue::from_str(self.id.to_string().as_str()),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_track,
            &JsValue::from_str("events"),
            &self
                .events
                .iter()
                .map(|event| event.to_js_object())
                .collect::<js_sys::Array>(),
        )
        .unwrap();

        js_track
    }
}

pub struct TrackVec<'a>(Vec<Track<'a>>);

impl<'a> TrackVec<'a> {
    pub(crate) fn to_js_array(&self) -> js_sys::Array {
        self.0.iter().map(|track| track.to_js_object()).collect()
    }
}

impl<'a> FromIterator<Track<'a>> for TrackVec<'a> {
    fn from_iter<T: IntoIterator<Item = Track<'a>>>(iter: T) -> Self {
        TrackVec(iter.into_iter().collect())
    }
}

impl<'a> Deref for TrackVec<'a> {
    type Target = Vec<Track<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
