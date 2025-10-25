use crate::error::{MusicSearchError, Result};
use flate2::read::ZlibDecoder;
use std::io::Read;

const QQ_KEY: &[u8] = b"!@#)(*$%123ZXC!@!@#)(NHL";

// Custom DES implementation ported from C# QQMusicearchUtils.cs
// This is needed because the standard `des` crate produces different results
// than QQ Music's custom DES implementation

const ENCRYPT: u32 = 1;
const DECRYPT: u32 = 0;

fn bitnum(a: &[u8], b: usize, c: usize) -> u32 {
    (((a[b / 32 * 4 + 3 - b % 32 / 8] >> (7 - (b % 8))) & 0x01) as u32) << c
}

fn bitnumintr(a: u32, b: usize, c: usize) -> u8 {
    (((a >> (31 - b)) & 0x00000001) << c) as u8
}

fn bitnumintl(a: u32, b: usize, c: usize) -> u32 {
    ((a << b) & 0x80000000) >> c
}

fn sboxbit(a: u8) -> usize {
    ((a & 0x20) | ((a & 0x1f) >> 1) | ((a & 0x01) << 4)) as usize
}

const SBOX1: [u8; 64] = [
    14,  4,  13,  1,   2, 15,  11,  8,   3, 10,   6, 12,   5,  9,   0,  7,
     0, 15,   7,  4,  14,  2,  13,  1,  10,  6,  12, 11,   9,  5,   3,  8,
     4,  1,  14,  8,  13,  6,   2, 11,  15, 12,   9,  7,   3, 10,   5,  0,
    15, 12,   8,  2,   4,  9,   1,  7,   5, 11,   3, 14,  10,  0,   6, 13
];

const SBOX2: [u8; 64] = [
    15,  1,   8, 14,   6, 11,   3,  4,   9,  7,   2, 13,  12,  0,   5, 10,
     3, 13,   4,  7,  15,  2,   8, 15,  12,  0,   1, 10,   6,  9,  11,  5,
     0, 14,   7, 11,  10,  4,  13,  1,   5,  8,  12,  6,   9,  3,   2, 15,
    13,  8,  10,  1,   3, 15,   4,  2,  11,  6,   7, 12,   0,  5,  14,  9
];

const SBOX3: [u8; 64] = [
    10,  0,   9, 14,   6,  3,  15,  5,   1, 13,  12,  7,  11,  4,   2,  8,
    13,  7,   0,  9,   3,  4,   6, 10,   2,  8,   5, 14,  12, 11,  15,  1,
    13,  6,   4,  9,   8, 15,   3,  0,  11,  1,   2, 12,   5, 10,  14,  7,
     1, 10,  13,  0,   6,  9,   8,  7,   4, 15,  14,  3,  11,  5,   2, 12
];

const SBOX4: [u8; 64] = [
     7, 13,  14,  3,   0,  6,   9, 10,   1,  2,   8,  5,  11, 12,   4, 15,
    13,  8,  11,  5,   6, 15,   0,  3,   4,  7,   2, 12,   1, 10,  14,  9,
    10,  6,   9,  0,  12, 11,   7, 13,  15,  1,   3, 14,   5,  2,   8,  4,
     3, 15,   0,  6,  10, 10,  13,  8,   9,  4,   5, 11,  12,  7,   2, 14
];

const SBOX5: [u8; 64] = [
     2, 12,   4,  1,   7, 10,  11,  6,   8,  5,   3, 15,  13,  0,  14,  9,
    14, 11,   2, 12,   4,  7,  13,  1,   5,  0,  15, 10,   3,  9,   8,  6,
     4,  2,   1, 11,  10, 13,   7,  8,  15,  9,  12,  5,   6,  3,   0, 14,
    11,  8,  12,  7,   1, 14,   2, 13,   6, 15,   0,  9,  10,  4,   5,  3
];

const SBOX6: [u8; 64] = [
    12,  1,  10, 15,   9,  2,   6,  8,   0, 13,   3,  4,  14,  7,   5, 11,
    10, 15,   4,  2,   7, 12,   9,  5,   6,  1,  13, 14,   0, 11,   3,  8,
     9, 14,  15,  5,   2,  8,  12,  3,   7,  0,   4, 10,   1, 13,  11,  6,
     4,  3,   2, 12,   9,  5,  15, 10,  11, 14,   1,  7,   6,  0,   8, 13
];

