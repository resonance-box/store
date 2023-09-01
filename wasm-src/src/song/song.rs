use crate::{
    event::event::Event,
    shared::{id::Id, unit::time::Ticks},
    track::track::Track,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_SONG_INTERFACE: &'static str = r#"
export interface Song {
  title: string;
  ppq: number;
  endOfSong: number;
}
"#;

#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
pub struct Song {
    pub(crate) title: String,
    pub(crate) ppq: u32,
    pub(crate) end_of_song: Ticks,
    pub(crate) tracks: Vec<Track>,
}

#[wasm_bindgen]
impl Song {
    pub(crate) fn new(title: String, ppq: u32) -> Self {
        Song {
            title,
            ppq,
            end_of_song: Ticks::new(0),
            tracks: Vec::new(),
        }
    }

    pub(crate) fn get_tracks(&self) -> Vec<&Track> {
        self.tracks.iter().map(|t| t).collect()
    }

    pub(crate) fn get_track(&self, track_id: Id) -> Option<&Track> {
        self.tracks.iter().find(|t| t.get_id() == track_id)
    }

    pub(crate) fn get_track_mut(&mut self, track_id: Id) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.get_id() == track_id)
    }

    fn merge_events_each_track<F>(&self, get_sorted_events_fn: F) -> Vec<&Event>
    where
        F: Fn(&Track) -> Vec<&Event>,
    {
        let mut events = Vec::new();

        let mut events_each_track: Vec<_> = self
            .tracks
            .iter()
            .map(|t| get_sorted_events_fn(t).into_iter())
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

    pub(crate) fn get_sorted_all_events(&self) -> Vec<&Event> {
        self.merge_events_each_track(|t| t.get_sorted_events())
    }

    pub(crate) fn get_sorted_all_events_in_ticks_range(
        &self,
        start_ticks: Ticks,
        end_ticks: Ticks,
    ) -> Vec<&Event> {
        self.merge_events_each_track(|t| t.get_sorted_events_in_ticks_range(start_ticks, end_ticks))
    }

    pub(crate) fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub(crate) fn remove_track(&mut self, track_id: Id) {
        if let Some(index) = self.tracks.iter().position(|t| t.get_id() == track_id) {
            self.tracks.remove(index);
        }
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
            &self
                .get_tracks()
                .iter()
                .map(|track| track.to_js_object())
                .collect::<js_sys::Array>(),
        )
        .unwrap();

        js_song
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{
        event::EventInput,
        note::{NoteInput, NoteNumber, Velocity},
    };

    use super::*;

    #[test]
    fn test_song() {
        let song = Song::new("test".to_string(), 480);
        assert_eq!(song.title, "test");
        assert_eq!(song.ppq, 480);
        assert_eq!(song.end_of_song, Ticks::new(0));
        assert_eq!(song.tracks.len(), 0);
    }

    #[test]
    fn test_get_tracks_ref() {
        let song = Song::new("test".to_string(), 480);
        assert_eq!(song.get_tracks().len(), 0);
    }

    #[test]
    fn test_get_track() {
        let mut song = Song::new("test".to_string(), 480);

        let track = Track::new();
        let track_id = track.get_id();
        song.tracks = vec![track];

        let got_track_id = song.get_track(track_id).unwrap().get_id();

        assert_eq!(got_track_id, track_id);
    }

    #[test]
    fn test_get_track_mut() {
        let mut song = Song::new("test".to_string(), 480);

        let track = Track::new();
        let track_id = track.get_id();
        song.tracks = vec![track];

        let got_track_id = song.get_track_mut(track_id).unwrap().get_id();

        assert_eq!(got_track_id, track_id);
    }

    #[test]
    fn test_get_sorted_all_events() {
        let mut song = Song::new("test".to_string(), 480);

        let mut track1 = Track::new();
        let mut track2 = Track::new();
        let mut track3 = Track::new();

        track1.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(480),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track1.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(240),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track2.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track2.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(960),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track3.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(720),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        song.tracks = vec![track1, track2, track3];

        let events = song.get_sorted_all_events();

        assert_eq!(events.len(), 5);
        assert_eq!(events[0].get_ticks().as_u32(), 0);
        assert_eq!(events[1].get_ticks().as_u32(), 240);
        assert_eq!(events[2].get_ticks().as_u32(), 480);
        assert_eq!(events[3].get_ticks().as_u32(), 720);
        assert_eq!(events[4].get_ticks().as_u32(), 960);
    }

    #[test]
    fn test_get_sorted_all_events_in_ticks_range() {
        let mut song = Song::new("test".to_string(), 480);

        let mut track1 = Track::new();
        let mut track2 = Track::new();
        let mut track3 = Track::new();

        track1.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(480),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track1.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(240),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track2.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(0),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track2.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(960),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        track3.add_event(EventInput::Note(NoteInput {
            ticks: Ticks::new(720),
            duration: Ticks::new(480),
            velocity: Velocity::new(100),
            note_number: NoteNumber::new(60),
        }));

        song.tracks = vec![track1, track2, track3];

        let events = song.get_sorted_all_events_in_ticks_range(Ticks::new(480), Ticks::new(960));

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].get_ticks().as_u32(), 240);
        assert_eq!(events[1].get_ticks().as_u32(), 480);
        assert_eq!(events[2].get_ticks().as_u32(), 720);
    }

    #[test]
    fn test_add_track() {
        let mut song = Song::new("test".to_string(), 480);

        let track1 = Track::new();
        let track2 = Track::new();

        song.add_track(track1);
        song.add_track(track2);

        assert_eq!(song.tracks.len(), 2);
    }

    #[test]
    fn test_remove_track() {
        let mut song = Song::new("test".to_string(), 480);

        let track1 = Track::new();
        let track_id1 = track1.get_id();
        let track2 = Track::new();
        song.tracks = vec![track1, track2];

        song.remove_track(track_id1);

        assert_eq!(song.tracks.len(), 1);
    }
}
