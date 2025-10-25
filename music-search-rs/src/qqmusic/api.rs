use crate::error::{MusicSearchError, Result};
use crate::models::*;
use crate::qqmusic::decrypt::decrypt_lyrics;
use crate::qqmusic::models::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use rand::Rng;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};

pub struct QQMusicApi {
    client: Client,
    cookie: Option<String>,
}

impl QQMusicApi {
    pub fn new(cookie: Option<String>) -> Result<Self> {
        info!("Initializing QQ Music API client");
        debug!("Cookie provided: {}", cookie.is_some());
        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()?,
            cookie,
        })
    }

    /// Search for songs, albums, or playlists
    #[instrument(skip(self), fields(service = "qqmusic"))]
    pub async fn search(&self, keyword: &str, search_type: SearchType) -> Result<ResultVo<SearchResultVo>> {
        info!("Searching for '{}' with type {:?}", keyword, search_type);
        
        // 0: song, 2: album, 3: playlist
        let type_code = match search_type {
            SearchType::SongId => 0,
            SearchType::AlbumId => 2,
            SearchType::PlaylistId => 3,
        };
        
        debug!("Type code: {}", type_code);

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
        debug!("Received response, length: {} bytes", response.len());
        
        let result: MusicFcgApiResult = serde_json::from_str(&response)
            .map_err(|e| {
                error!("Failed to parse search response: {}", e);
                e
            })?;

        if result.code == 0 && result.req_1.code == 0 && result.req_1.data.code == 0 {
            let vo = result.req_1.data.body.convert(search_type);
            info!("Search successful, found {} songs, {} albums, {} playlists", 
                vo.song_vos.len(), vo.album_vos.len(), vo.playlist_vos.len());
            return Ok(ResultVo::success(vo));
        }

        warn!("Search failed with codes: result={}, req_1={}, data={}", 
            result.code, result.req_1.code, result.req_1.data.code);
        Ok(ResultVo::failure(error_msg::NETWORK_ERROR.to_string()))
    }

    /// Get song information
    #[instrument(skip(self), fields(service = "qqmusic"))]
    pub async fn get_song(&self, id: &str) -> Result<SongResult> {
        info!("Getting song info for ID: {}", id);
        
        let callback = "getOneSongInfoCallback";
        let is_numeric = id.chars().all(|c| c.is_numeric());
        debug!("Song ID is numeric: {}", is_numeric);
        
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
        debug!("Parsed JSON response length: {} bytes", json_str.len());
        
        let result: SongResult = serde_json::from_str(&json_str)
            .map_err(|e| {
                error!("Failed to parse song response: {}", e);
                e
            })?;
        
        info!("Successfully retrieved song info, songs count: {}", result.data.len());
        Ok(result)
    }

    /// Get playlist information
    #[instrument(skip(self), fields(service = "qqmusic"))]
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistResult> {
        info!("Getting playlist info for ID: {}", playlist_id);
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
        let result: PlaylistResult = serde_json::from_str(&response)
            .map_err(|e| {
                error!("Failed to parse playlist response: {}", e);
                e
            })?;
        
        info!("Successfully retrieved playlist, songs count: {}", 
            result.cdlist.get(0).map(|p| p.song_list.len()).unwrap_or(0));
        Ok(result)
    }

    /// Get album information
    #[instrument(skip(self), fields(service = "qqmusic"))]
    pub async fn get_album(&self, album_id: &str) -> Result<AlbumResult> {
        info!("Getting album info for ID: {}", album_id);
        
        let is_numeric = album_id.chars().all(|c| c.is_numeric());
        debug!("Album ID is numeric: {}", is_numeric);
        
        let mut params = HashMap::new();
        if is_numeric {
            params.insert("albumid", album_id);
        } else {
            params.insert("albummid", album_id);
        }

        let response = self.send_post("https://c.y.qq.com/v8/fcg-bin/fcg_v8_album_info_cp.fcg", &params).await?;
        let result: AlbumResult = serde_json::from_str(&response)
            .map_err(|e| {
                error!("Failed to parse album response: {}", e);
                e
            })?;
        
        info!("Successfully retrieved album, songs count: {}", result.data.list.len());
        Ok(result)
    }

    /// Get lyric information
    #[instrument(skip(self), fields(service = "qqmusic"))]
    pub async fn get_lyric(&self, song_id: &str) -> Result<LyricResult> {
        info!("Getting lyrics for song ID: {}", song_id);
        let mut params = HashMap::new();
        params.insert("version", "15");
        params.insert("miniversion", "82");
        params.insert("lrctype", "4");
        params.insert("musicid", song_id);

        let mut response = self.send_post("https://c.y.qq.com/qqmusic/fcgi-bin/lyric_download.fcg", &params).await?;
        debug!("Received lyrics response, length: {} bytes", response.len());
        debug!("Response preview (first 500 chars): {}", 
            response.chars().take(500).collect::<String>());
        
        // Remove comments
        response = response.replace("<!--", "").replace("-->", "");
        debug!("After removing comments, length: {} bytes", response.len());

        // Parse XML and extract lyrics
        let mut result = LyricResult {
            code: 0,
            lyric: String::new(),
            trans: String::new(),
            roma: String::new(),
        };

        // Use proper XML parsing to extract encrypted lyrics
        let lyrics_map = parse_lyric_xml(&response)?;
        debug!("Parsed XML, found {} lyric tags", lyrics_map.len());
        
        // Process original lyrics (content tag)
        if let Some(encrypted) = lyrics_map.get("content") {
            // Remove ALL whitespace including newlines
            let encrypted: String = encrypted.chars().filter(|c| !c.is_whitespace()).collect();
            if !encrypted.is_empty() {
                debug!("Found encrypted original lyrics, length: {} chars (after whitespace removal), preview: {}", 
                    encrypted.len(), 
                    encrypted.chars().collect::<String>());
                match decrypt_lyrics(&encrypted) {
                    Ok(decrypted) => {
                        debug!("Successfully decrypted original lyrics, length: {} chars", decrypted.len());
                        // Check if decrypted content is XML (Lyric_1 format)
                        if decrypted.contains("<?xml") {
                            if let Ok(inner_lyrics) = parse_nested_lyric_xml(&decrypted) {
                                result.lyric = inner_lyrics;
                            } else {
                                result.lyric = decrypted;
                            }
                        } else {
                            result.lyric = decrypted;
                        }
                    }
                    Err(e) => {
                        error!("Failed to decrypt original lyrics: {}", e);
                    }
                }
            }
        }

        // Process translation lyrics (contentts tag)
        if let Some(encrypted) = lyrics_map.get("contentts") {
            // Remove ALL whitespace including newlines
            let encrypted: String = encrypted.chars().filter(|c| !c.is_whitespace()).collect();
            if !encrypted.is_empty() {
                debug!("Found encrypted translation lyrics, length: {} chars (after whitespace removal)", encrypted.len());
                match decrypt_lyrics(&encrypted) {
                    Ok(decrypted) => {
                        debug!("Successfully decrypted translation lyrics, length: {} chars", decrypted.len());
                        result.trans = decrypted;
                    }
                    Err(e) => {
                        error!("Failed to decrypt translation lyrics: {}", e);
                    }
                }
            }
        }

        // Process romanization lyrics (contentroma tag)
        if let Some(encrypted) = lyrics_map.get("contentroma") {
            // Remove ALL whitespace including newlines
            let encrypted: String = encrypted.chars().filter(|c| !c.is_whitespace()).collect();
            if !encrypted.is_empty() {
                debug!("Found encrypted romanization lyrics, length: {} chars (after whitespace removal)", encrypted.len());
                match decrypt_lyrics(&encrypted) {
                    Ok(decrypted) => {
                        debug!("Successfully decrypted romanization lyrics, length: {} chars", decrypted.len());
                        result.roma = decrypted;
                    }
                    Err(e) => {
                        error!("Failed to decrypt romanization lyrics: {}", e);
                    }
                }
            }
        }

        info!("Lyrics retrieval complete. Original: {}, Translation: {}, Romanization: {}", 
            !result.lyric.is_empty(), !result.trans.is_empty(), !result.roma.is_empty());
        Ok(result)
    }

    /// Get song link
    #[instrument(skip(self), fields(service = "qqmusic"))]
    pub async fn get_song_link(&self, song_mid: &str) -> Result<ResultVo<String>> {
        info!("Fetching song link for track: {}", song_mid);
        let guid = self.get_guid();
        debug!("Using GUID: {}", guid);

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
        debug!("Received song link response, length: {} bytes", response.len());
        
        let json_val: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| {
                error!("Failed to parse song link response: {}", e);
                MusicSearchError::SerializationError(format!("Failed to parse song link response: {}", e))
            })?;

        if let (Some(req), Some(req_0)) = (json_val.get("req"), json_val.get("req_0")) {
            let req_code = req["code"].as_i64().unwrap_or(-1);
            let req_0_code = req_0["code"].as_i64().unwrap_or(-1);
            
            debug!("Response codes: req={}, req_0={}", req_code, req_0_code);
            
            if req_code == 0 && req_0_code == 0 {
                if let (Some(sip), Some(purl)) = (
                    req["data"]["sip"][0].as_str(),
                    req_0["data"]["midurlinfo"][0]["purl"].as_str()
                ) {
                    let link = format!("{}{}", sip, purl);
                    info!("Successfully retrieved song link, URL length: {} chars", link.len());
                    return Ok(ResultVo::success(link));
                } else {
                    warn!("Song link fields missing in response");
                }
            } else {
                warn!("Failed to get song link with codes: req={}, req_0={}", req_code, req_0_code);
            }
        } else {
            error!("Missing 'req' or 'req_0' fields in response");
        }

        info!("No song link available");
        Ok(ResultVo::success(String::new()))
    }

    async fn send_post(&self, url: &str, params: &HashMap<&str, &str>) -> Result<String> {
        debug!("POST request to: {}", url);
        debug!("Parameters count: {}", params.len());
        
        let mut req = self.client
            .post(url)
            .header("Referer", "https://c.y.qq.com/")
            .form(params);

        if let Some(cookie) = &self.cookie {
            req = req.header("Cookie", cookie);
            debug!("Using cookie for authentication");
        }

        let response = req.send().await?;
        let text = response.text().await?;
        debug!("Response received, length: {} bytes", text.len());
        Ok(text)
    }

    async fn send_json_post(&self, url: &str, data: &serde_json::Value) -> Result<String> {
        debug!("JSON POST request to: {}", url);
        
        let mut req = self.client
            .post(url)
            .header("Referer", "https://c.y.qq.com/")
            .json(data);

        if let Some(cookie) = &self.cookie {
            req = req.header("Cookie", cookie);
            debug!("Using cookie for authentication");
        }

        let response = req.send().await?;
        let text = response.text().await?;
        debug!("JSON response received, length: {} bytes", text.len());
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
    debug!("Resolving JSONP response with callback: {}", callback_sign);
    
    if !val.starts_with(callback_sign) {
        warn!("Response doesn't start with expected callback sign: {}", callback_sign);
        return String::new();
    }

    let json_str = val.replace(&format!("{}(", callback_sign), "");
    let result = json_str.trim_end_matches(')').to_string();
    debug!("Extracted JSON length: {} chars", result.len());
    result
}

