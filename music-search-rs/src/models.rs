use serde::{Deserialize, Serialize};

/// Search source enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchSource {
    #[serde(rename = "NET_EASE_MUSIC")]
    NetEaseMusic,
    #[serde(rename = "QQ_MUSIC")]
    QQMusic,
}

/// Search type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchType {
    #[serde(rename = "SONG_ID")]
    SongId,
    #[serde(rename = "ALBUM_ID")]
    AlbumId,
    #[serde(rename = "PLAYLIST_ID")]
    PlaylistId,
}

/// Generic result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultVo<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error_msg: Option<String>,
}

impl<T> ResultVo<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error_msg: None,
        }
    }

    pub fn failure(error_msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error_msg: Some(error_msg),
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// Song information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongVo {
    pub id: String,
    pub display_id: String,
    pub pics: String,
    pub name: String,
    pub singer: Vec<String>,
    pub album: String,
    /// Duration in milliseconds
    pub duration: i64,
}

/// Simple song information (for playlists/albums)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleSongVo {
    pub id: String,
    pub display_id: String,
    pub name: String,
    pub singer: Vec<String>,
}

/// Lyric information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricVo {
    pub search_source: SearchSource,
    pub lyric: Option<String>,
    pub translate_lyric: Option<String>,
    pub transliteration_lyric: Option<String>,
}

/// Playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistVo {
    pub name: String,
    pub author_name: String,
    pub description: Option<String>,
    pub simple_song_vos: Vec<SimpleSongVo>,
}

/// Album information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumVo {
    pub name: String,
    pub company: Option<String>,
    pub desc: Option<String>,
    pub simple_song_vos: Vec<SimpleSongVo>,
    pub time_public: Option<String>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultVo {
    pub search_type: SearchType,
    pub search_source: SearchSource,
    pub song_vos: Vec<SongSearchResultVo>,
    pub album_vos: Vec<AlbumSearchResultVo>,
    pub playlist_vos: Vec<PlaylistSearchResultVo>,
}

impl SearchResultVo {
    pub fn new(search_type: SearchType, search_source: SearchSource) -> Self {
        Self {
            search_type,
            search_source,
            song_vos: Vec::new(),
            album_vos: Vec::new(),
            playlist_vos: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.song_vos.is_empty() && self.album_vos.is_empty() && self.playlist_vos.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongSearchResultVo {
    pub display_id: String,
    pub title: String,
    pub author_name: Vec<String>,
    pub album_name: String,
    /// Duration in milliseconds
    pub duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSearchResultVo {
    pub display_id: String,
    pub album_name: String,
    pub author_name: Vec<String>,
    pub song_count: i64,
    pub publish_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistSearchResultVo {
    pub display_id: String,
    pub playlist_name: String,
    pub author_name: String,
    pub description: Option<String>,
    pub play_count: i64,
    pub song_count: i64,
}

/// Error messages constants
pub mod error_msg {
    pub const SEARCH_RESULT_EMPTY: &str = "查询结果为空，请修改查询条件";
    pub const PLAYLIST_NOT_EXIST: &str = "歌单信息暂未被收录或查询失败";
    pub const ALBUM_NOT_EXIST: &str = "专辑信息暂未被收录或查询失败";
    pub const SONG_NOT_EXIST: &str = "歌曲信息暂未被收录或查询失败";
    pub const LRC_NOT_EXIST: &str = "歌词信息暂未被收录或查询失败";
    pub const SONG_URL_GET_FAILED: &str = "歌曲直链，获取失败";
    pub const NEED_LOGIN: &str = "本请求需要登陆信息才可使用，请检查 Cookie 是否填写或过期";
    pub const NETWORK_ERROR: &str = "网络错误，请检查网络链接";
    pub const SYSTEM_ERROR: &str = "系统错误";
}
