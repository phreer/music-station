use crate::error::{MusicSearchError, Result};
use flate2::read::ZlibDecoder;
use std::io::Read;

const QQ_KEY: &[u8] = b"!@#)(*$%123ZXC!@!@#)(NHL";

/// Decrypt QRC lyrics from QQ Music
pub fn decrypt_lyrics(encrypted_lyrics: &str) -> Result<String> {
    // Parse hex string to bytes
    let encrypted_bytes = hex_string_to_bytes(encrypted_lyrics)?;
    
    let mut data = vec![0u8; encrypted_bytes.len()];
    let mut schedule = vec![vec![vec![0u8; 6]; 16]; 3];
    
    triple_des_key_setup(QQ_KEY, &mut schedule, false); // false = DECRYPT
    
    // Decrypt in 8-byte blocks
    for i in (0..encrypted_bytes.len()).step_by(8) {
        let mut temp = [0u8; 8];
        triple_des_crypt(&encrypted_bytes[i..], &mut temp, &schedule);
        data[i..i+8].copy_from_slice(&temp);
    }
    
    // Decompress
    let unzipped = zlib_decompress(&data)?;
    
    // Convert to UTF-8 string
    let result = String::from_utf8(unzipped)
        .map_err(|e| MusicSearchError::Other(format!("UTF-8 decode error: {}", e)))?;
    
    Ok(result)
}