/// Parse the outer XML response to extract encrypted lyric content from tags
fn parse_lyric_xml(xml_str: &str) -> Result<HashMap<String, String>> {
    let mut reader = Reader::from_str(xml_str);
    reader.trim_text(true);
    
    let mut lyrics = HashMap::new();
    let mut buf = Vec::new();
    let mut current_tag = String::new();
    
    // Tags we're interested in
    let target_tags = ["content", "contentts", "contentroma"];
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if target_tags.contains(&tag_name.as_str()) {
                    current_tag = tag_name;
                }
            }
            Ok(Event::Text(e)) => {
                if !current_tag.is_empty() {
                    let text = String::from_utf8_lossy(&e).to_string();
                    if !text.trim().is_empty() {
                        debug!("Found {} tag with {} bytes (text)", current_tag, text.len());
                        lyrics.insert(current_tag.clone(), text);
                    }
                    current_tag.clear();
                }
            }
            Ok(Event::CData(e)) => {
                if !current_tag.is_empty() {
                    let text = String::from_utf8_lossy(&e).to_string();
                    if !text.trim().is_empty() {
                        debug!("Found {} tag with {} bytes (cdata)", current_tag, text.len());
                        lyrics.insert(current_tag.clone(), text);
                    }
                    current_tag.clear();
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("Error parsing lyric XML at position {}: {}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    
    Ok(lyrics)
}

/// Parse nested XML (Lyric_1 format) to extract the actual lyric content
fn parse_nested_lyric_xml(xml_str: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml_str);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut in_lyric_tag = false;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = e.name();
                let tag_name = String::from_utf8_lossy(name.as_ref()).to_string();
                if tag_name == "Lyric_1" {
                    in_lyric_tag = true;
                    // Extract LyricContent attribute
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        if key == "LyricContent" {
                            let value = attr.unescape_value()?.to_string();
                            debug!("Found LyricContent attribute with {} chars", value.len());
                            return Ok(value);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("Error parsing nested lyric XML: {}", e);
                return Err(MusicSearchError::XmlParse(format!("XML parse error: {}", e)));
            }
            _ => {}
        }
        buf.clear();
    }
    
    if in_lyric_tag {
        debug!("Lyric_1 tag found but no LyricContent attribute");
    }
    
    // If we didn't find the attribute, return the original string
    Ok(xml_str.to_string())
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
