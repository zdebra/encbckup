use anyhow::anyhow;
use chacha20poly1305::{
    aead::{stream, Aead, NewAead},
    XChaCha20Poly1305,
};

use rand::{rngs::OsRng, RngCore};
use std::{
    fs::{self, File},
    io::{Cursor, Read, Write},
    path::PathBuf,
};

fn main() {
    Cli::from_args(std::env::args().skip(1).collect());
}

struct Cli {
    // path: std::path::PathBuf,
}

impl Cli {
    fn from_args(args: Vec<String>) {
        let mut args = args.into_iter();
        let command = args.next().expect("command is missing");
        match command.as_str() {
            "backup" => {
                let path = args.next().expect("path to backup is missing");
                let path = std::path::PathBuf::from(path);

                if !path.try_exists().expect("a valid path") {
                    panic!("path `{}` doesn't exist", path.display())
                }

                let cli = Self {};
                if let Err(err) = cli.backup(path) {
                    panic!("backup failed: {}", err);
                }
                println!("backup was successful!");
            }
            _ => panic!("invalid command"),
        }
    }

    fn backup(&self, path: std::path::PathBuf) -> Result<(), String> {
        if !path.is_file() {
            panic!("currently handling files only!")
        }
        let mut file_origin = std::fs::File::open(path.clone()).unwrap();
        let mut compressed_bytes = Vec::new();
        match self.compress(&mut file_origin, &mut compressed_bytes) {
            Err(e) => panic!("{}", e),
            Ok(_) => {
                println!("compression OK!");
            }
        }

        let out_path = path_append(path.clone(), ".bkp");
        let mut enc_out = File::create(out_path).unwrap();

        match self.encrypt(&mut compressed_bytes.as_slice(), &mut enc_out) {
            Err(e) => panic!("{}", e),
            Ok(_) => {
                println!("encryption OK!")
            }
        }

        Ok(())
    }

    fn compress<R: std::io::Read, W: std::io::Write>(
        &self,
        r: &mut R,
        w: &mut W,
    ) -> Result<(), String> {
        let mut writer = brotli::CompressorWriter::new(
            w, 4096, /* buffer size */
            9,    /* compression levels 0-9 */
            20,   /* recommended lg_window_size is between 20 and 22 */
        );

        let mut buf = [0u8; 4096];
        loop {
            match r.read(&mut buf) {
                Err(e) => {
                    if let std::io::ErrorKind::Interrupted = e.kind() {
                        continue;
                    }
                    return Err(e.to_string());
                }
                Ok(size) => {
                    if size == 0 {
                        match writer.flush() {
                            Err(e) => {
                                if let std::io::ErrorKind::Interrupted = e.kind() {
                                    continue;
                                }
                                return Err(e.to_string());
                            }
                            Ok(_) => break,
                        }
                    }
                    match writer.write_all(&buf[..size]) {
                        Err(e) => return Err(e.to_string()),
                        Ok(_) => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn encrypt<R: std::io::Read, W: std::io::Write>(
        &self,
        r: &mut R,
        w: &mut W,
    ) -> Result<(), String> {
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 19];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);

        let aead = XChaCha20Poly1305::new(key.as_ref().into());
        let mut stream_encryptor = stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into());

        const BUFFER_LEN: usize = 500;
        let mut buffer = [0u8; BUFFER_LEN];

        loop {
            match r.read(&mut buffer) {
                Err(e) => {
                    if let std::io::ErrorKind::Interrupted = e.kind() {
                        continue;
                    }
                    return Err(e.to_string());
                }
                Ok(read_count) => {
                    if read_count == BUFFER_LEN {
                        let ciphertext = stream_encryptor.encrypt_next(buffer.as_slice()).unwrap();
                        if let Err(e) = w.write(&ciphertext) {
                            return Err(e.to_string());
                        }
                    } else {
                        let ciphertext = stream_encryptor
                            .encrypt_last(&buffer[..read_count])
                            .unwrap();
                        if let Err(e) = w.write(&ciphertext) {
                            return Err(e.to_string());
                        }
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn upload(&self) {}
}

fn path_append(path: PathBuf, to_append: &str) -> PathBuf {
    let filename = path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();
    let mut it = path.components();
    it.next_back().unwrap(); // remove last
    let mut path = std::path::PathBuf::new();
    path.extend(it);
    path.extend(vec![filename.to_owned() + to_append]);
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "invalid command")]
    fn test_from_args_invalid_command() {
        Cli::from_args(vec!["}".to_string()]);
    }

    #[test]
    #[should_panic(expected = "path `}` doesn't exist")]
    fn test_from_args_invalid_path() {
        Cli::from_args(vec!["backup".to_string(), "}".to_string()]);
    }

    #[test]
    #[should_panic(expected = "path `/lala/tralala` doesn't exist")]
    fn test_from_args_nonexisting_path() {
        Cli::from_args(vec!["backup".to_string(), "/lala/tralala".to_string()]);
    }

    #[test]
    fn test_from_args() {
        Cli::from_args(vec!["backup".to_string(), "src".to_string()]);
    }
}
