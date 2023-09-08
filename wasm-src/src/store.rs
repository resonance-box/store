use crate::{
    event::event::{Event, EventUpdater},
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

  createSong(title: string, ppq: number): void;

  clearSong(): void;

  getTrack(trackId: string): Track | undefined;

  getTracks(): Track[];

  addTrack(track: Track): Track;

  removeTrack(trackId: string): void;

  getEvent(eventId: string): Event | undefined;

  getEvents(): Event[];

  getEventsInTicksRange(startTicks: number, endTicks: number, withinDuration: boolean): Event[];

  addEvent(event: Event): Event;

  updateEvent(event: EventUpdater): Event;

  removeEvent(eventId: string): void;
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

    #[wasm_bindgen(js_name = createSong)]
    pub fn create_song_js(&mut self, title: String, ppq: u32) {
        self.song = Some(Song::new(title, ppq));
    }

    #[wasm_bindgen(js_name = clearSong)]
    pub fn clear_song_js(&mut self) {
        self.song = None;
    }

    #[wasm_bindgen(js_name = getTrack)]
    pub fn get_track_js(&self, track_id: &str) -> Option<js_sys::Object> {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let track = song.get_track(&track_id);
        track.map(|track| track.to_js_object())
    }

    #[wasm_bindgen(js_name = getTracks)]
    pub fn get_tracks_js(&self) -> js_sys::Array {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let tracks = song.get_tracks();
        tracks.to_js_array()
    }

    #[wasm_bindgen(js_name = addTrack)]
    pub fn add_track_js(&mut self, track: js_sys::Object) -> js_sys::Object {
        let song = self.song.as_mut().expect_throw("Song is not set");
        let track = Track::from_js_object(track);
        song.add_track(track).to_js_object()
    }

    #[wasm_bindgen(js_name = removeTrack)]
    pub fn remove_track_js(&mut self, track_id: &str) {
        let track_id = Id::try_from(track_id).expect_throw("Track id is not valid");
        let song = self.song.as_mut().expect_throw("Song is not set");
        song.remove_track(&track_id);
    }

    #[wasm_bindgen(js_name = getEvent)]
    pub fn get_event_js(&self, event_id: &str) -> Option<js_sys::Object> {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let event_id = Id::try_from(event_id).expect_throw("Event id is not valid");
        let event = song.get_event(&event_id);
        event.map(|event| event.to_js_object())
    }

    #[wasm_bindgen(js_name = getEvents)]
    pub fn get_events_js(&self) -> js_sys::Array {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let events = song.get_events(None); // TODO: None
        events.iter().map(|event| event.to_js_object()).collect()
    }

    #[wasm_bindgen(js_name = getEventsInTicksRange)]
    pub fn get_events_in_ticks_range_js(
        &self,
        start_ticks: u32,
        end_ticks: u32,
        within_duration: bool,
    ) -> js_sys::Array {
        let song = self.song.as_ref().expect_throw("Song is not set");
        let events = song.get_events_in_ticks_range(
            Ticks::new(start_ticks),
            Ticks::new(end_ticks),
            within_duration,
            None, // TODO: None
        );
        events.iter().map(|event| event.to_js_object()).collect()
    }

    #[wasm_bindgen(js_name = addEvent)]
    pub fn add_event_js(&mut self, event: js_sys::Object) -> js_sys::Object {
        let song = self.song.as_mut().expect_throw("Song is not set");
        let event = Event::from_js_object(event);
        let event = song.add_event(event);
        event.to_js_object()
    }

    #[wasm_bindgen(js_name = updateEvent)]
    pub fn update_event_js(&mut self, event: js_sys::Object) -> js_sys::Object {
        let song = self.song.as_mut().expect_throw("Song is not set");
        let event = EventUpdater::from_js_object(event);
        let event = song.update_event(event);
        event.to_js_object()
    }

    #[wasm_bindgen(js_name = removeEvent)]
    pub fn remove_event_js(&mut self, event_id: &str) {
        let song = self.song.as_mut().expect_throw("Song is not set");
        let event_id = Id::try_from(event_id).expect_throw("Event id is not valid");
        song.remove_event(&event_id);
    }
}