const SBOX7: [u8; 64] = [
     4, 11,   2, 14,  15,  0,   8, 13,   3, 12,   9,  7,   5, 10,   6,  1,
    13,  0,  11,  7,   4,  9,   1, 10,  14,  3,   5, 12,   2, 15,   8,  6,
     1,  4,  11, 13,  12,  3,   7, 14,  10, 15,   6,  8,   0,  5,   9,  2,
     6, 11,  13,  8,   1,  4,  10,  7,   9,  5,   0, 15,  14,  2,   3, 12
];

const SBOX8: [u8; 64] = [
    13,  2,   8,  4,   6, 15,  11,  1,  10,  9,   3, 14,   5,  0,  12,  7,
     1, 15,  13,  8,  10,  3,   7,  4,  12,  5,   6, 11,   0, 14,   9,  2,
     7, 11,   4,  1,   9, 12,  14,  2,   0,  6,  10, 13,  15,  3,   5,  8,
     2,  1,  14,  7,   4, 10,   8, 13,  15, 12,   9,  0,   3,  5,   6, 11
];

fn key_schedule(key: &[u8], schedule: &mut [[u8; 6]; 16], mode: u32) {
    let key_rnd_shift: [u32; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];
    let key_perm_c: [usize; 28] = [
        56, 48, 40, 32, 24, 16, 8, 0, 57, 49, 41, 33, 25, 17,
        9, 1, 58, 50, 42, 34, 26, 18, 10, 2, 59, 51, 43, 35
    ];
    let key_perm_d: [usize; 28] = [
        62, 54, 46, 38, 30, 22, 14, 6, 61, 53, 45, 37, 29, 21,
        13, 5, 60, 52, 44, 36, 28, 20, 12, 4, 27, 19, 11, 3
    ];
    let key_compression: [usize; 48] = [
        13, 16, 10, 23, 0, 4, 2, 27, 14, 5, 20, 9,
        22, 18, 11, 3, 25, 7, 15, 6, 26, 19, 12, 1,
        40, 51, 30, 36, 46, 54, 29, 39, 50, 44, 32, 47,
        43, 48, 38, 55, 33, 52, 45, 41, 49, 35, 28, 31
    ];

    let mut c = 0u32;
    let mut d = 0u32;

    for i in 0..28 {
        c |= bitnum(key, key_perm_c[i], 31 - i);
    }

    for i in 0..28 {
        d |= bitnum(key, key_perm_d[i], 31 - i);
    }

    for i in 0..16 {
        c = ((c << key_rnd_shift[i]) | (c >> (28 - key_rnd_shift[i]))) & 0xfffffff0;
        d = ((d << key_rnd_shift[i]) | (d >> (28 - key_rnd_shift[i]))) & 0xfffffff0;

        let to_gen = if mode == DECRYPT {
            15 - i
        } else {
            i
        };

        for j in 0..6 {
            schedule[to_gen][j] = 0;
        }

        for j in 0..24 {
            schedule[to_gen][j / 8] |= bitnumintr(c, key_compression[j], 7 - (j % 8));
        }

        for j in 24..48 {
            schedule[to_gen][j / 8] |= bitnumintr(d, key_compression[j] - 27, 7 - (j % 8));
        }
    }
}

fn ip(state: &mut [u32; 2], input: &[u8]) {
    state[0] = bitnum(input, 57, 31) | bitnum(input, 49, 30) | bitnum(input, 41, 29) | bitnum(input, 33, 28) |
               bitnum(input, 25, 27) | bitnum(input, 17, 26) | bitnum(input, 9, 25) | bitnum(input, 1, 24) |
               bitnum(input, 59, 23) | bitnum(input, 51, 22) | bitnum(input, 43, 21) | bitnum(input, 35, 20) |
               bitnum(input, 27, 19) | bitnum(input, 19, 18) | bitnum(input, 11, 17) | bitnum(input, 3, 16) |
               bitnum(input, 61, 15) | bitnum(input, 53, 14) | bitnum(input, 45, 13) | bitnum(input, 37, 12) |
               bitnum(input, 29, 11) | bitnum(input, 21, 10) | bitnum(input, 13, 9) | bitnum(input, 5, 8) |
               bitnum(input, 63, 7) | bitnum(input, 55, 6) | bitnum(input, 47, 5) | bitnum(input, 39, 4) |
               bitnum(input, 31, 3) | bitnum(input, 23, 2) | bitnum(input, 15, 1) | bitnum(input, 7, 0);

    state[1] = bitnum(input, 56, 31) | bitnum(input, 48, 30) | bitnum(input, 40, 29) | bitnum(input, 32, 28) |
               bitnum(input, 24, 27) | bitnum(input, 16, 26) | bitnum(input, 8, 25) | bitnum(input, 0, 24) |
               bitnum(input, 58, 23) | bitnum(input, 50, 22) | bitnum(input, 42, 21) | bitnum(input, 34, 20) |
               bitnum(input, 26, 19) | bitnum(input, 18, 18) | bitnum(input, 10, 17) | bitnum(input, 2, 16) |
               bitnum(input, 60, 15) | bitnum(input, 52, 14) | bitnum(input, 44, 13) | bitnum(input, 36, 12) |
               bitnum(input, 28, 11) | bitnum(input, 20, 10) | bitnum(input, 12, 9) | bitnum(input, 4, 8) |
               bitnum(input, 62, 7) | bitnum(input, 54, 6) | bitnum(input, 46, 5) | bitnum(input, 38, 4) |
               bitnum(input, 30, 3) | bitnum(input, 22, 2) | bitnum(input, 14, 1) | bitnum(input, 6, 0);
}

