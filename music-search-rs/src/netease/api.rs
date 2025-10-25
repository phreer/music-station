use crate::error::{MusicSearchError, Result};
use crate::models::*;
use crate::netease::models::*;
use aes::Aes128;
use base64::{engine::general_purpose, Engine as _};
use cbc::{Decryptor, Encryptor};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::Rng;
use reqwest::Client;
use rsa::{BigUint, RsaPublicKey};
use serde_json::json;
use std::collections::HashMap;

type Aes128CbcEnc = Encryptor<Aes128>;

const MODULUS: &str = "00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";
const NONCE: &str = "0CoJUm6Qyw8W8jud";
const VI: &[u8] = b"0102030405060708";

pub struct NetEaseMusicApi {
    client: Client,
    secret_key: String,
    enc_sec_key: String,
    cookie: Option<String>,
}

impl NetEaseMusicApi {
    pub fn new(cookie: Option<String>) -> Result<Self> {
        let secret_key = create_secret_key(16);
        let enc_sec_key = rsa_encode(&secret_key)?;

        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()?,
            secret_key,
            enc_sec_key,
            cookie,
        })
    }

    /// Search for songs, albums, or playlists
    pub async fn search(&self, keyword: &str, search_type: SearchType) -> Result<ResultVo<SearchResultVo>> {
        let url = "https://music.163.com/weapi/cloudsearch/get/web";

        // 1: song, 10: album, 1000: playlist
        let type_code = match search_type {
            SearchType::SongId => "1",
            SearchType::AlbumId => "10",
            SearchType::PlaylistId => "1000",
        };

        let data = json!({
            "csrf_token": "",
            "s": keyword,
            "type": type_code,
            "limit": "20",
            "offset": "0"
        });

        let prepared = self.prepare(&data.to_string())?;
        let response = self.send_post(url, &prepared).await?;

        let json_val: serde_json::Value = serde_json::from_str(&response)?;

        let code = json_val["code"].as_i64().unwrap_or(0);

        if code == 50000005 {
            return Ok(ResultVo::failure(error_msg::NEED_LOGIN.to_string()));
        }

        if let Some(result) = json_val["result"].as_object() {
            if code == 200 {
                let result_str = serde_json::to_string(result)?;
                let search_result: SearchResult = serde_json::from_str(&result_str)?;
                let vo = search_result.convert(search_type);
                return Ok(ResultVo::success(vo));
            }
        }

        Ok(ResultVo::failure(error_msg::SONG_NOT_EXIST.to_string()))
    }

    /// Get songs by IDs
    pub async fn get_songs(&self, song_ids: &[String]) -> Result<HashMap<String, Song>> {
        if song_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let url = "https://music.163.com/weapi/v3/song/detail?csrf_token=";
        
        let songs: Vec<serde_json::Value> = song_ids
            .iter()
            .map(|id| json!({"id": id}))
            .collect();

        let data = json!({
            "c": serde_json::to_string(&songs)?,
            "csrf_token": ""
        });

        let prepared = self.prepare(&data.to_string())?;
        let response = self.send_post(url, &prepared).await?;

        let detail_result: DetailResult = serde_json::from_str(&response)?;

        let mut result = HashMap::new();
        if detail_result.code == 200 {
            for song in detail_result.songs {
                result.insert(song.id.clone(), song);
            }
        }

        Ok(result)
    }

    /// Get playlist information
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistResult> {
        let url = "https://music.163.com/weapi/v6/playlist/detail?csrf_token=";

        let data = json!({
            "csrf_token": "",
            "id": playlist_id,
            "offset": "0",
            "total": "true",
            "limit": "1000",
            "n": "1000"
        });

        let prepared = self.prepare(&data.to_string())?;
        let response = self.send_post(url, &prepared).await?;

        let result: PlaylistResult = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// Get album information
    pub async fn get_album(&self, album_id: &str) -> Result<AlbumResult> {
        let url = format!("https://music.163.com/weapi/v1/album/{}?csrf_token=", album_id);

        let data = json!({
            "csrf_token": ""
        });

        let prepared = self.prepare(&data.to_string())?;
        let response = self.send_post(&url, &prepared).await?;

        let result: AlbumResult = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// Get lyric information
    pub async fn get_lyric(&self, song_id: &str) -> Result<LyricResult> {
        let url = "https://music.163.com/weapi/song/lyric?csrf_token=";

        let data = json!({
            "id": song_id,
            "os": "pc",
            "lv": "-1",
            "kv": "-1",
            "tv": "-1",
            "rv": "-1",
            "yv": "-1",
            "ytv": "-1",
            "yrv": "-1",
            "csrf_token": ""
        });

        let prepared = self.prepare(&data.to_string())?;
        let response = self.send_post(url, &prepared).await?;

        let result: LyricResult = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// Get song URL
    pub async fn get_song_url(&self, song_ids: &[String]) -> Result<HashMap<String, Datum>> {
        let url = "https://music.163.com/weapi/song/enhance/player/url?csrf_token=";

        let ids_str = format!("[{}]", song_ids.join(","));
        let data = json!({
            "ids": ids_str,
            "br": "999000",
            "csrf_token": ""
        });

        let prepared = self.prepare(&data.to_string())?;
        let response = self.send_post(url, &prepared).await?;

        let song_urls: SongUrls = serde_json::from_str(&response)?;

        let mut result = HashMap::new();
        if song_urls.code == 200 {
            for datum in song_urls.data {
                result.insert(datum.id.clone(), datum);
            }
        }

        Ok(result)
    }

    fn prepare(&self, raw: &str) -> Result<HashMap<String, String>> {
        let mut params = aes_encode(raw, NONCE)?;
        params = aes_encode(&params, &self.secret_key)?;

        let mut data = HashMap::new();
        data.insert("params".to_string(), params);
        data.insert("encSecKey".to_string(), self.enc_sec_key.clone());

        Ok(data)
    }

    async fn send_post(&self, url: &str, data: &HashMap<String, String>) -> Result<String> {
        let mut req = self.client
            .post(url)
            .header("Referer", "https://music.163.com/")
            .form(data);

        if let Some(cookie) = &self.cookie {
            req = req.header("Cookie", cookie);
        }

        let response = req.send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}

fn aes_encode(secret_data: &str, secret: &str) -> Result<String> {
    use cbc::cipher::block_padding::Pkcs7;

    let key = secret.as_bytes();
    let iv = VI;

    let cipher = Aes128CbcEnc::new(key.into(), iv.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(secret_data.as_bytes());

    Ok(general_purpose::STANDARD.encode(&ciphertext))
}

fn rsa_encode(text: &str) -> Result<String> {
    // Reverse the text
    let reversed: String = text.chars().rev().collect();
    
    // Convert to hex
    let hex_str = hex::encode(reversed.as_bytes());
    
    // Parse as BigUint
    let a = BigUint::parse_bytes(hex_str.as_bytes(), 16)
        .ok_or_else(|| MusicSearchError::Encryption("Failed to parse hex".to_string()))?;
    
    let e = BigUint::parse_bytes(b"010001", 16)
        .ok_or_else(|| MusicSearchError::Encryption("Failed to parse exponent".to_string()))?;
    
    let n = BigUint::parse_bytes(MODULUS.as_bytes(), 16)
        .ok_or_else(|| MusicSearchError::Encryption("Failed to parse modulus".to_string()))?;
    
    // Perform modular exponentiation: result = a^e mod n
    let result = a.modpow(&e, &n);
    
    // Convert to hex string and pad to 256 characters
    let mut key = format!("{:x}", result);
    if key.len() < 256 {
        key = format!("{:0>256}", key);
    } else if key.len() > 256 {
        key = key[key.len() - 256..].to_string();
    }
    
    Ok(key)
}

fn create_secret_key(length: usize) -> String {
    const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secret_key() {
        let key = create_secret_key(16);
        assert_eq!(key.len(), 16);
    }

    #[tokio::test]
    async fn test_search() {
        let api = NetEaseMusicApi::new(None).unwrap();
        let result = api.search("告白气球", SearchType::SongId).await;
        assert!(result.is_ok());
    }
}
