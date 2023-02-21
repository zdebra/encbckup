use cryptostream::read;
use openssl::symm::Cipher;
use std::io::Read;

pub struct Cryptography {
    pub key: Vec<u8>,
    pub iv: Vec<u8>,
}

pub fn encrypt<R: std::io::Read, W: std::io::Write>(
    r: &mut R,
    w: &mut W,
    c: &Cryptography,
) -> Result<(), String> {
    let mut encryptor = read::Encryptor::new(r, Cipher::aes_128_cbc(), &c.key, &c.iv).unwrap();

    const BUFFER_LEN: usize = 128;
    let mut buffer = [0u8; BUFFER_LEN];

    loop {
        match encryptor.read(&mut buffer) {
            Err(e) => {
                if let std::io::ErrorKind::Interrupted = e.kind() {
                    continue;
                }
                return Err(e.to_string());
            }
            Ok(read_count) => {
                if read_count == 0 {
                    if let Err(e) = w.flush() {
                        return Err(e.to_string());
                    }
                    break;
                }
                if let Err(e) = w.write_all(&buffer) {
                    return Err(e.to_string());
                }
            }
        }
    }
    Ok(())
}

pub fn decrypt<R: std::io::Read, W: std::io::Write>(
    r: &mut R,
    w: &mut W,
    c: &Cryptography,
) -> Result<(), String> {
    let mut decryptor = read::Decryptor::new(r, Cipher::aes_128_cbc(), &c.key, &c.iv).unwrap();

    const BUFFER_LEN: usize = 128;
    let mut buffer = [0u8; BUFFER_LEN];

    loop {
        match decryptor.read(&mut buffer) {
            Err(e) => {
                if let std::io::ErrorKind::Interrupted = e.kind() {
                    continue;
                }
                return Err(e.to_string());
            }
            Ok(read_count) => {
                if read_count == 0 {
                    if let Err(e) = w.flush() {
                        return Err(e.to_string());
                    }
                    break;
                }
                if let Err(e) = w.write_all(&buffer) {
                    return Err(e.to_string());
                }
            }
        }
    }
    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = vec![20, 30, 40];
        let mut buf = Vec::new();

        let key: Vec<_> = base64::decode("kjtbxCPw3XPFThb3mKmzfg==").unwrap();
        let iv: Vec<_> = base64::decode("dB0Ej+7zWZWTS5JUCldWMg==").unwrap();

        let c = Cryptography { key, iv };
        let res = encrypt(&mut data.as_slice(), &mut buf, &c);
        assert!(res.is_ok());

        let mut out = Vec::new();
        let res = decrypt(&mut buf.as_slice(), &mut out, &c);
        assert!(res.is_ok());

        assert_eq!(data, out);
    }
}
