use crate::{
    shared::{id::Id, unit::time::Ticks},
    track::track::Track,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_SONG_CLASS: &'static str = r#"
export class Song {
  free(): void;

  constructor(title: string, ppq: number);

  getTrack(track_id: string): Track | undefined;

  getTracks(): Array<Track>;

  addEmptyTrack(): void;

  removeTrack(track_id: string): void;

  endOfSong: number;

  ppq: number;

  title: string;
}
"#;

#[wasm_bindgen(skip_typescript)]
pub struct Song {
    title: String,
    ppq: u32,
    end_of_song: Ticks,
    tracks: Vec<Track>,
}

#[wasm_bindgen]
impl Song {
    #[wasm_bindgen(constructor)]
    pub fn new(title: String, ppq: u32) -> Self {
        Song {
            title,
            ppq,
            end_of_song: Ticks::new(0),
            tracks: Vec::new(),
        }
    }

    #[wasm_bindgen(getter = title)]
    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(setter = title)]
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    #[wasm_bindgen(getter = ppq)]
    pub fn get_ppq(&self) -> u32 {
        self.ppq
    }

    #[wasm_bindgen(setter = ppq)]
    pub fn set_ppq(&mut self, ppq: u32) {
        self.ppq = ppq;
    }

    pub(crate) fn get_end_of_song(&self) -> Ticks {
        self.end_of_song
    }

    #[wasm_bindgen(getter = endOfSong)]
    pub fn get_end_of_song_js(&self) -> u32 {
        self.get_end_of_song().as_u32()
    }

    pub(crate) fn set_end_of_song(&mut self, end_of_song: Ticks) {
        self.end_of_song = end_of_song;
    }

    #[wasm_bindgen(setter = endOfSong)]
    pub fn set_end_of_song_js(&mut self, end_of_song: u32) {
        self.set_end_of_song(Ticks::new(end_of_song));
    }

    pub(crate) fn get_track(&self, track_id: Id) -> Option<&Track> {
        self.tracks.iter().find(|t| t.get_id() == track_id)
    }

    #[wasm_bindgen(js_name = getTrack)]
    pub fn get_track_js(&self, track_id: &str) -> Option<Track> {
        if let Ok(track_id) = Id::try_from(track_id) {
            if let Some(track) = self.get_track(track_id) {
                return Some(track.clone());
            }
        }
        None
    }

    #[wasm_bindgen(js_name = getTracks)]
    pub fn get_tracks_js(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.tracks).unwrap()
    }

    fn _add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    #[wasm_bindgen(js_name = addEmptyTrack)]
    pub fn add_empty_track(&mut self) {
        self._add_track(Track::new());
    }

    pub(crate) fn remove_track(&mut self, track_id: Id) {
        if let Some(index) = self.tracks.iter().position(|t| t.get_id() == track_id) {
            self.tracks.remove(index);
        }
    }

    #[wasm_bindgen(js_name = removeTrack)]
    pub fn remove_track_js(&mut self, track_id: &str) {
        if let Ok(track_id) = Id::try_from(track_id) {
            self.remove_track(track_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_song() {
        let title = "My Song";
        let ppq = 480;
        let song = Song::new(title.to_string(), ppq);

        assert_eq!(song.title, title);
        assert_eq!(song.ppq, ppq);
        assert_eq!(song.end_of_song.as_u32(), 0);
    }

    #[test]
    fn test_get_track() {
        let mut song = Song::new("My Song".to_string(), 480);

        song.add_empty_track();
        song.add_empty_track();

        let track_id = song.tracks[0].get_id();

        let track = song.get_track(track_id).unwrap();

        assert_eq!(track.get_id(), track_id);
    }

    #[test]
    fn test_add_empty_tracks() {
        let mut song = Song::new("My Song".to_string(), 480);

        song.add_empty_track();
        song.add_empty_track();

        assert_eq!(song.tracks.len(), 2);
    }

    #[test]
    fn test_remove_track() {
        let mut song = Song::new("My Song".to_string(), 480);

        song.add_empty_track();
        song.add_empty_track();

        let track_id = song.tracks[0].get_id();

        song.remove_track(track_id);

        assert_eq!(song.tracks.len(), 1);
    }
}
