pub mod error;
pub mod models;
pub mod netease;
pub mod qqmusic;

use async_trait::async_trait;
pub use error::{MusicSearchError, Result};
pub use models::*;
pub use netease::NetEaseMusicApi;
pub use qqmusic::QQMusicApi;
use std::collections::HashMap;

/// Unified Music API trait for search services
#[async_trait]
pub trait MusicApi: Send + Sync {
    /// Get the search source
    fn source(&self) -> SearchSource;

    /// Search for songs, albums, or playlists
    async fn search(&self, keyword: &str, search_type: SearchType) -> Result<ResultVo<SearchResultVo>>;

    /// Get playlist information
    async fn get_playlist(&self, playlist_id: &str) -> Result<ResultVo<PlaylistVo>>;

    /// Get album information
    async fn get_album(&self, album_id: &str) -> Result<ResultVo<AlbumVo>>;

    /// Get multiple songs information
    async fn get_songs(&self, song_ids: &[String]) -> Result<HashMap<String, ResultVo<SongVo>>>;

    /// Get song link/URL
    async fn get_song_link(&self, song_id: &str) -> Result<ResultVo<String>>;

    /// Get lyric information
    async fn get_lyric(&self, id: &str, display_id: &str, is_verbatim: bool) -> Result<ResultVo<LyricVo>>;
}

/// Implementation of MusicApi for NetEase Music
#[async_trait]
impl MusicApi for NetEaseMusicApi {
    fn source(&self) -> SearchSource {
        SearchSource::NetEaseMusic
    }

    async fn search(&self, keyword: &str, search_type: SearchType) -> Result<ResultVo<SearchResultVo>> {
        self.search(keyword, search_type).await
    }

    async fn get_playlist(&self, playlist_id: &str) -> Result<ResultVo<PlaylistVo>> {
        let result = self.get_playlist(playlist_id).await?;
        
        if result.code == 200 {
            let song_ids: Vec<String> = result.playlist.track_ids
                .iter()
                .map(|t| t.id.to_string())
                .collect();
            
            let songs = self.get_songs(&song_ids).await?;
            let simple_songs: Vec<SimpleSongVo> = song_ids
                .iter()
                .filter_map(|id| {
                    songs.get(id).map(|s| SimpleSongVo {
                        id: s.id.clone(),
                        display_id: s.id.clone(),
                        name: s.name.clone(),
                        singer: s.ar.iter().map(|a| a.name.clone()).collect(),
                    })
                })
                .collect();
            
            Ok(ResultVo::success(result.convert(simple_songs)))
        } else if result.code == 20001 {
            Ok(ResultVo::failure(error_msg::NEED_LOGIN.to_string()))
        } else {
            Ok(ResultVo::failure(error_msg::PLAYLIST_NOT_EXIST.to_string()))
        }
    }

    async fn get_album(&self, album_id: &str) -> Result<ResultVo<AlbumVo>> {
        let result = self.get_album(album_id).await?;
        
        if result.code == 200 {
            Ok(ResultVo::success(result.convert()))
        } else {
            Ok(ResultVo::failure(error_msg::ALBUM_NOT_EXIST.to_string()))
        }
    }

    async fn get_songs(&self, song_ids: &[String]) -> Result<HashMap<String, ResultVo<SongVo>>> {
        let songs_map = self.get_songs(song_ids).await?;
        
        let mut result = HashMap::new();
        for song_id in song_ids {
            if let Some(song) = songs_map.get(song_id) {
                result.insert(
                    song_id.clone(),
                    ResultVo::success(SongVo {
                        id: song.id.clone(),
                        display_id: song_id.clone(),
                        pics: song.al.pic_url.clone(),
                        name: song.name.clone(),
                        singer: song.ar.iter().map(|a| a.name.clone()).collect(),
                        album: song.al.name.clone(),
                        duration: song.dt,
                    }),
                );
            } else {
                result.insert(
                    song_id.clone(),
                    ResultVo::failure(error_msg::SONG_NOT_EXIST.to_string()),
                );
            }
        }
        
        Ok(result)
    }

