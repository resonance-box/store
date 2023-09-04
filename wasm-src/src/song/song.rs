use crate::{
    event::event::{Event, EventInput, EventUpdater},
    shared::{id::Id, unit::time::Ticks},
    track::track::{Track, TrackVec},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    vec,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_SONG_INTERFACE: &'static str = r#"
export interface Song {
  title: string;
  ppq: number;
  endOfSong: number;
  tracks: Track[];
}
"#;

#[derive(Clone)]
pub(crate) struct GetEventsFilter {
    track_ids: Option<Vec<Id>>,
}

#[derive(Clone)]
pub struct Song {
    pub(crate) title: String,
    pub(crate) ppq: u32,
    pub(crate) end_of_song: Ticks,
    tracks: TrackVec,
    events: HashMap<Id, Event>,
    ticks_index: BTreeMap<Ticks, HashSet<Id>>,
    end_ticks_index: BTreeMap<Ticks, HashSet<Id>>,
}

impl Song {
    pub fn new(title: String, ppq: u32) -> Self {
        Song {
            title,
            ppq,
            end_of_song: Ticks::new(0),
            tracks: TrackVec::new(),
            events: HashMap::new(),
            ticks_index: BTreeMap::new(),
            end_ticks_index: BTreeMap::new(),
        }
    }

    pub(crate) fn get_track(&self, track_id: &Id) -> Option<&Track> {
        self.tracks.iter().find(|track| track.id == *track_id)
    }

    fn get_track_mut(&mut self, track_id: &Id) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|track| track.id == *track_id)
    }

    pub(crate) fn get_tracks(&self) -> &TrackVec {
        &self.tracks
    }

    pub(crate) fn add_empty_track(&mut self) -> &Track {
        let current_track_count = self.tracks.len();

        let track = Track::new();
        self.tracks.push(track);
        self.tracks.get(current_track_count).unwrap()
    }

    pub(crate) fn remove_track(&mut self, track_id: &Id) {
        if let Some(index) = self.tracks.iter().position(|track| track.id == *track_id) {
            if let Some(track) = self.get_track(track_id) {
                let event_ids_to_remove: Vec<_> = track
                    .get_events()
                    .iter()
                    .map(|event| event.get_id())
                    .collect();

                for event_id in event_ids_to_remove {
                    self.remove_event(&event_id);
                }
            }

            self.tracks.remove(index);
        }
    }

    pub(crate) fn get_event(&self, event_id: &Id) -> Option<&Event> {
        self.events.get(event_id)
    }

    fn merge_events_each_track<'a, F>(
        &self,
        track_ids: Vec<Id>,
        get_events_from_track_id_fn: F,
    ) -> Vec<&'a Event>
    where
        F: Fn(&Id) -> Vec<&'a Event>,
    {
        let mut events = Vec::new();

        let mut events_each_track: Vec<_> = track_ids
            .iter()
            .map(|track_id| get_events_from_track_id_fn(track_id).into_iter())
            .collect();

        let mut current_event_caches: Vec<Option<&Event>> = vec![None; events_each_track.len()];

        loop {
            let mut min_ticks = u32::MAX;
            let mut min_event = None;
            let mut min_track_index = None;

            for (track_index, events) in events_each_track.iter_mut().enumerate() {
                let event = current_event_caches[track_index].or_else(|| events.next());

                if let Some(event) = event {
                    if event.get_ticks().as_u32() < min_ticks {
                        min_ticks = event.get_ticks().as_u32();
                        min_event = Some(event);
                        min_track_index = Some(track_index);
                    }

                    current_event_caches[track_index] = Some(event);
                }
            }

            match (min_track_index, min_event) {
                (Some(min_track_index), Some(min_event)) => {
                    events.push(min_event);
                    current_event_caches[min_track_index] = None;
                }
                (None, None) => break,
                _ => panic!("unexpected"),
            }
        }

        events
    }

    pub(crate) fn get_events(&self, filter: Option<GetEventsFilter>) -> Vec<&Event> {
        if let Some(track_ids) = filter.and_then(|f| f.track_ids) {
            return self.merge_events_each_track(track_ids, |track_id| {
                self.get_track(track_id)
                    .map(|track| track.get_events())
                    .unwrap_or_default()
            });
        }

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
        filter: Option<GetEventsFilter>,
    ) -> Vec<&Event> {
        if let Some(track_ids) = filter.clone().and_then(|f| f.track_ids) {
            return self.merge_events_each_track(track_ids, |track_id| {
                self.get_track(track_id)
                    .map(|track| {
                        track.get_events_in_ticks_range(start_ticks, end_ticks, within_duration)
                    })
                    .unwrap_or_default()
            });
        }

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

    fn _add_event(&mut self, event: Event) {
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

    pub(crate) fn add_event(&mut self, event: EventInput) -> Event {
        let event = Event::from_event_input(event);
        let track_id = event.get_track_id();

        self._add_event(event);
        self.get_track_mut(&track_id)
            .map(|track| track.add_event(event));

        event
    }

    pub(crate) fn update_event(&mut self, updater: EventUpdater) -> Event {
        let id = updater.get_id();
        let event = self.events.get(&id).expect_throw("Event not found");
        let event = event.clone_with_updater(updater);
        self.remove_event(&id);
        self._add_event(event);
        event
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

        let track_id = event.get_track_id();
        if let Some(track) = self.get_track_mut(&track_id) {
            track.remove_event(&event_id);
        }

        self.events.remove(&event_id);
    }

    pub(crate) fn to_js_object(&self) -> js_sys::Object {
        let js_song = js_sys::Object::new();

        js_sys::Reflect::set(
            &js_song,
            &JsValue::from_str("title"),
            &JsValue::from_str(self.title.as_str()),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_song,
            &JsValue::from_str("ppq"),
            &JsValue::from_f64(self.ppq as f64),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_song,
            &JsValue::from_str("endOfSong"),
            &JsValue::from_f64(self.end_of_song.as_u32() as f64),
        )
        .unwrap();

        js_sys::Reflect::set(
            &js_song,
            &JsValue::from_str("tracks"),
            &self.get_tracks().to_js_array(),
        )
        .unwrap();

        js_song
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{
        event::EventInput,
        note::{NoteInput, NoteNumber, Velocity},
    };

    #[test]
    fn test_song() {
        let song = Song::new("test".to_string(), 480);
        assert_eq!(song.title, "test");
        assert_eq!(song.ppq, 480);
        assert_eq!(song.end_of_song, Ticks::new(0));
    }

    #[test]
    fn test_tracks_scenario() {
        let mut song = Song::new("test".to_string(), 480);
        song.add_empty_track();
        song.add_empty_track();

        let tracks = song.get_tracks();
        assert_eq!(tracks.len(), 2);

        let first_track_id = tracks[0].id;
        let track = song.get_track(&first_track_id).unwrap();
        assert_eq!(track.id, tracks[0].id);

        let track_id = tracks[0].id;
        song.remove_track(&track_id);
        let tracks = song.get_tracks();
        assert_eq!(tracks.len(), 1);

        assert!(song.get_track(&first_track_id).is_none());
    }

    #[test]
    fn test_events_scenario() {
        let mut song = Song::new("test".to_string(), 480);

        let track1 = song.add_empty_track();
        let track_id1 = track1.id;

        let track2 = song.add_empty_track();
        let track_id2 = track2.id;

        let event1 = song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(240),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));
        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(480),
            duration: Ticks::new(720),
            velocity: Velocity::new(90),
            note_number: NoteNumber::new(72),
            track_id: track_id2,
        }));

        let event = song.get_event(&event1.get_id()).unwrap();
        assert_eq!(event.get_ticks().as_u32(), 240);

        let events = song.get_events(None);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].get_ticks().as_u32(), 240);

        let events = song.get_events(Some(GetEventsFilter {
            track_ids: Some(vec![track_id1]),
        }));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].get_ticks().as_u32(), 240);

        song.remove_event(&event.get_id());
        let events = song.get_events(None);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_get_events() {
        let mut song = Song::new("test".to_string(), 480);

        let track1 = song.add_empty_track();
        let track_id1 = track1.id;

        let track2 = song.add_empty_track();
        let track_id2 = track2.id;

        let track3 = song.add_empty_track();
        let track_id3 = track3.id;

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(480),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(240),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id2,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(960),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id2,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(720),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id3,
        }));

        let events = song.get_events(None);

        assert_eq!(events.len(), 5);
        assert_eq!(events[0].get_ticks().as_u32(), 0);
        assert_eq!(events[1].get_ticks().as_u32(), 240);
        assert_eq!(events[2].get_ticks().as_u32(), 480);
        assert_eq!(events[3].get_ticks().as_u32(), 720);
        assert_eq!(events[4].get_ticks().as_u32(), 960);

        let events = song.get_events(Some(GetEventsFilter {
            track_ids: Some(vec![track_id1]),
        }));

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].get_ticks().as_u32(), 240);
        assert_eq!(events[1].get_ticks().as_u32(), 480);
    }

    fn create_tracks_and_events(song: &mut Song) -> [Id; 2] {
        let track1 = song.add_empty_track();
        let track_id1 = track1.id;

        let track2 = song.add_empty_track();
        let track_id2 = track2.id;

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(480),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(959),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id2,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(240),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(240),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id2,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(960),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id2,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(479),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(960),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id2,
        }));

        song.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(120),
            duration: Ticks::new(1920),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
            track_id: track_id1,
        }));

        [track_id1, track_id2]
    }

    #[test]
    fn test_get_events_in_ticks_range_within_duration() {
        let mut song = Song::new("test".to_string(), 480);

        let [track_id1, track_id2] = self::create_tracks_and_events(&mut song);

        let events = song.get_events_in_ticks_range(Ticks::new(480), Ticks::new(960), true, None);
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].get_ticks().as_u32(), 0);
        assert_eq!(events[1].get_ticks().as_u32(), 120);
        assert_eq!(events[2].get_ticks().as_u32(), 240);
        assert_eq!(events[3].get_ticks().as_u32(), 480);
        assert_eq!(events[4].get_ticks().as_u32(), 959);

        let events = song.get_events_in_ticks_range(
            Ticks::new(480),
            Ticks::new(960),
            true,
            Some(GetEventsFilter {
                track_ids: Some(vec![track_id1]),
            }),
        );
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].get_ticks().as_u32(), 120);
        assert_eq!(events[1].get_ticks().as_u32(), 480);

        let events = song.get_events_in_ticks_range(
            Ticks::new(480),
            Ticks::new(960),
            true,
            Some(GetEventsFilter {
                track_ids: Some(vec![track_id2]),
            }),
        );
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].get_ticks().as_u32(), 0);
        assert_eq!(events[1].get_ticks().as_u32(), 240);
        assert_eq!(events[2].get_ticks().as_u32(), 959);
    }

    #[test]
    fn test_get_events_in_ticks_range_without_duration() {
        let mut song = Song::new("test".to_string(), 480);

        let [track_id1, track_id2] = self::create_tracks_and_events(&mut song);

        let events = song.get_events_in_ticks_range(Ticks::new(480), Ticks::new(960), false, None);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].get_ticks().as_u32(), 480);
        assert_eq!(events[1].get_ticks().as_u32(), 959);

        let events = song.get_events_in_ticks_range(
            Ticks::new(480),
            Ticks::new(960),
            false,
            Some(GetEventsFilter {
                track_ids: Some(vec![track_id1]),
            }),
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].get_ticks().as_u32(), 480);

        let events = song.get_events_in_ticks_range(
            Ticks::new(480),
            Ticks::new(960),
            false,
            Some(GetEventsFilter {
                track_ids: Some(vec![track_id2]),
            }),
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].get_ticks().as_u32(), 959);
    }
}
