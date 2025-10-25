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
    (((a << b) & 0x80000000) >> c)
}

fn sboxbit(a: u8) -> usize {
    (((a & 0x20) | ((a & 0x1f) >> 1) | ((a & 0x01) << 4)) as usize)
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

fn f(mut state: u32, key: &[u8; 6]) -> u32 {
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

fn crypt(input: &[u8], output: &mut [u8], key: &[[u8; 6]; 16]) {
    let mut state = [0u32; 2];

    ip(&mut state, input);

    for idx in 0..15 {
        let t = state[1];
        state[1] = f(state[1], &key[idx]) ^ state[0];
        state[0] = t;
    }

    state[0] = f(state[1], &key[15]) ^ state[0];

    inv_ip(&state, output);
}

pub fn triple_des_key_setup(key: &[u8], schedule: &mut [[[u8; 6]; 16]; 3], mode: u32) {
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

pub fn triple_des_crypt(input: &[u8], output: &mut [u8], key: &[[[u8; 6]; 16]; 3]) {
    let mut temp = [0u8; 8];
    crypt(input, output, &key[0]);
    temp.copy_from_slice(output);
    crypt(&temp, output, &key[1]);
    temp.copy_from_slice(output);
    crypt(&temp, output, &key[2]);
}

fn main() {
    let qq_key = b"!@#)(*$%123ZXC!@!@#)(NHL";
    
    println!("=== Rust Custom DES Debug Tool ===");
    println!();
    println!("QQ Key: {:02X?}", qq_key);
    println!();
    
    // First encrypted block from test
    let encrypted_block1 = [0x00, 0x36, 0x7F, 0xE8, 0xE5, 0x05, 0x42, 0xAB];
    println!("Block 1 encrypted: {:02X?}", encrypted_block1);
    
    // Setup key schedule
    let mut schedule = [[[0u8; 6]; 16]; 3];
    triple_des_key_setup(qq_key, &mut schedule, DECRYPT);
    println!("Key schedule setup complete (DECRYPT mode)");
    
    // Decrypt
    let mut decrypted = [0u8; 8];
    triple_des_crypt(&encrypted_block1, &mut decrypted, &schedule);
    
    println!("Decrypted:          {:02X?}", decrypted);
    println!();
    println!("Expected (from C#): [78, 9C, 45, 58, DB, 6E, 55, D7]");
    println!("Got (from Rust):    {:?}", decrypted);
    
    if decrypted == [0x78, 0x9C, 0x45, 0x58, 0xDB, 0x6E, 0x55, 0xD7] {
        println!("✓ SUCCESS: Output matches C#!");
    } else {
        println!("✗ FAILURE: Output doesn't match C#!");
    }
}