fn inv_ip(state: &[u32; 2], output: &mut [u8]) {
    output[3] = (bitnumintr(state[1], 7, 7) | bitnumintr(state[0], 7, 6) | bitnumintr(state[1], 15, 5) |
                 bitnumintr(state[0], 15, 4) | bitnumintr(state[1], 23, 3) | bitnumintr(state[0], 23, 2) |
                 bitnumintr(state[1], 31, 1) | bitnumintr(state[0], 31, 0)) as u8;

    output[2] = (bitnumintr(state[1], 6, 7) | bitnumintr(state[0], 6, 6) | bitnumintr(state[1], 14, 5) |
                 bitnumintr(state[0], 14, 4) | bitnumintr(state[1], 22, 3) | bitnumintr(state[0], 22, 2) |
                 bitnumintr(state[1], 30, 1) | bitnumintr(state[0], 30, 0)) as u8;

    output[1] = (bitnumintr(state[1], 5, 7) | bitnumintr(state[0], 5, 6) | bitnumintr(state[1], 13, 5) |
                 bitnumintr(state[0], 13, 4) | bitnumintr(state[1], 21, 3) | bitnumintr(state[0], 21, 2) |
                 bitnumintr(state[1], 29, 1) | bitnumintr(state[0], 29, 0)) as u8;

    output[0] = (bitnumintr(state[1], 4, 7) | bitnumintr(state[0], 4, 6) | bitnumintr(state[1], 12, 5) |
                 bitnumintr(state[0], 12, 4) | bitnumintr(state[1], 20, 3) | bitnumintr(state[0], 20, 2) |
                 bitnumintr(state[1], 28, 1) | bitnumintr(state[0], 28, 0)) as u8;

    output[7] = (bitnumintr(state[1], 3, 7) | bitnumintr(state[0], 3, 6) | bitnumintr(state[1], 11, 5) |
                 bitnumintr(state[0], 11, 4) | bitnumintr(state[1], 19, 3) | bitnumintr(state[0], 19, 2) |
                 bitnumintr(state[1], 27, 1) | bitnumintr(state[0], 27, 0)) as u8;

    output[6] = (bitnumintr(state[1], 2, 7) | bitnumintr(state[0], 2, 6) | bitnumintr(state[1], 10, 5) |
                 bitnumintr(state[0], 10, 4) | bitnumintr(state[1], 18, 3) | bitnumintr(state[0], 18, 2) |
                 bitnumintr(state[1], 26, 1) | bitnumintr(state[0], 26, 0)) as u8;

    output[5] = (bitnumintr(state[1], 1, 7) | bitnumintr(state[0], 1, 6) | bitnumintr(state[1], 9, 5) |
                 bitnumintr(state[0], 9, 4) | bitnumintr(state[1], 17, 3) | bitnumintr(state[0], 17, 2) |
                 bitnumintr(state[1], 25, 1) | bitnumintr(state[0], 25, 0)) as u8;

    output[4] = (bitnumintr(state[1], 0, 7) | bitnumintr(state[0], 0, 6) | bitnumintr(state[1], 8, 5) |
                 bitnumintr(state[0], 8, 4) | bitnumintr(state[1], 16, 3) | bitnumintr(state[0], 16, 2) |
                 bitnumintr(state[1], 24, 1) | bitnumintr(state[0], 24, 0)) as u8;
}