fn hex_string_to_bytes(hex_string: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(hex_string.len() / 2);
    for i in (0..hex_string.len()).step_by(2) {
        if i + 2 > hex_string.len() {
            break;
        }
        let byte = u8::from_str_radix(&hex_string[i..i+2], 16)
            .map_err(|e| MusicSearchError::Other(format!("Hex parse error: {}", e)))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

// S-boxes for DES
const SBOX1: &[u8] = &[
    14,  4, 13,  1,  2, 15, 11,  8,  3, 10,  6, 12,  5,  9,  0,  7,
     0, 15,  7,  4, 14,  2, 13,  1, 10,  6, 12, 11,  9,  5,  3,  8,
     4,  1, 14,  8, 13,  6,  2, 11, 15, 12,  9,  7,  3, 10,  5,  0,
    15, 12,  8,  2,  4,  9,  1,  7,  5, 11,  3, 14, 10,  0,  6, 13
];

const SBOX2: &[u8] = &[
    15,  1,  8, 14,  6, 11,  3,  4,  9,  7,  2, 13, 12,  0,  5, 10,
     3, 13,  4,  7, 15,  2,  8, 15, 12,  0,  1, 10,  6,  9, 11,  5,
     0, 14,  7, 11, 10,  4, 13,  1,  5,  8, 12,  6,  9,  3,  2, 15,
    13,  8, 10,  1,  3, 15,  4,  2, 11,  6,  7, 12,  0,  5, 14,  9
];

const SBOX3: &[u8] = &[
    10,  0,  9, 14,  6,  3, 15,  5,  1, 13, 12,  7, 11,  4,  2,  8,
    13,  7,  0,  9,  3,  4,  6, 10,  2,  8,  5, 14, 12, 11, 15,  1,
    13,  6,  4,  9,  8, 15,  3,  0, 11,  1,  2, 12,  5, 10, 14,  7,
     1, 10, 13,  0,  6,  9,  8,  7,  4, 15, 14,  3, 11,  5,  2, 12
];

const SBOX4: &[u8] = &[
     7, 13, 14,  3,  0,  6,  9, 10,  1,  2,  8,  5, 11, 12,  4, 15,
    13,  8, 11,  5,  6, 15,  0,  3,  4,  7,  2, 12,  1, 10, 14,  9,
    10,  6,  9,  0, 12, 11,  7, 13, 15,  1,  3, 14,  5,  2,  8,  4,
     3, 15,  0,  6, 10, 10, 13,  8,  9,  4,  5, 11, 12,  7,  2, 14
];

const SBOX5: &[u8] = &[
     2, 12,  4,  1,  7, 10, 11,  6,  8,  5,  3, 15, 13,  0, 14,  9,
    14, 11,  2, 12,  4,  7, 13,  1,  5,  0, 15, 10,  3,  9,  8,  6,
     4,  2,  1, 11, 10, 13,  7,  8, 15,  9, 12,  5,  6,  3,  0, 14,
    11,  8, 12,  7,  1, 14,  2, 13,  6, 15,  0,  9, 10,  4,  5,  3
];

const SBOX6: &[u8] = &[
    12,  1, 10, 15,  9,  2,  6,  8,  0, 13,  3,  4, 14,  7,  5, 11,
    10, 15,  4,  2,  7, 12,  9,  5,  6,  1, 13, 14,  0, 11,  3,  8,
     9, 14, 15,  5,  2,  8, 12,  3,  7,  0,  4, 10,  1, 13, 11,  6,
     4,  3,  2, 12,  9,  5, 15, 10, 11, 14,  1,  7,  6,  0,  8, 13
];

const SBOX7: &[u8] = &[
     4, 11,  2, 14, 15,  0,  8, 13,  3, 12,  9,  7,  5, 10,  6,  1,
    13,  0, 11,  7,  4,  9,  1, 10, 14,  3,  5, 12,  2, 15,  8,  6,
     1,  4, 11, 13, 12,  3,  7, 14, 10, 15,  6,  8,  0,  5,  9,  2,
     6, 11, 13,  8,  1,  4, 10,  7,  9,  5,  0, 15, 14,  2,  3, 12
];

const SBOX8: &[u8] = &[
    13,  2,  8,  4,  6, 15, 11,  1, 10,  9,  3, 14,  5,  0, 12,  7,
     1, 15, 13,  8, 10,  3,  7,  4, 12,  5,  6, 11,  0, 14,  9,  2,
     7, 11,  4,  1,  9, 12, 14,  2,  0,  6, 10, 13, 15,  3,  5,  8,
     2,  1, 14,  7,  4, 10,  8, 13, 15, 12,  9,  0,  3,  5,  6, 11
];

fn bitnum(a: &[u8], b: usize, c: usize) -> u32 {
    (((a[b / 32 * 4 + 3 - b % 32 / 8] >> (7 - b % 8)) & 0x01) as u32) << c
}

fn bitnumintr(a: u32, b: usize, c: usize) -> u8 {
    (((a >> (31 - b)) & 0x00000001) << c) as u8
}

fn bitnumintl(a: u32, b: usize, c: usize) -> u32 {
    (((a << b) & 0x80000000) >> c)
}

fn sboxbit(a: u8) -> usize {
    (((a & 0x20) | ((a & 0x1f) >> 1) | ((a & 0x01) << 4))) as usize
}

fn key_schedule(key: &[u8], schedule: &mut Vec<Vec<u8>>, decrypt: bool) {
    let key_rnd_shift = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];
    let key_perm_c = [56, 48, 40, 32, 24, 16, 8, 0, 57, 49, 41, 33, 25, 17,
        9, 1, 58, 50, 42, 34, 26, 18, 10, 2, 59, 51, 43, 35];
    let key_perm_d = [62, 54, 46, 38, 30, 22, 14, 6, 61, 53, 45, 37, 29, 21,
        13, 5, 60, 52, 44, 36, 28, 20, 12, 4, 27, 19, 11, 3];
    let key_compression = [13, 16, 10, 23, 0, 4, 2, 27, 14, 5, 20, 9,
        22, 18, 11, 3, 25, 7, 15, 6, 26, 19, 12, 1,
        40, 51, 30, 36, 46, 54, 29, 39, 50, 44, 32, 47,
        43, 48, 38, 55, 33, 52, 45, 41, 49, 35, 28, 31];

    let mut c = 0u32;
    let mut d = 0u32;

    for i in 0..28 {
        c |= bitnum(key, key_perm_c[i], 31 - i);
        d |= bitnum(key, key_perm_d[i], 31 - i);
    }

    for i in 0..16 {
        c = ((c << key_rnd_shift[i]) | (c >> (28 - key_rnd_shift[i]))) & 0xfffffff0;
        d = ((d << key_rnd_shift[i]) | (d >> (28 - key_rnd_shift[i]))) & 0xfffffff0;

        let to_gen = if decrypt { 15 - i } else { i };

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

fn des_f(state: u32, key: &[u8]) -> u32 {
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

    for i in 0..6 {
        lrgstate[i] ^= key[i];
    }

    let result = (SBOX1[sboxbit(lrgstate[0] >> 2)] as u32) << 28 |
                 (SBOX2[sboxbit(((lrgstate[0] & 0x03) << 4) | (lrgstate[1] >> 4))] as u32) << 24 |
                 (SBOX3[sboxbit(((lrgstate[1] & 0x0f) << 2) | (lrgstate[2] >> 6))] as u32) << 20 |
                 (SBOX4[sboxbit(lrgstate[2] & 0x3f)] as u32) << 16 |
                 (SBOX5[sboxbit(lrgstate[3] >> 2)] as u32) << 12 |
                 (SBOX6[sboxbit(((lrgstate[3] & 0x03) << 4) | (lrgstate[4] >> 4))] as u32) << 8 |
                 (SBOX7[sboxbit(((lrgstate[4] & 0x0f) << 2) | (lrgstate[5] >> 6))] as u32) << 4 |
                 SBOX8[sboxbit(lrgstate[5] & 0x3f)] as u32;

    bitnumintl(result, 15, 0) | bitnumintl(result, 6, 1) | bitnumintl(result, 19, 2) |
    bitnumintl(result, 20, 3) | bitnumintl(result, 28, 4) | bitnumintl(result, 11, 5) |
    bitnumintl(result, 27, 6) | bitnumintl(result, 16, 7) | bitnumintl(result, 0, 8) |
    bitnumintl(result, 14, 9) | bitnumintl(result, 22, 10) | bitnumintl(result, 25, 11) |
    bitnumintl(result, 4, 12) | bitnumintl(result, 17, 13) | bitnumintl(result, 30, 14) |
    bitnumintl(result, 9, 15) | bitnumintl(result, 1, 16) | bitnumintl(result, 7, 17) |
    bitnumintl(result, 23, 18) | bitnumintl(result, 13, 19) | bitnumintl(result, 31, 20) |
    bitnumintl(result, 26, 21) | bitnumintl(result, 2, 22) | bitnumintl(result, 8, 23) |
    bitnumintl(result, 18, 24) | bitnumintl(result, 12, 25) | bitnumintl(result, 29, 26) |
    bitnumintl(result, 5, 27) | bitnumintl(result, 21, 28) | bitnumintl(result, 10, 29) |
    bitnumintl(result, 3, 30) | bitnumintl(result, 24, 31)
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
                 bitnumintr(state[1], 31, 1) | bitnumintr(state[0], 31, 0));
    
    output[2] = (bitnumintr(state[1], 6, 7) | bitnumintr(state[0], 6, 6) | bitnumintr(state[1], 14, 5) |
                 bitnumintr(state[0], 14, 4) | bitnumintr(state[1], 22, 3) | bitnumintr(state[0], 22, 2) |
                 bitnumintr(state[1], 30, 1) | bitnumintr(state[0], 30, 0));
    
    output[1] = (bitnumintr(state[1], 5, 7) | bitnumintr(state[0], 5, 6) | bitnumintr(state[1], 13, 5) |
                 bitnumintr(state[0], 13, 4) | bitnumintr(state[1], 21, 3) | bitnumintr(state[0], 21, 2) |
                 bitnumintr(state[1], 29, 1) | bitnumintr(state[0], 29, 0));
    
    output[0] = (bitnumintr(state[1], 4, 7) | bitnumintr(state[0], 4, 6) | bitnumintr(state[1], 12, 5) |
                 bitnumintr(state[0], 12, 4) | bitnumintr(state[1], 20, 3) | bitnumintr(state[0], 20, 2) |
                 bitnumintr(state[1], 28, 1) | bitnumintr(state[0], 28, 0));
    
    output[7] = (bitnumintr(state[1], 3, 7) | bitnumintr(state[0], 3, 6) | bitnumintr(state[1], 11, 5) |
                 bitnumintr(state[0], 11, 4) | bitnumintr(state[1], 19, 3) | bitnumintr(state[0], 19, 2) |
                 bitnumintr(state[1], 27, 1) | bitnumintr(state[0], 27, 0));
    
    output[6] = (bitnumintr(state[1], 2, 7) | bitnumintr(state[0], 2, 6) | bitnumintr(state[1], 10, 5) |
                 bitnumintr(state[0], 10, 4) | bitnumintr(state[1], 18, 3) | bitnumintr(state[0], 18, 2) |
                 bitnumintr(state[1], 26, 1) | bitnumintr(state[0], 26, 0));
    
    output[5] = (bitnumintr(state[1], 1, 7) | bitnumintr(state[0], 1, 6) | bitnumintr(state[1], 9, 5) |
                 bitnumintr(state[0], 9, 4) | bitnumintr(state[1], 17, 3) | bitnumintr(state[0], 17, 2) |
                 bitnumintr(state[1], 25, 1) | bitnumintr(state[0], 25, 0));
    
    output[4] = (bitnumintr(state[1], 0, 7) | bitnumintr(state[0], 0, 6) | bitnumintr(state[1], 8, 5) |
                 bitnumintr(state[0], 8, 4) | bitnumintr(state[1], 16, 3) | bitnumintr(state[0], 16, 2) |
                 bitnumintr(state[1], 24, 1) | bitnumintr(state[0], 24, 0));
}

fn des_crypt(input: &[u8], output: &mut [u8], key: &Vec<Vec<u8>>) {
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

fn triple_des_key_setup(key: &[u8], schedule: &mut Vec<Vec<Vec<u8>>>, decrypt: bool) {
    if decrypt {
        key_schedule(&key[0..8], &mut schedule[2], decrypt);
        key_schedule(&key[8..16], &mut schedule[1], !decrypt);
        key_schedule(&key[16..24], &mut schedule[0], decrypt);
    } else {
        key_schedule(&key[0..8], &mut schedule[0], decrypt);
        key_schedule(&key[8..16], &mut schedule[1], !decrypt);
        key_schedule(&key[16..24], &mut schedule[2], decrypt);
    }
}

fn triple_des_crypt(input: &[u8], output: &mut [u8], key: &Vec<Vec<Vec<u8>>>) {
    let mut temp1 = [0u8; 8];
    let mut temp2 = [0u8; 8];
    
    des_crypt(input, &mut temp1, &key[0]);
    des_crypt(&temp1, &mut temp2, &key[1]);
    des_crypt(&temp2, output, &key[2]);
}
