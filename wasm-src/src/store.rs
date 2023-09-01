use crate::{
    event::event::{EventInput, EventUpdater},
    shared::{id::Id, unit::time::Ticks},
    song::song::Song,
    track::track::Track,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_STORE_CLASS: &'static str = r#"
export class Store {
  free(): void;

  constructor();

  getSong(): Song | undefined;

  setSong(song: Song): void;

  createSong(title: string, ppq: number): void;

  clearSong(): void;

  getTrack(trackId: string): Track | undefined;

  getTracks(): Array<Track>;

  getSortedAllEvents(): Array<Event>;

  getSortedAllEventsInTicksRange(startTicks: number, endTicks: number): Array<Event>;

  addEmptyTrack(): void;

  removeTrack(trackId: string): void;

  getEvent(trackId: string, eventId: string): Event | undefined;

  getSortedEvents(trackId: string): Array<Event>;

  getSortedEventsInTicksRange(trackId: string, startTicks: number, endTicks: number): Array<Event>;

  addEvent(trackId: string, event: EventInput): void;

  updateEvent(trackId: string, event: EventUpdater): void;

  removeEvent(trackId: string, eventId: string): void;
}
"#;

#[wasm_bindgen(skip_typescript)]
pub struct Store {
    song: Option<Song>,
}

#[wasm_bindgen]
impl Store {
    pub(crate) fn new() -> Self {
        Store { song: None }
    }

    #[wasm_bindgen(constructor)]
    pub fn new_js() -> Self {
        Self::new()
    }

    #[wasm_bindgen(js_name = getSong)]
    pub fn get_song_js(&self) -> Option<js_sys::Object> {
        self.song.as_ref().map(|song| song.to_js_object())
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
        song.get_track(track_id)
    }

    fn get_track_mut(&mut self, track_id: Id) -> Option<&mut Track> {
        let song = self.song.as_mut().expect_throw("Song is not set");
        song.get_track_mut(track_id)
    }

    #[wasm_bindgen(js_name = getTrack)]
    pub fn get_track_js(&self, track_id: &str) -> Option<js_sys::Object> {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        self.get_track(track_id).map(|track| track.to_js_object())
    }

    #[wasm_bindgen(js_name = getTracks)]
    pub fn get_tracks_js(&self) -> js_sys::Array {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let tracks = song.get_tracks();
        tracks.iter().map(|track| track.to_js_object()).collect()
    }

    #[wasm_bindgen(js_name = getSortedAllEvents)]
    pub fn get_sorted_all_events_js(&self) -> js_sys::Array {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let events = song.get_sorted_all_events();
        events.iter().map(|event| event.to_js_object()).collect()
    }

    #[wasm_bindgen(js_name = getSortedAllEventsInTicksRange)]
    pub fn get_sorted_all_events_in_ticks_range_js(
        &self,
        start_ticks: u32,
        end_ticks: u32,
        within_duration: bool,
    ) -> js_sys::Array {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let events = song.get_sorted_all_events_in_ticks_range(
            Ticks::new(start_ticks),
            Ticks::new(end_ticks),
            within_duration,
        );
        events.iter().map(|event| event.to_js_object()).collect()
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
        within_duration: bool,
    ) -> js_sys::Array {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = self.get_track(track_id).expect_throw("Track not found");
        track
            .get_sorted_events_in_ticks_range(
                Ticks::new(start_ticks),
                Ticks::new(end_ticks),
                within_duration,
            )
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
