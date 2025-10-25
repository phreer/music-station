use des::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use des::Des;

fn main() {
    let qq_key = b"!@#)(*$%123ZXC!@!@#)(NHL";
    
    println!("=== Rust DES Debug Tool ===");
    println!();
    println!("QQ Key: {:02X?}", qq_key);
    println!("Key1 (0-7):   {:02X?}", &qq_key[0..8]);
    println!("Key2 (8-15):  {:02X?}", &qq_key[8..16]);
    println!("Key3 (16-23): {:02X?}", &qq_key[16..24]);
    println!();
    
    // First encrypted block from test
    let encrypted_block1 = [0x00, 0x36, 0x7F, 0xE8, 0xE5, 0x05, 0x42, 0xAB];
    println!("Block 1 encrypted: {:02X?}", encrypted_block1);
    
    // Create ciphers
    let cipher1 = Des::new_from_slice(&qq_key[0..8]).unwrap();
    let cipher2 = Des::new_from_slice(&qq_key[8..16]).unwrap();
    let cipher3 = Des::new_from_slice(&qq_key[16..24]).unwrap();
    
    // Decrypt with K3-decrypt, K2-encrypt, K1-decrypt
    let mut block = des::cipher::Block::<Des>::clone_from_slice(&encrypted_block1);
    println!("After clone:      {:02X?}", block.as_slice());
    
    cipher3.decrypt_block(&mut block);
    println!("After K3 decrypt: {:02X?}", block.as_slice());
    
    cipher2.encrypt_block(&mut block);
    println!("After K2 encrypt: {:02X?}", block.as_slice());
    
    cipher1.decrypt_block(&mut block);
    println!("After K1 decrypt: {:02X?}", block.as_slice());
    
    println!();
    println!("Expected (from C#): [78, 9C, 45, 58, DB, 6E, 55, D7]");
    println!("Got (from Rust):    {:?}", block.as_slice());
    
    if block.as_slice() == [0x78, 0x9C, 0x45, 0x58, 0xDB, 0x6E, 0x55, 0xD7] {
        println!("✓ SUCCESS: Output matches C#!");
    } else {
        println!("✗ FAILURE: Output doesn't match C#!");
    }
}
