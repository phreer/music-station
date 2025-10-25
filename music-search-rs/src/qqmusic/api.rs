use crate::error::{MusicSearchError, Result};
use crate::models::*;
use crate::qqmusic::decrypt::decrypt_lyrics;
use crate::qqmusic::models::*;
use rand::Rng;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

pub struct QQMusicApi {
    client: Client,
    cookie: Option<String>,
}

impl QQMusicApi {
    pub fn new(cookie: Option<String>) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()?,
            cookie,
        })
    }

    /// Search for songs, albums, or playlists
    pub async fn search(&self, keyword: &str, search_type: SearchType) -> Result<ResultVo<SearchResultVo>> {
        // 0: song, 2: album, 3: playlist
        let type_code = match search_type {
            SearchType::SongId => 0,
            SearchType::AlbumId => 2,
            SearchType::PlaylistId => 3,
        };

        let data = json!({
            "req_1": {
                "method": "DoSearchForQQMusicDesktop",
                "module": "music.search.SearchCgiService",
                "param": {
                    "num_per_page": "20",
                    "page_num": "1",
                    "query": keyword,
                    "search_type": type_code
                }
            }
        });

        let response = self.send_json_post("https://u.y.qq.com/cgi-bin/musicu.fcg", &data).await?;
        let result: MusicFcgApiResult = serde_json::from_str(&response)?;

        if result.code == 0 && result.req_1.code == 0 && result.req_1.data.code == 0 {
            let vo = result.req_1.data.body.convert(search_type);
            return Ok(ResultVo::success(vo));
        }

        Ok(ResultVo::failure(error_msg::NETWORK_ERROR.to_string()))
    }

    /// Get song information
    pub async fn get_song(&self, id: &str) -> Result<SongResult> {
        let callback = "getOneSongInfoCallback";
        let is_numeric = id.chars().all(|c| c.is_numeric());
        
        let mut params = HashMap::new();
        if is_numeric {
            params.insert("songid", id);
        } else {
            params.insert("songmid", id);
        }
        params.insert("tpl", "yqq_song_detail");
        params.insert("format", "jsonp");
        params.insert("callback", callback);
        params.insert("g_tk", "5381");
        params.insert("jsonpCallback", callback);
        params.insert("loginUin", "0");
        params.insert("hostUin", "0");
        params.insert("outCharset", "utf8");
        params.insert("notice", "0");
        params.insert("platform", "yqq");
        params.insert("needNewCode", "0");

        let response = self.send_post("https://c.y.qq.com/v8/fcg-bin/fcg_play_single_song.fcg", &params).await?;
        let json_str = resolve_resp_json(callback, &response);
        
        let result: SongResult = serde_json::from_str(&json_str)?;
        Ok(result)
    }

    /// Get playlist information
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistResult> {
        let mut params = HashMap::new();
        params.insert("disstid", playlist_id);
        params.insert("format", "json");
        params.insert("outCharset", "utf8");
        params.insert("type", "1");
        params.insert("json", "1");
        params.insert("utf8", "1");
        params.insert("onlysong", "0");
        params.insert("new_format", "1");

        let response = self.send_post("https://c.y.qq.com/qzone/fcg-bin/fcg_ucc_getcdinfo_byids_cp.fcg", &params).await?;
        let result: PlaylistResult = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// Get album information
    pub async fn get_album(&self, album_id: &str) -> Result<AlbumResult> {
        let is_numeric = album_id.chars().all(|c| c.is_numeric());
        
        let mut params = HashMap::new();
        if is_numeric {
            params.insert("albumid", album_id);
        } else {
            params.insert("albummid", album_id);
        }

        let response = self.send_post("https://c.y.qq.com/v8/fcg-bin/fcg_v8_album_info_cp.fcg", &params).await?;
        let result: AlbumResult = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// Get lyric information
    pub async fn get_lyric(&self, song_id: &str) -> Result<LyricResult> {
        let mut params = HashMap::new();
        params.insert("version", "15");
        params.insert("miniversion", "82");
        params.insert("lrctype", "4");
        params.insert("musicid", song_id);

        let mut response = self.send_post("https://c.y.qq.com/qqmusic/fcgi-bin/lyric_download.fcg", &params).await?;
        
        // Remove comments
        response = response.replace("<!--", "").replace("-->", "");

        // Parse XML and extract lyrics
        let mut result = LyricResult {
            code: 0,
            lyric: String::new(),
            trans: String::new(),
            roma: String::new(),
        };

        // Simple XML parsing to extract encrypted lyrics
        if let Some(content_start) = response.find("<content>") {
            if let Some(content_end) = response.find("</content>") {
                let encrypted = &response[content_start + 9..content_end];
                if !encrypted.is_empty() {
                    if let Ok(decrypted) = decrypt_lyrics(encrypted) {
                        result.lyric = decrypted;
                    }
                }
            }
        }

        if let Some(trans_start) = response.find("<contentts>") {
            if let Some(trans_end) = response.find("</contentts>") {
                let encrypted = &response[trans_start + 11..trans_end];
                if !encrypted.is_empty() {
                    if let Ok(decrypted) = decrypt_lyrics(encrypted) {
                        result.trans = decrypted;
                    }
                }
            }
        }

        if let Some(roma_start) = response.find("<contentroma>") {
            if let Some(roma_end) = response.find("</contentroma>") {
                let encrypted = &response[roma_start + 13..roma_end];
                if !encrypted.is_empty() {
                    if let Ok(decrypted) = decrypt_lyrics(encrypted) {
                        result.roma = decrypted;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get song link
    pub async fn get_song_link(&self, song_mid: &str) -> Result<ResultVo<String>> {
        let guid = self.get_guid();

        let data = json!({
            "req": {
                "method": "GetCdnDispatch",
                "module": "CDN.SrfCdnDispatchServer",
                "param": {
                    "guid": guid,
                    "calltype": "0",
                    "userip": ""
                }
            },
            "req_0": {
                "method": "CgiGetVkey",
                "module": "vkey.GetVkeyServer",
                "param": {
                    "guid": "8348972662",
                    "songmid": [song_mid],
                    "songtype": [1],
                    "uin": "0",
                    "loginflag": 1,
                    "platform": "20"
                }
            },
            "comm": {
                "uin": 0,
                "format": "json",
                "ct": 24,
                "cv": 0
            }
        });

        let response = self.send_json_post("https://u.y.qq.com/cgi-bin/musicu.fcg", &data).await?;
        let json_val: serde_json::Value = serde_json::from_str(&response)?;

        if let (Some(req), Some(req_0)) = (json_val.get("req"), json_val.get("req_0")) {
            if req["code"].as_i64() == Some(0) && req_0["code"].as_i64() == Some(0) {
                if let (Some(sip), Some(purl)) = (
                    req["data"]["sip"][0].as_str(),
                    req_0["data"]["midurlinfo"][0]["purl"].as_str()
                ) {
                    let link = format!("{}{}", sip, purl);
                    return Ok(ResultVo::success(link));
                }
            }
        }

        Ok(ResultVo::success(String::new()))
    }

    async fn send_post(&self, url: &str, params: &HashMap<&str, &str>) -> Result<String> {
        let mut req = self.client
            .post(url)
            .header("Referer", "https://c.y.qq.com/")
            .form(params);

        if let Some(cookie) = &self.cookie {
            req = req.header("Cookie", cookie);
        }

        let response = req.send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    async fn send_json_post(&self, url: &str, data: &serde_json::Value) -> Result<String> {
        let mut req = self.client
            .post(url)
            .header("Referer", "https://c.y.qq.com/")
            .json(data);

        if let Some(cookie) = &self.cookie {
            req = req.header("Cookie", cookie);
        }

        let response = req.send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    fn get_guid(&self) -> String {
        let mut rng = rand::thread_rng();
        (0..10)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }
}

fn resolve_resp_json(callback_sign: &str, val: &str) -> String {
    if !val.starts_with(callback_sign) {
        return String::new();
    }

    let json_str = val.replace(&format!("{}(", callback_sign), "");
    json_str.trim_end_matches(')').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let api = QQMusicApi::new(None).unwrap();
        let result = api.search("告白气球", SearchType::SongId).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_resp_json() {
        let input = "callback({\"data\": \"test\"})";
        let result = resolve_resp_json("callback", input);
        assert_eq!(result, "{\"data\": \"test\"}");
    }
}