fn des_f(mut state: u32, key: &[u8; 6]) -> u32 {
    let mut lrgstate = [0u8; 6];

    let t1 = bitnumintl(state, 31, 0) | ((state & 0xf0000000) >> 1) | bitnumintl(state, 4, 5) |
             bitnumintl(state, 3, 6) | ((state & 0x0f000000) >> 3) | bitnumintl(state, 8, 11) |
             bitnumintl(state, 7, 12) | ((state & 0x00f00000) >> 5) | bitnumintl(state, 12, 17) |
             bitnumintl(state, 11, 18) | ((state & 0x000f0000) >> 7) | bitnumintl(state, 16, 23);

    let t2 = bitnumintl(state, 15, 0) | ((state & 0x0000f000) << 15) | bitnumintl(state, 20, 5) |
             bitnumintl(state, 19, 6) | ((state & 0x00000f00) << 13) | bitnumintl(state, 24, 11) |
             bitnumintl(state, 23, 12) | ((state & 0x000000f0) << 11) | bitnumintl(state, 28, 17) |
             bitnumintl(state, 27, 18) | ((state & 0x0000000f) << 9) | bitnumintl(state, 0, 23);

    lrgstate[0] = ((t1 >> 24) & 0x000000ff) as u8;
    lrgstate[1] = ((t1 >> 16) & 0x000000ff) as u8;
    lrgstate[2] = ((t1 >> 8) & 0x000000ff) as u8;
    lrgstate[3] = ((t2 >> 24) & 0x000000ff) as u8;
    lrgstate[4] = ((t2 >> 16) & 0x000000ff) as u8;
    lrgstate[5] = ((t2 >> 8) & 0x000000ff) as u8;

    lrgstate[0] ^= key[0];
    lrgstate[1] ^= key[1];
    lrgstate[2] ^= key[2];
    lrgstate[3] ^= key[3];
    lrgstate[4] ^= key[4];
    lrgstate[5] ^= key[5];

    state = ((SBOX1[sboxbit(lrgstate[0] >> 2)] as u32) << 28) |
            ((SBOX2[sboxbit(((lrgstate[0] & 0x03) << 4) | (lrgstate[1] >> 4))] as u32) << 24) |
            ((SBOX3[sboxbit(((lrgstate[1] & 0x0f) << 2) | (lrgstate[2] >> 6))] as u32) << 20) |
            ((SBOX4[sboxbit(lrgstate[2] & 0x3f)] as u32) << 16) |
            ((SBOX5[sboxbit(lrgstate[3] >> 2)] as u32) << 12) |
            ((SBOX6[sboxbit(((lrgstate[3] & 0x03) << 4) | (lrgstate[4] >> 4))] as u32) << 8) |
            ((SBOX7[sboxbit(((lrgstate[4] & 0x0f) << 2) | (lrgstate[5] >> 6))] as u32) << 4) |
            (SBOX8[sboxbit(lrgstate[5] & 0x3f)] as u32);

    state = bitnumintl(state, 15, 0) | bitnumintl(state, 6, 1) | bitnumintl(state, 19, 2) |
            bitnumintl(state, 20, 3) | bitnumintl(state, 28, 4) | bitnumintl(state, 11, 5) |
            bitnumintl(state, 27, 6) | bitnumintl(state, 16, 7) | bitnumintl(state, 0, 8) |
            bitnumintl(state, 14, 9) | bitnumintl(state, 22, 10) | bitnumintl(state, 25, 11) |
            bitnumintl(state, 4, 12) | bitnumintl(state, 17, 13) | bitnumintl(state, 30, 14) |
            bitnumintl(state, 9, 15) | bitnumintl(state, 1, 16) | bitnumintl(state, 7, 17) |
            bitnumintl(state, 23, 18) | bitnumintl(state, 13, 19) | bitnumintl(state, 31, 20) |
            bitnumintl(state, 26, 21) | bitnumintl(state, 2, 22) | bitnumintl(state, 8, 23) |
            bitnumintl(state, 18, 24) | bitnumintl(state, 12, 25) | bitnumintl(state, 29, 26) |
            bitnumintl(state, 5, 27) | bitnumintl(state, 21, 28) | bitnumintl(state, 10, 29) |
            bitnumintl(state, 3, 30) | bitnumintl(state, 24, 31);

    state
}

fn des_crypt(input: &[u8], output: &mut [u8], key: &[[u8; 6]; 16]) {
    let mut state = [0u32; 2];

    ip(&mut state, input);

    for idx in 0..15 {
        let t = state[1];
        state[1] = des_f(state[1], &key[idx]) ^ state[0];
        state[0] = t;
    }

    state[0] = des_f(state[1], &key[15]) ^ state[0];

    inv_ip(&state, output);
}

