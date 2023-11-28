pub fn xorcrypt(content: &mut [u8], key: &[u8; 20]) {
    for i in 0..content.len() {
        content[i] ^= key[i % key.len()];
    }
}
