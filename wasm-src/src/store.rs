use crate::{
    event::event::{EventInput, EventUpdater},
    shared::{id::Id, unit::time::Ticks},
    song::song::Song,
    track::track::Track,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Store {
    song: Option<Song>,
}

#[wasm_bindgen]
impl Store {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Store { song: None }
    }

    #[wasm_bindgen(js_name = getSong)]
    pub fn get_song_js(&self) -> Option<Song> {
        self.song.clone()
    }

    #[wasm_bindgen(js_name = setSong)]
    pub fn set_song_js(&mut self, song: Song) {
        self.song = Some(song);
    }

    #[wasm_bindgen(js_name = createSong)]
    pub fn create_song_js(&mut self, title: String, ppq: u32) {
        self.song = Some(Song::new(title, ppq));
    }

    #[wasm_bindgen(js_name = clearSong)]
    pub fn clear_song_js(&mut self) {
        self.song = None;
    }

    fn get_track(&self, track_id: Id) -> Option<&Track> {
        let song = self.song.as_ref().expect_throw("Song is not set");
        if let Some(track) = song.get_track(track_id) {
            return Some(track);
        }
        None
    }

    fn get_track_mut(&mut self, track_id: Id) -> Option<&mut Track> {
        let song = self.song.as_mut().expect_throw("Song is not set");
        if let Some(track) = song.get_track_mut(track_id) {
            return Some(track);
        }
        None
    }

    #[wasm_bindgen(js_name = getTrack)]
    pub fn get_track_js(&self, track_id: &str) -> Option<Track> {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        if let Some(track) = self.get_track(track_id) {
            return Some(track.clone());
        }
        None
    }

    #[wasm_bindgen(js_name = getTracks)]
    pub fn get_tracks_js(&self) -> JsValue {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let tracks = song.get_tracks_ref();
        serde_wasm_bindgen::to_value(tracks).unwrap()
    }

    #[wasm_bindgen(js_name = addEmptyTrack)]
    pub fn add_empty_track_js(&mut self) {
        let song = self.song.as_mut().expect_throw("Song is not set");
        song.add_track(Track::new());
    }

    #[wasm_bindgen(js_name = removeTrack)]
    pub fn remove_track_js(&mut self, track_id: &str) {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let song = self.song.as_mut().expect_throw("Song is not set");
        song.remove_track(track_id);
    }

    #[wasm_bindgen(js_name = getEvent)]
    pub fn get_event_js(&self, track_id: &str, event_id: &str) -> Option<js_sys::Object> {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = self.get_track(track_id).expect_throw("Track not found");
        if let Ok(event_id) = Id::try_from(event_id) {
            if let Some(event) = track.get_event(event_id) {
                Some(event.to_js_object())
            } else {
                None
            }
        } else {
            None
        }
    }

    #[wasm_bindgen(js_name = getSortedEvents)]
    pub fn get_sorted_events_js(&self, track_id: &str) -> js_sys::Array {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = self.get_track(track_id).expect_throw("Track not found");
        track
            .get_sorted_events()
            .iter()
            .map(|event| event.to_js_object())
            .collect()
    }

    #[wasm_bindgen(js_name = getSortedEventsInTicksRange)]
    pub fn get_sorted_events_in_ticks_range_js(
        &self,
        track_id: &str,
        start_ticks: u32,
        end_ticks: u32,
    ) -> js_sys::Array {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = self.get_track(track_id).expect_throw("Track not found");
        track
            .get_sorted_events_in_ticks_range(Ticks::new(start_ticks), Ticks::new(end_ticks))
            .iter()
            .map(|event| event.to_js_object())
            .collect()
    }

    #[wasm_bindgen(js_name = addEvent)]
    pub fn add_event_js(&mut self, track_id: &str, event: js_sys::Object) {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = self.get_track_mut(track_id).expect_throw("Track not found");
        let event = EventInput::from_js_object(event);
        track.add_event(event);
    }

    #[wasm_bindgen(js_name = updateEvent)]
    pub fn update_event_js(&mut self, track_id: &str, event: js_sys::Object) {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = self.get_track_mut(track_id).expect_throw("Track not found");
        let event = EventUpdater::from_js_object(event);
        track.update_event(event);
    }

    #[wasm_bindgen(js_name = removeEvent)]
    pub fn remove_event_js(&mut self, track_id: &str, event_id: &str) {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = self.get_track_mut(track_id).expect_throw("Track not found");
        if let Ok(event_id) = Id::try_from(event_id) {
            track.remove_event(event_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_store() -> Store {
        Store::new()
    }

    #[test]
    fn test_create_song() {
        let mut store = new_store();

        let title = "My Song";
        let ppq = 480;

        store.create_song_js(title.to_string(), ppq);
        let song = store.song.unwrap();

        assert_eq!(song.title, title);
        assert_eq!(song.ppq, ppq);
        assert_eq!(song.end_of_song.as_u32(), 0);
    }

    #[test]
    fn test_get_song_js() {
        let mut store = new_store();

        let title = "My Song";
        let ppq = 480;

        store.create_song_js(title.to_string(), ppq);
        let song = store.get_song_js().unwrap();

        assert_eq!(song.title, title);
        assert_eq!(song.ppq, ppq);
        assert_eq!(song.end_of_song.as_u32(), 0);
    }

    #[test]
    fn test_set_song_js() {
        let mut store = new_store();

        let title = "My Song";
        let ppq = 480;

        store.create_song_js(title.to_string(), ppq);
        let mut song = store.song.clone().unwrap();

        let title = "My New Song";
        let ppq = 240;
        song.title = title.to_string();
        song.ppq = ppq;

        store.set_song_js(song);

        let song = store.song.unwrap();

        assert_eq!(song.title, title);
        assert_eq!(song.ppq, ppq);
        assert_eq!(song.end_of_song.as_u32(), 0);
    }

    #[test]
    fn test_clear_song_js() {
        let mut store = new_store();

        let title = "My Song";
        let ppq = 480;

        store.create_song_js(title.to_string(), ppq);
        store.clear_song_js();

        assert!(store.song.is_none());
    }
}
