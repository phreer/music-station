use serde::{Deserialize, Serialize};
use crate::models::*;

/// Custom deserializer to convert number to string
fn deserialize_number_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct NumberToStringVisitor;

    impl<'de> Visitor<'de> for NumberToStringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }
    }

    deserializer.deserialize_any(NumberToStringVisitor)
}

/// NetEase Music search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    // SearchType = SONG
    #[serde(default)]
    pub songs: Vec<Song>,
    #[serde(rename = "songCount", default)]
    pub song_count: i64,

    // SearchType = ALBUM
    #[serde(default)]
    pub albums: Vec<Album>,
    #[serde(rename = "albumCount", default)]
    pub album_count: i64,

    // SearchType = PLAYLIST
    #[serde(default)]
    pub playlists: Vec<SimplePlaylist>,
    #[serde(rename = "playlistCount", default)]
    pub playlist_count: i64,
}

impl SearchResult {
    pub fn convert(&self, search_type: SearchType) -> SearchResultVo {
        let mut vo = SearchResultVo::new(search_type, SearchSource::NetEaseMusic);

        match search_type {
            SearchType::SongId => {
                if self.song_count > 0 {
                    for song in &self.songs {
                        vo.song_vos.push(SongSearchResultVo {
                            display_id: song.id.clone(),
                            title: song.name.clone(),
                            author_name: song.ar.iter().map(|a| a.name.clone()).collect(),
                            album_name: song.al.name.clone(),
                            duration: song.dt,
                        });
                    }
                }
            }
            SearchType::AlbumId => {
                if self.album_count > 0 {
                    for album in &self.albums {
                        vo.album_vos.push(AlbumSearchResultVo {
                            display_id: album.id.to_string(),
                            album_name: album.name.clone(),
                            author_name: album.artists.iter().map(|a| a.name.clone()).collect(),
                            song_count: album.size,
                            publish_time: Some(format_date(album.publish_time)),
                        });
                    }
                }
            }
            SearchType::PlaylistId => {
                if self.playlist_count > 0 {
                    for playlist in &self.playlists {
                        vo.playlist_vos.push(PlaylistSearchResultVo {
                            display_id: playlist.id.clone(),
                            playlist_name: playlist.name.clone(),
                            author_name: playlist.creator.nickname.clone(),
                            description: Some(playlist.description.clone().unwrap_or_default()),
                            play_count: playlist.play_count,
                            song_count: playlist.track_count,
                        });
                    }
                }
            }
        }

        vo
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    #[serde(deserialize_with = "deserialize_number_to_string")]
    pub id: String,
    pub name: String,
    pub ar: Vec<Artist>,
    pub al: Album2,
    /// Duration in milliseconds
    pub dt: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album2 {
    pub id: i64,
    pub name: String,
    #[serde(rename = "picUrl")]
    pub pic_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: i64,
    pub name: String,
    pub artists: Vec<Artist>,
    pub size: i64,
    #[serde(rename = "publishTime")]
    pub publish_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePlaylist {
    pub id: String,
    pub name: String,
    pub creator: Creator,
    pub description: Option<String>,
    #[serde(rename = "playCount")]
    pub play_count: i64,
    #[serde(rename = "trackCount")]
    pub track_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creator {
    #[serde(rename = "userId")]
    pub user_id: i64,
    pub nickname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTrack {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub creator: Creator,
    pub description: Option<String>,
    #[serde(rename = "trackIds")]
    pub track_ids: Vec<SimpleTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistResult {
    pub code: i32,
    pub playlist: Playlist,
}

impl PlaylistResult {
    pub fn convert(&self, simple_song_vos: Vec<SimpleSongVo>) -> PlaylistVo {
        PlaylistVo {
            name: self.playlist.name.clone(),
            author_name: self.playlist.creator.nickname.clone(),
            description: self.playlist.description.clone(),
            simple_song_vos,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumResult {
    pub code: i32,
    pub album: AlbumInfo,
    pub songs: Vec<Song>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumInfo {
    pub name: String,
    pub company: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "publishTime")]
    pub publish_time: i64,
}

impl AlbumResult {
    pub fn convert(&self) -> AlbumVo {
        AlbumVo {
            name: self.album.name.clone(),
            company: self.album.company.clone(),
            desc: self.album.description.clone(),
            simple_song_vos: self.songs.iter().map(|s| SimpleSongVo {
                id: s.id.clone(),
                display_id: s.id.clone(),
                name: s.name.clone(),
                singer: s.ar.iter().map(|a| a.name.clone()).collect(),
            }).collect(),
            time_public: Some(format_date(self.album.publish_time)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailResult {
    pub code: i32,
    pub songs: Vec<Song>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongUrls {
    pub code: i32,
    pub data: Vec<Datum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datum {
    pub id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricResult {
    pub code: i32,
    pub lrc: Option<Lrc>,
    pub tlyric: Option<Lrc>,
    pub romalrc: Option<Lrc>,
    pub yrc: Option<Lrc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lrc {
    pub lyric: String,
}

fn format_date(timestamp: i64) -> String {
    // Convert timestamp (ms) to readable date
    use std::time::{UNIX_EPOCH, Duration};
    let d = UNIX_EPOCH + Duration::from_millis(timestamp as u64);
    format!("{:?}", d) // Simple formatting, can be improved with chrono
}