    async fn get_song_link(&self, song_id: &str) -> Result<ResultVo<String>> {
        let datum_map = self.get_song_url(&[song_id.to_string()]).await?;
        
        if let Some(datum) = datum_map.get(song_id) {
            if let Some(url) = &datum.url {
                return Ok(ResultVo::success(url.clone()));
            }
        }
        
        Ok(ResultVo::failure(error_msg::SONG_URL_GET_FAILED.to_string()))
    }

    async fn get_lyric(&self, _id: &str, display_id: &str, _is_verbatim: bool) -> Result<ResultVo<LyricVo>> {
        let result = self.get_lyric(display_id).await?;
        
        if result.code != 200 {
            return Ok(ResultVo::failure(error_msg::LRC_NOT_EXIST.to_string()));
        }
        
        let vo = LyricVo {
            search_source: SearchSource::NetEaseMusic,
            lyric: result.lrc.map(|l| l.lyric),
            translate_lyric: result.tlyric.map(|l| l.lyric),
            transliteration_lyric: result.romalrc.map(|l| l.lyric),
        };
        
        Ok(ResultVo::success(vo))
    }
}

/// Implementation of MusicApi for QQ Music
#[async_trait]
impl MusicApi for QQMusicApi {
    fn source(&self) -> SearchSource {
        SearchSource::QQMusic
    }

    async fn search(&self, keyword: &str, search_type: SearchType) -> Result<ResultVo<SearchResultVo>> {
        self.search(keyword, search_type).await
    }

    async fn get_playlist(&self, playlist_id: &str) -> Result<ResultVo<PlaylistVo>> {
        let result = self.get_playlist(playlist_id).await?;
        
        if result.code == 0 {
            Ok(ResultVo::success(result.convert()))
        } else {
            Ok(ResultVo::failure(error_msg::PLAYLIST_NOT_EXIST.to_string()))
        }
    }

    async fn get_album(&self, album_id: &str) -> Result<ResultVo<AlbumVo>> {
        let result = self.get_album(album_id).await?;
        
        if result.code == 0 {
            Ok(ResultVo::success(result.convert()))
        } else {
            Ok(ResultVo::failure(error_msg::ALBUM_NOT_EXIST.to_string()))
        }
    }

    async fn get_songs(&self, song_ids: &[String]) -> Result<HashMap<String, ResultVo<SongVo>>> {
        let mut result = HashMap::new();
        
        for song_id in song_ids {
            let song_result = self.get_song(song_id).await?;
            
            if song_result.is_illegal() {
                result.insert(
                    song_id.clone(),
                    ResultVo::failure(error_msg::SONG_NOT_EXIST.to_string()),
                );
            } else {
                let song = &song_result.data[0];
                result.insert(
                    song_id.clone(),
                    ResultVo::success(SongVo {
                        id: song.id.clone(),
                        display_id: song.mid.clone(),
                        pics: format!("https://y.qq.com/music/photo_new/T002R800x800M000{}.jpg", song.album.pmid),
                        name: song.title.clone().unwrap_or_else(|| song.name.clone()),
                        singer: song.singer.iter().map(|s| s.name.clone()).collect(),
                        album: song.album.name.clone(),
                        duration: song.interval * 1000,
                    }),
                );
            }
        }
        
        Ok(result)
    }

    async fn get_song_link(&self, song_id: &str) -> Result<ResultVo<String>> {
        self.get_song_link(song_id).await
    }

    async fn get_lyric(&self, id: &str, _display_id: &str, _is_verbatim: bool) -> Result<ResultVo<LyricVo>> {
        let result = self.get_lyric(id).await?;
        
        if result.code != 0 {
            return Ok(ResultVo::failure(error_msg::LRC_NOT_EXIST.to_string()));
        }
        
        let vo = LyricVo {
            search_source: SearchSource::QQMusic,
            lyric: Some(result.lyric).filter(|s| !s.is_empty()),
            translate_lyric: Some(result.trans).filter(|s| !s.is_empty()),
            transliteration_lyric: Some(result.roma).filter(|s| !s.is_empty()),
        };
        
        Ok(ResultVo::success(vo))
    }
}
