use serde::{Deserialize, Serialize};
use crate::models::*;

/// QQ Music search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicFcgApiResult {
    pub code: i32,
    #[serde(rename = "req_1")]
    pub req_1: MusicFcgReq1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicFcgReq1 {
    pub code: i32,
    pub data: MusicFcgReq1Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicFcgReq1Data {
    pub code: i32,
    pub body: MusicFcgReq1DataBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicFcgReq1DataBody {
    #[serde(default)]
    pub album: Option<AlbumBody>,
    #[serde(default)]
    pub song: Option<SongBody>,
    #[serde(default)]
    pub songlist: Option<PlaylistBody>,
}

impl MusicFcgReq1DataBody {
    pub fn convert(&self, search_type: SearchType) -> SearchResultVo {
        let mut vo = SearchResultVo::new(search_type, SearchSource::QQMusic);

        match search_type {
            SearchType::SongId => {
                if let Some(song_body) = &self.song {
                    for song in &song_body.list {
                        vo.song_vos.push(SongSearchResultVo {
                            display_id: song.id.clone(),
                            title: song.title.clone().unwrap_or_else(|| song.name.clone()),
                            author_name: song.singer.iter().map(|s| s.name.clone()).collect(),
                            album_name: song.album.name.clone(),
                            duration: song.interval * 1000,
                        });
                    }
                }
            }
            SearchType::AlbumId => {
                if let Some(album_body) = &self.album {
                    for album in &album_body.list {
                        vo.album_vos.push(AlbumSearchResultVo {
                            display_id: album.album_mid.clone(),
                            album_name: album.album_name.clone(),
                            author_name: album.singer_list.iter().map(|s| s.name.clone()).collect(),
                            song_count: album.song_count,
                            publish_time: Some(album.public_time.clone()),
                        });
                    }
                }
            }
            SearchType::PlaylistId => {
                if let Some(playlist_body) = &self.songlist {
                    for playlist in &playlist_body.list {
                        vo.playlist_vos.push(PlaylistSearchResultVo {
                            display_id: playlist.dissid.clone(),
                            playlist_name: playlist.dissname.clone(),
                            author_name: playlist.creator.name.clone(),
                            description: Some(playlist.introduction.clone()),
                            play_count: playlist.listennum,
                            song_count: playlist.song_count,
                        });
                    }
                }
            }
        }

        vo
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumBody {
    pub list: Vec<AlbumInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumInfo {
    #[serde(rename = "albumID")]
    pub album_id: i64,
    #[serde(rename = "albumMID")]
    pub album_mid: String,
    #[serde(rename = "albumName")]
    pub album_name: String,
    #[serde(rename = "song_count")]
    pub song_count: i64,
    #[serde(rename = "publicTime")]
    pub public_time: String,
    #[serde(rename = "singer_list")]
    pub singer_list: Vec<Singer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongBody {
    pub list: Vec<Song>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistBody {
    pub list: Vec<PlaylistInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistInfo {
    pub dissid: String,
    pub dissname: String,
    pub introduction: String,
    #[serde(rename = "song_count", alias = "song_Count")]
    pub song_count: i64,
    pub listennum: i64,
    pub creator: PlaylistCreator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistCreator {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    #[serde(deserialize_with = "deserialize_number_to_string")]
    pub id: String,
    pub mid: String,
    pub name: String,
    pub title: Option<String>,
    pub interval: i64,
    pub album: SongAlbum,
    pub singer: Vec<Singer>,
}

impl Song {
    pub fn convert_simple(&self) -> SimpleSongVo {
        SimpleSongVo {
            id: self.id.clone(),
            display_id: self.mid.clone(),
            name: self.name.clone(),
            singer: self.singer.iter().map(|s| s.name.clone()).collect(),
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongAlbum {
    pub id: i64,
    pub mid: String,
    pub pmid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Singer {
    pub id: i64,
    pub mid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongResult {
    pub code: i32,
    pub data: Vec<Song>,
}

impl SongResult {
    pub fn is_illegal(&self) -> bool {
        self.code != 0 || self.data.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistResult {
    pub code: i32,
    pub cdlist: Vec<Playlist>,
}

impl PlaylistResult {
    pub fn convert(&self) -> PlaylistVo {
        let playlist = &self.cdlist[0];
        PlaylistVo {
            name: playlist.dissname.clone(),
            author_name: playlist.nickname.clone(),
            description: Some(playlist.desc.clone()),
            simple_song_vos: playlist.song_list.iter().map(|s| s.convert_simple()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub dissname: String,
    pub nickname: String,
    pub desc: String,
    #[serde(rename = "songList")]
    pub song_list: Vec<Song>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumResult {
    pub code: i32,
    pub data: AlbumData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumData {
    #[serde(rename = "aDate")]
    pub a_date: String,
    pub company: Option<String>,
    pub desc: Option<String>,
    pub name: String,
    pub list: Vec<AlbumSong>,
}

impl AlbumResult {
    pub fn convert(&self) -> AlbumVo {
        AlbumVo {
            name: self.data.name.clone(),
            company: self.data.company.clone(),
            desc: self.data.desc.clone(),
            simple_song_vos: self.data.list.iter().map(|s| s.convert_simple()).collect(),
            time_public: Some(self.data.a_date.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSong {
    pub songid: i64,
    pub songmid: String,
    pub songname: String,
    pub singer: Vec<Singer>,
}

impl AlbumSong {
    pub fn convert_simple(&self) -> SimpleSongVo {
        SimpleSongVo {
            id: self.songid.to_string(),
            display_id: self.songmid.clone(),
            name: self.songname.clone(),
            singer: self.singer.iter().map(|s| s.name.clone()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricResult {
    pub code: i32,
    #[serde(default)]
    pub lyric: String,
    #[serde(default)]
    pub trans: String,
    #[serde(default)]
    pub roma: String,
}
