use crate::{
    event::event::{Event, EventInput, EventUpdater},
    shared::{id::Id, unit::time::Ticks},
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_TRACK_INTERFACE: &'static str = r#"
export interface Track {
  id: string;
  events: Array<Event>;
}
"#;

#[wasm_bindgen(skip_typescript)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Track {
    id: Id,
    events: HashMap<Id, Event>,
    ticks_index: BTreeMap<Ticks, HashSet<Id>>,
    end_ticks_index: BTreeMap<Ticks, HashSet<Id>>,
}

#[wasm_bindgen]
impl Track {
    pub(crate) fn new() -> Self {
        Track {
            id: Id::new(),
            events: HashMap::new(),
            ticks_index: BTreeMap::new(),
            end_ticks_index: BTreeMap::new(),
        }
    }

    pub(crate) fn get_id(&self) -> Id {
        self.id
    }

    pub(crate) fn get_event(&self, event_id: Id) -> Option<&Event> {
        self.events.get(&event_id)
    }

    pub(crate) fn get_sorted_events(&self) -> Vec<&Event> {
        self.ticks_index
            .iter()
            .map(|(_, ids)| ids.iter().filter_map(|id| self.events.get(id)))
            .flatten()
            .collect()
    }

    pub(crate) fn get_sorted_events_in_ticks_range(
        &self,
        start_ticks: Ticks,
        end_ticks: Ticks,
    ) -> Vec<&Event> {
        let got_event_ids: RefCell<HashSet<Id>> = RefCell::new(HashSet::new());

        let events: Vec<&Event> = self
            .ticks_index
            .range(start_ticks..=end_ticks)
            .map(|(_, ids)| {
                ids.iter()
                    .filter_map(|id| self.events.get(id))
                    .map(|event| {
                        got_event_ids.borrow_mut().insert(event.get_id());
                        event
                    })
            })
            .flatten()
            .collect();

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

    fn _add_event(&mut self, event: Event) -> Event {
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

        event
    }

    pub(crate) fn add_event(&mut self, event: EventInput) -> Event {
        let event = Event::from_event_input(event);
        self._add_event(event)
    }

    pub(crate) fn update_event(&mut self, updater: EventUpdater) {
        let id = updater.get_id();

        if let Some(old_event) = self.get_event(id) {
            let new_event = old_event.clone_with_updater(updater);
            self.remove_event(id);
            self._add_event(new_event);
        }
    }

    pub(crate) fn remove_event(&mut self, event_id: Id) {
        let ticks = self
            .get_event(event_id)
            .expect_throw(format!("Event with id {} does not exist", event_id.to_string()).as_str())
            .get_ticks();

        if let Some(ids) = self.ticks_index.get_mut(&ticks) {
            ids.remove(&event_id);
        }

        self.events.remove(&event_id);
    }

    pub(crate) fn to_js_object(&self) -> js_sys::Object {
        let js_event = js_sys::Object::new();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("id"),
            &JsValue::from_str(&self.get_id().to_string()),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_event,
            &JsValue::from_str("events"),
            &self
                .get_sorted_events()
                .iter()
                .map(|event| event.to_js_object())
                .collect::<js_sys::Array>(),
        )
        .unwrap();

        js_event
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::note::{NoteInput, NoteNumber, NoteUpdater, Velocity};

    #[test]
    fn test_get_sorted_event() {
        let mut track = Track::new();

        let event_input_1 = EventInput::Note(NoteInput {
            ticks: Ticks::new(480),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_1 = track.add_event(event_input_1);

        let event_input_2 = EventInput::Note(NoteInput {
            ticks: Ticks::new(960),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_2 = track.add_event(event_input_2);

        let event_input_3 = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_3 = track.add_event(event_input_3);

        let got_events = track.get_sorted_events();

        assert_eq!(got_events.len(), 3);
        assert_eq!(got_events[0].get_id(), event_3.get_id());
        assert_eq!(got_events[1].get_id(), event_1.get_id());
        assert_eq!(got_events[2].get_id(), event_2.get_id());
    }

    #[test]
    fn test_get_sorted_events_in_ticks_range() {
        let mut track = Track::new();

        let event_input_1 = EventInput::Note(NoteInput {
            ticks: Ticks::new(480),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_1 = track.add_event(event_input_1);

        let event_input_2 = EventInput::Note(NoteInput {
            ticks: Ticks::new(960),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_2 = track.add_event(event_input_2);

        let event_input_3 = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(240),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        track.add_event(event_input_3);

        let event_input_4 = EventInput::Note(NoteInput {
            ticks: Ticks::new(240),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_4 = track.add_event(event_input_4);

        let event_input_5 = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        track.add_event(event_input_5);

        let event_input_6 = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(960),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_6 = track.add_event(event_input_6);

        let event_input_7 = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(479),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        track.add_event(event_input_7);

        let event_input_8 = EventInput::Note(NoteInput {
            ticks: Ticks::new(961),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        track.add_event(event_input_8);

        let event_input_9 = EventInput::Note(NoteInput {
            ticks: Ticks::new(120),
            duration: Ticks::new(1920),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event_9 = track.add_event(event_input_9);

        let got_events = track.get_sorted_events_in_ticks_range(Ticks::new(480), Ticks::new(960));

        assert_eq!(got_events.len(), 5);
        assert_eq!(got_events[0].get_ticks(), event_6.get_ticks());
        assert_eq!(got_events[1].get_ticks(), event_9.get_ticks());
        assert_eq!(got_events[2].get_ticks(), event_4.get_ticks());
        assert_eq!(got_events[3].get_ticks(), event_1.get_ticks());
        assert_eq!(got_events[4].get_ticks(), event_2.get_ticks());
    }

    #[test]
    fn test_add_event() {
        let mut track = Track::new();

        let event_input = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event = track.add_event(event_input);

        let got_event = track.get_event(event.get_id()).unwrap();

        match (got_event, &event_input) {
            (Event::Note(got_note), EventInput::Note(note)) => {
                assert_eq!(got_note.ticks, note.ticks);
                assert_eq!(got_note.duration, note.duration);
                assert_eq!(got_note.velocity, note.velocity);
                assert_eq!(got_note.note_number, note.note_number);
            }
        }
    }

    #[test]
    fn test_update_event() {
        let mut track = Track::new();

        let event_input = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event = track.add_event(event_input);
        let event_id = event.get_id();

        let event_updater = EventUpdater::Note(NoteUpdater {
            id: event_id,
            ticks: Some(Ticks::new(480)),
            duration: Some(Ticks::new(960)),
            velocity: Some(Velocity::new(80)),
            note_number: Some(NoteNumber::new(80)),
        });
        track.update_event(event_updater);

        let got_event = track.get_event(event_id).unwrap();

        match (got_event, event_updater) {
            (Event::Note(got_note), EventUpdater::Note(note)) => {
                assert_eq!(got_note.id, note.id);
                assert_eq!(got_note.ticks, note.ticks.unwrap());
                assert_eq!(got_note.duration, note.duration.unwrap());
                assert_eq!(got_note.velocity, note.velocity.unwrap());
                assert_eq!(got_note.note_number, note.note_number.unwrap());
            }
        }
    }

    #[test]
    fn test_remove_event() {
        let mut track = Track::new();

        let event_input = EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        });
        let event = track.add_event(event_input);
        let event_id = event.get_id();

        track.remove_event(event_id);

        assert!(track.get_event(event_id).is_none());
    }
}