fn triple_des_key_setup(key: &[u8], schedule: &mut [[[u8; 6]; 16]; 3], mode: u32) {
    if mode == ENCRYPT {
        key_schedule(&key[0..8], &mut schedule[0], mode);
        key_schedule(&key[8..16], &mut schedule[1], DECRYPT);
        key_schedule(&key[16..24], &mut schedule[2], mode);
    } else {
        key_schedule(&key[0..8], &mut schedule[2], mode);
        key_schedule(&key[8..16], &mut schedule[1], ENCRYPT);
        key_schedule(&key[16..24], &mut schedule[0], mode);
    }
}

fn triple_des_crypt(input: &[u8], output: &mut [u8], key: &[[[u8; 6]; 16]; 3]) {
    let mut temp = [0u8; 8];
    des_crypt(input, output, &key[0]);
    temp.copy_from_slice(output);
    des_crypt(&temp, output, &key[1]);
    temp.copy_from_slice(output);
    des_crypt(&temp, output, &key[2]);
}


/// Decrypts QQ Music lyrics data using Triple-DES and DEFLATE decompression
///
/// The decryption process:
/// 1. Parse hex string to bytes
/// 2. Decrypt using Triple-DES with custom implementation (EDE mode)
/// 3. Decompress using DEFLATE
/// 4. Convert to UTF-8 string
///
/// # Arguments
/// * `encrypted_hex` - Hex string of encrypted lyrics data
///
/// # Returns
/// Decrypted lyrics as UTF-8 string
pub fn decrypt_lyrics(encrypted_hex: &str) -> Result<String> {
    // Parse hex string to bytes
    let encrypted_data = hex_string_to_bytes(encrypted_hex)?;

    // Setup Triple-DES key schedule
    let mut key_schedule = [[[0u8; 6]; 16]; 3];
    triple_des_key_setup(QQ_KEY, &mut key_schedule, DECRYPT);

    // Decrypt data in 8-byte blocks using Triple-DES
    let mut decrypted = vec![0u8; encrypted_data.len()];
    for (i, chunk) in encrypted_data.chunks(8).enumerate() {
        let offset = i * 8;
        if chunk.len() == 8 {
            triple_des_crypt(chunk, &mut decrypted[offset..offset + 8], &key_schedule);
        } else {
            // Handle last block if not 8 bytes (should not happen with valid data)
            let mut padded = [0u8; 8];
            padded[..chunk.len()].copy_from_slice(chunk);
            let mut output = [0u8; 8];
            triple_des_crypt(&padded, &mut output, &key_schedule);
            decrypted[offset..offset + chunk.len()].copy_from_slice(&output[..chunk.len()]);
        }
    }

    println!("Decrypted bytes (32 total): {:02X?}", &decrypted);
    // Decompress using DEFLATE
    let decompressed = deflate_decompress(&decrypted)?;

    // Convert to UTF-8
    String::from_utf8(decompressed).map_err(|e| {
        MusicSearchError::DecryptionError(format!("Failed to convert to UTF-8: {}", e))
    })
}


fn hex_string_to_bytes(hex_string: &str) -> Result<Vec<u8>> {
    use tracing::debug;
    
    // Validate input is even length
    if hex_string.len() % 2 != 0 {
        return Err(MusicSearchError::Other(format!(
            "Hex string has odd length: {}", hex_string.len()
        )));
    }
    
    // Validate all characters are valid hex
    for (i, ch) in hex_string.chars().enumerate() {
        if !ch.is_ascii_hexdigit() {
            return Err(MusicSearchError::Other(format!(
                "Invalid hex character '{}' at position {}", ch, i
            )));
        }
    }
    
    let mut bytes = Vec::with_capacity(hex_string.len() / 2);
    for i in (0..hex_string.len()).step_by(2) {
        if i + 2 > hex_string.len() {
            break;
        }
        let byte = u8::from_str_radix(&hex_string[i..i+2], 16)
            .map_err(|e| MusicSearchError::Other(format!("Hex parse error at position {}: {}", i, e)))?;
        bytes.push(byte);
    }
    debug!("hex_string_to_bytes: converted {} chars to {} bytes", hex_string.len(), bytes.len());
    Ok(bytes)
}

