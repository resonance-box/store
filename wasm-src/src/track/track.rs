use crate::{
    event::event::Event,
    shared::{id::Id, unit::time::Ticks},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    ops::{Deref, DerefMut},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_TRACK_INTERFACE: &'static str = r#"
export interface Track {
  id: string;
  events: Event[];
}
"#;

#[derive(Clone)]
pub struct Track {
    pub(crate) id: Id,
    events: HashMap<Id, Event>,
    ticks_index: BTreeMap<Ticks, HashSet<Id>>,
    end_ticks_index: BTreeMap<Ticks, HashSet<Id>>,
}

impl Track {
    pub(crate) fn new(id: Id, events: Option<Vec<Event>>) -> Self {
        let mut track = Track {
            id,
            events: HashMap::new(),
            ticks_index: BTreeMap::new(),
            end_ticks_index: BTreeMap::new(),
        };

        if let Some(events) = events {
            for event in events {
                track.add_event(event);
            }
        }

        track
    }

    pub(crate) fn get_event(&self, event_id: &Id) -> Option<&Event> {
        self.events.get(event_id)
    }

    pub(crate) fn get_events(&self) -> Vec<&Event> {
        self.ticks_index
            .iter()
            .map(|(_, ids)| ids.iter().filter_map(|id| self.events.get(id)))
            .flatten()
            .collect()
    }

    pub(crate) fn get_events_in_ticks_range(
        &self,
        start_ticks: Ticks,
        end_ticks: Ticks,
        within_duration: bool,
    ) -> Vec<&Event> {
        let got_event_ids: RefCell<HashSet<Id>> = RefCell::new(HashSet::new());

        // TODO: refactor
        let events: Vec<&Event> = self
            .ticks_index
            .range(start_ticks..end_ticks)
            .map(|(_, ids)| {
                ids.iter()
                    .filter_map(|id| self.events.get(id))
                    .map(|event| {
                        if within_duration {
                            got_event_ids.borrow_mut().insert(event.get_id());
                        }
                        event
                    })
            })
            .flatten()
            .collect();

        if !within_duration {
            return events;
        }

        // TODO: refactor
        let tick = Ticks::new(1);
        let mut has_duration_events: Vec<&Event> = self
            .end_ticks_index
            .range((start_ticks + tick)..)
            .map(|(_, ids)| {
                ids.iter()
                    .filter_map(|id| self.events.get(id))
                    .filter(|event| {
                        event.get_ticks() < start_ticks
                            && !got_event_ids.borrow().contains(&event.get_id())
                    })
            })
            .flatten()
            .collect();

        // MEMO: can it be implemented so that it does not need to be sorted?
        has_duration_events.sort_by(|a, b| a.get_ticks().cmp(&b.get_ticks()));

        let mut merged_events = Vec::with_capacity(events.len() + has_duration_events.len());

        let (mut i, mut j) = (0, 0);
        while i < events.len() && j < has_duration_events.len() {
            if events[i].get_ticks() <= has_duration_events[j].get_ticks() {
                merged_events.push(events[i]);
                i += 1;
            } else {
                merged_events.push(has_duration_events[j]);
                j += 1;
            }
        }

        merged_events.extend_from_slice(&events[i..]);
        merged_events.extend_from_slice(&has_duration_events[j..]);

        merged_events
    }

    pub(crate) fn add_event(&mut self, event: Event) {
        let id = event.get_id();
        let ticks = event.get_ticks();

        self.events.insert(id, event);

        self.ticks_index
            .entry(ticks)
            .or_insert_with(HashSet::new)
            .insert(id);

        if let Some(duration) = event.get_duration() {
            let end_ticks = ticks + duration;

            self.end_ticks_index
                .entry(end_ticks)
                .or_insert_with(HashSet::new)
                .insert(id);
        }
    }

    pub(crate) fn remove_event(&mut self, event_id: &Id) {
        let event = self.events.get(&event_id).expect_throw("Event not found");

        let ticks = self
            .get_event(event_id)
            .expect_throw(format!("Event with id {} does not exist", event_id.to_string()).as_str())
            .get_ticks();

        if let Some(ids) = self.ticks_index.get_mut(&ticks) {
            ids.remove(&event_id);
        }

        if let Some(duration) = event.get_duration() {
            let end_ticks = ticks + duration;

            if let Some(ids) = self.end_ticks_index.get_mut(&end_ticks) {
                ids.remove(&event_id);
            }
        }

        self.events.remove(&event_id);
    }

    pub(crate) fn from_js_object(obj: js_sys::Object) -> Self {
        let id = js_sys::Reflect::get(&obj, &JsValue::from_str("id"))
            .unwrap()
            .as_string()
            .unwrap();
        let id = Id::try_from(id.as_str()).unwrap();

        let events: Vec<Event> =
            js_sys::Array::from(&js_sys::Reflect::get(&obj, &JsValue::from_str("events")).unwrap())
                .iter()
                .map(|event| js_sys::Object::from(event))
                .map(|event| Event::from_js_object(event))
                .collect();

        Track::new(id, Some(events))
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
                .get_events()
                .iter()
                .map(|event| event.to_js_object())
                .collect::<js_sys::Array>(),
        )
        .unwrap();

        js_track
    }
}

#[derive(Clone)]
pub struct TrackVec(Vec<Track>);

impl TrackVec {
    pub(crate) fn new() -> Self {
        TrackVec(Vec::new())
    }

    pub(crate) fn to_js_array(&self) -> js_sys::Array {
        self.0.iter().map(|track| track.to_js_object()).collect()
    }
}

impl FromIterator<Track> for TrackVec {
    fn from_iter<T: IntoIterator<Item = Track>>(iter: T) -> Self {
        TrackVec(iter.into_iter().collect())
    }
}

impl Deref for TrackVec {
    type Target = Vec<Track>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TrackVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::event::note::{Note, NoteNumber, Velocity};

    use super::*;

    #[test]
    fn test_new_track() {
        let id = Id::new();
        let track = Track::new(id, None);
        assert_eq!(track.id, id);
        assert_eq!(track.events.len(), 0);
        assert_eq!(track.ticks_index.len(), 0);
        assert_eq!(track.end_ticks_index.len(), 0);
    }

    #[test]
    fn test_new_track_with_events() {
        let track_id = Id::new();

        let event1 = Event::Note(Note {
            id: Id::new(),
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id,
        });

        let event2 = Event::Note(Note {
            id: Id::new(),
            ticks: Ticks::new(480),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id,
        });

        let track = Track::new(track_id, Some(vec![event1, event2]));
        assert_eq!(track.id, track_id);
        assert_eq!(track.get_events().len(), 2);
    }
}
