use crate::{
    shared::{id::Id, unit::time::Ticks},
    track::track::Track,
};
use wasm_bindgen::prelude::*;

// #[wasm_bindgen(typescript_custom_section)]
// const TS_SONG_CLASS: &'static str = r#"
// export class Song {
//   free(): void;

//   constructor(title: string, ppq: number);

//   getTrack(track_id: string): Track | undefined;

//   getTracks(): Array<Track>;

//   addEmptyTrack(): void;

//   removeTrack(track_id: string): void;

//   endOfSong: number;

//   ppq: number;

//   title: string;
// }
// "#;

// #[wasm_bindgen(skip_typescript)]
#[wasm_bindgen]
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
    pub(crate) fn get_tracks_ref(&self) -> &Vec<Track> {
        &self.tracks
    }

    pub(crate) fn get_track(&self, track_id: Id) -> Option<&Track> {
        self.tracks.iter().find(|t| t.get_id() == track_id)
    }

    pub(crate) fn get_track_mut(&mut self, track_id: Id) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.get_id() == track_id)
    }

    pub(crate) fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub(crate) fn remove_track(&mut self, track_id: Id) {
        if let Some(index) = self.tracks.iter().position(|t| t.get_id() == track_id) {
            self.tracks.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(song.get_tracks_ref().len(), 0);
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