fn deflate_decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decrypt_full_debug() {
        let encrypted = "00367FE8E50542ABECE8E677924C7C2DF977E3910E7E4272C1D871D80BFF1C12";
        let encrypted_data = hex_string_to_bytes(encrypted).unwrap();
        
        // Setup Triple-DES key schedule
        let mut key_schedule = [[[0u8; 6]; 16]; 3];
        triple_des_key_setup(QQ_KEY, &mut key_schedule, DECRYPT);
        
        // Decrypt all blocks
        let mut decrypted = vec![0u8; encrypted_data.len()];
        for (i, chunk) in encrypted_data.chunks(8).enumerate() {
            let offset = i * 8;
            triple_des_crypt(chunk, &mut decrypted[offset..offset + 8], &key_schedule);
        }
        
        println!("Decrypted bytes (32 total): {:02X?}", &decrypted);
        
        // Try DEFLATE decompression
        match deflate_decompress(&decrypted) {
            Ok(decompressed) => {
                println!("Decompression succeeded! {} bytes", decompressed.len());
                let s = String::from_utf8_lossy(&decompressed[0..100.min(decompressed.len())]);
                println!("First 100 chars: {}", s);
            }
            Err(e) => {
                println!("Decompression failed: {}", e);
            }
        }
    }
    
    #[test]
    fn test_decrypt_lyrics_debug() {
        let encrypted = "00367FE8E50542ABECE8E677924C7C2D";
        let encrypted_data = hex_string_to_bytes(encrypted).unwrap();
        
        // Setup Triple-DES key schedule
        let mut key_schedule = [[[0u8; 6]; 16]; 3];
        triple_des_key_setup(QQ_KEY, &mut key_schedule, DECRYPT);
        
        // Decrypt first block
        let mut decrypted = [0u8; 8];
        triple_des_crypt(&encrypted_data[0..8], &mut decrypted, &key_schedule);
        
        println!("First 8 decrypted bytes: {:02X?}", decrypted);
        
        // Expected from C# test: 78-9C-45-58-DB-6E-55-D7
        let expected = [0x78, 0x9C, 0x45, 0x58, 0xDB, 0x6E, 0x55, 0xD7];
        assert_eq!(decrypted, expected, "First block should match C# output");
    }
    
    #[test]
    fn test_decrypt_lyrics() {
        let encrypted = "00367FE8E50542ABECE8E677924C7C2DF977E3910E7E4272C1D871D80BFF1C12E71AEC96FF14066D18DB3C9A36181E3489C695667311BDC4B3AE764754CD0C3EF8E7939D7BC630D19C470098B6E380BBF365F6DEBDEC438477A0A67B986E76A5863F1FE9E0C064936BAB9D5B7F48B4D5E732478C94453991BFA073F7D906F8837D998F0F1CF8529C8116FE1631683E7F28696BBCAA1DBB09FC50CA505B39626EBC0B25E29281047A51906BE626B6AE79D507E125D738AEA5A212CB18304EC6D9D1E26380595D3FD18F5E0C025FC87F5B334E2B466DA584B930C2557616970378FD399AB271360DA939494536327156112AA383BC3E06237FDDFB1EC48916B85C9B4C0AD0745CFDA641BE78EE4AF707CFAD9AAB21511CF508EF0861E1D9D7F6F1B925974ABE0D287B3B40CB3F2C624A2EFFCC6F4AB61E3E91E8DB0A4CC986F60D75D24A99245920347B992DA15FBD069814E266DA6C6A87D67CCA1585D4F403282A6812703A1AF3E54617369E1F10D4E850850B21445A066BAEDE07982E4995AE662B40D71461388A92DBD0E1A9CDCCB8EA425BED5705F343C3A84B5844C2B31CBAAF85C5CADD1E65D73A402664F260DA788297ED67D049463C39FDF19FFFA00A024D512BD9AA69C7E5E996A2578EAFE9D7C2F223FDBAF7C144AAF4170C6802A40122A0AA592662F7FF0A3F53EF0B316BF743D6D40FB6162DA0C3F725D88B2835E458CEA9F30C63D32B65CE4DE12FAED0B8FE345C1AB3B69EC507BFA87E322F434E12A97B33B4BA95219112D9299D90E6121BEDBC8278943F2035F25F9DE6C174CAC8C3460C25E5DB4B1226A4BE07B366E98C26D756D56DC94EAA7639616A5441417BD0199059D4A71118CDFDCE41069B3A7F51D839A015B6ACDD2BCAD5E07AB4E8E2770D74C7CC91C873697F91660D9BF8F86F319C10A45623C1EDA880D56F24FB76BD111F7F3970EC1598DF2BF6A54914AC1795F59AC958006E810CC5EA8A41C55E0F39A0DA57690C5FFDF301836D099166C34EAC3F021AE998FB29F48B3C9855CC1C4529C5FF767C83EBF5B0DCAE86BC2AC7EEBDEED7BA54AB88221311F80D6105F9446B5B951F3946572C72BA4BA277893186A056C2A63D49DAE06225E61B39034BE0BCC2A15FFBA2947838F0F0B5992F280A86B6AA6176B79157BB5B773443BA4CCA7D810E5038AF70EC3F25022EE7148C78B1A637A8F9B2E9BD23DA56F46D138F4C7E938EFB251B9A8FBC92A65A1DA5155C07C27A5AF197939BC0773E7D1E68A3DBE91A062DC93BB8C8C1EE8502533AFDB1E30870119C3728A3BFF0988D6569AAC222EF336CA086FD8B74156F1C104B51AC7FA5079710C00BF6F53E6455EF12D2B9B7F62239F7A330D3E1B055D1A8BA0F2E73E74AB37F03BF554627BB7BAF8397CB51FD38D0CEF8C063425F39F9A15816F5C6C784A048E73A4E7A4BF95CBC06B926DF54C6512A0802C4D023EB2EFFCDC7CFE0D3CDEDD9E528C8260A7E51E60B23EA3DDBBFC2818EAAB2E2B27C84F7C12B160FFE3B0E3C6C61633A9F1CCC69C1CADEFB883A86A108C7E16890E03241F782E47EAF6B79C0E2BF9D3CFE58D80F98DB8CDF2F4B54B03210DF45BDD338DE6C1E817E1C83B5603F9E672E428D995E1A038253241534C40F0E53CB2B7F991032EFCF3BA9BA3A614FFA6C4C005E652C438173672A997A27922AA630DBECC211BBF2F4DF7D005767848AFA2F32D5D6E48E002DE70D89FD4EBD17206F2564EBF66FB2D2E9CC28FF662C93E99B8613F643C31A083101115EA56650DA8ACF595FCB501FE274AFCA4C1CC87F67F751141DB3E76D70CA8142FD298454459FA0D7DFC5A6EB99E45FEA549C3D01E7980ADA53398E916E800E261DC5159BE7CA18A21668166925C861F08F2133DCBC0DB5CCD379DAE4372A41A51C045633980251E2E9DF41D08D93868EE3E0B9B668AFD4A22F5F858D74298FD5D625B8B13F5D1E5BD6801292619DA98281D6DA0211AB6E5F359F1F1DF30B7074253A94CCAA0026CFBF116DD6FCF08F1C294C9E6EF9913E9817A0882F66B1299980984B59511EECB1933181765BE7E9A8DF5B8D673E8FA9458BD8BB5E7FC7FF87415694316664EFE79114636EEE2D94B41BF45DC147CD7055C3B1750FC2A8F95EB89765F125A85BA9EA31F2AB4FD362DEAC6BD21EBFE92CD71560385E30B98F68CACF75DFE27978F91804C9D3FBA05DF3E04439DA8BEBAEAA8AEABE6C082AD07006F4114B6FC44E68751ED354568682F7746BC69861C97659EB2E211A87BF0ABA22239E86DE5A0D17F6FBC8EA4C81DAEACC6593A7EB506115BB917A5F6430758536EE7536937F257ACDAE2922350F77890078D59E675D566173CD98ED44C49F362A527163805E64A0D68C5EBEB7672B76AA19A3F929663A3ED999E3B23D8F103C16BD58AC5CD731A6F62E3F4DC28049732C0493AE1FFFDB08260F9C0B44BC16F3216DBCF97ED67EB3A4C85A4DD409B6B6109FF19518E0D9C1F9D69265C5F2A0AB34ABA569B750FC32F8ACA4C4ECDEC57631D8096403CF01F791749E0FA1E77E42F529997B5F238D928188EB7E85BFC882ACC16F9E5EABA74DA4F5B5AB3F8C2D87E6360C6E59950337FE11955790E9E830D8BCBE3958B8B4AB24FBFD31D03B1B47BF667500B58DB1955907807B661C12B67BE31EB3C34EFE046D8DF331FE54FBED3E14D7BEB1BADCD23C9A2A79F1369400138C6D7B996731108ECE423E66F74FEF82EA53836271253BE8B7C9C3BB8FA2BD04AED2D4A1E290DFC3A0B94CACB764DD0E3ADD7C35F261FE7E27E6F928D24ED98E38883DBBA9FC0A79FB724F28ACD0D6912024CD8F83DBB01A8B6EE7102A48FF4510CDBC5B93861FD9AA55DF19E15702DC90252A8ECA97A0260D6A62CC5C4DDE926AC29CC36036D457319CC9D75AEDB2D0D253F677D72D2EEF344F4F32256FDDA1E4C0754FEA899ABB7A7F7143E512B74B80059757DEF9190B8081E6813CAEC7546E7235394BB6F997084C4D357E6230F9DB556E7F24AA020FAA29BA4478E1E9658680D91572C72F29F96D777943DDBC4697AD772BFEFB3764312400F02B59649B0678D40DF68D7E92610D8EB529C380419C9F3FD3FC3F611E41A0E2BE996F8F212F8781C30147CB5B1BFFB9EFD539B30C436605AB1446C760E73480241ED9D976C0947D74888ADCA9DF519FB10E10794E36D547EE1C81956B222455F3C189DE87A0BCE3BAB24B20696705778554AA33D818B948DC7922F3698B075EDB5724B959FE923E3896A3EF24482747325DFC94C5702954A35539E98944ED2922DC2DE42C33AF9B2630E066CB18F97F1B98D096C197A19C13948E354438EEB8138985AA0FACBDC41C7F7FDEE6077A2693BAF89BBB882C9CD62290410C75D6AC5D7BBE3E71B5F07723F5F46F2F0ED4AE26AD4B6DFF4674826BFED18476A166B71D1495FD662F4526445C337D504901296C6EF333E643FA38AAF74FA5DA020CF6338C060426905565AE362FE73067C67D812E454368987A1917F1FFB7AB86DBD891E60A6D850EE98B1CE8E4B5D2B46B60030E5CCCD141B72FC0ECDECE9511F82F854AC6D9AC546F5F68D6C0075079D992DC3030F3C6E200446DBAA25304CDA9E62673B74E77E3ED0B4CB1E22C31B6D56E996CFF81BCEEB86A38F39CFD71D428D88AA82409961C7C7CEB249A5694651E5E3B31434A088C74A0488ED3F6066043274543C969457FBB4A4340A08011C8A8596B9433DFEED9474126390CFA07DD85BA91B51AA10BA1FA9C36252F32AEAF899772EBB216102E9FEA9B3CBED983362696115FEE56B32D27B3D2CCDD38E24F2E3496EFF47DFCC5E163E021D3596B070A11DBBF70CB0AE3153B602CB58A3999694D62147C35BFC931495372766F21B3E7036DDDEAC5F7F2AEA8E7E8002F3F5A99B3C77A2DFE1CC8A90132E60BDF3AD762D51B3BEBCFA7A2DA37ECB5ADE0C07F8DCB1BE5F393EFE42644D8F12DC16C966BEF28E34C5D5C9074F1BAF37820A92A06F56E7A681FBDD12D909E409D0EAB272C2B1E725A3F7C2D7B6B43223687752606EF6556DD5CD398F73A6A062987875F6754A62666F3B9869A36BBE522AAE93CBC016C7E737DE1B736A5FA194F857128DD53FDAF23C5C6982791A7BAE6C8EC061176A741DB9FF801EDC2CA42C9725FAC2018D984A8D742";
        println!("Hex length = {}", encrypted.len());
        
        let encrypted_data = hex_string_to_bytes(encrypted).unwrap();
        println!("Binary length = {}", encrypted_data.len());
        
        // Setup Triple-DES key schedule
        let mut key_schedule = [[[0u8; 6]; 16]; 3];
        triple_des_key_setup(QQ_KEY, &mut key_schedule, DECRYPT);
        
        // Decrypt data
        let mut decrypted = vec![0u8; encrypted_data.len()];
        for (i, chunk) in encrypted_data.chunks(8).enumerate() {
            let offset = i * 8;
            if chunk.len() == 8 {
                triple_des_crypt(chunk, &mut decrypted[offset..offset + 8], &key_schedule);
            }
        }
        
        println!("Decrypted length = {}", decrypted.len());
        println!("First 32 bytes: {:02X?}", &decrypted[0..32]);
        
        // Write decrypted bytes to file for debugging
        std::fs::write("/tmp/decrypted.bin", &decrypted).unwrap();
        println!("Wrote decrypted bytes to /tmp/decrypted.bin");
        
        // Try DEFLATE decompression
        match deflate_decompress(&decrypted) {
            Ok(decompressed) => {
                println!("Decompression succeeded! {} bytes", decompressed.len());
                let s = String::from_utf8_lossy(&decompressed[0..200.min(decompressed.len())]);
                println!("First 200 chars: {}", s);
                assert!(decompressed.len() > 0, "Decompressed data should not be empty");
            }
            Err(e) => {
                panic!("Decompression failed: {}", e);
            }
        }
    }
}
