mod crypt;

use crate::crypt::{decrypt, encrypt, Cryptography};
use std::{fs::File, io::Write, path::PathBuf};

fn main() {
    Cli::from_args(std::env::args().skip(1).collect());
}

struct Cli {
    key: Vec<u8>,
    iv: Vec<u8>,
}

impl Cli {
    fn from_args(args: Vec<String>) {
        let mut args = args.into_iter();
        let command = args.next().expect("command is missing");

        let key: Vec<_> = base64::decode("kjtbxCPw3XPFThb3mKmzfg==").unwrap();
        let iv: Vec<_> = base64::decode("dB0Ej+7zWZWTS5JUCldWMg==").unwrap();

        match command.as_str() {
            "backup" => {
                let path = args.next().expect("path to backup is missing");
                let path = std::path::PathBuf::from(path);

                if !path.try_exists().expect("a valid path") {
                    panic!("path `{}` doesn't exist", path.display())
                }

                let cli = Self { key, iv };
                if let Err(err) = cli.backup(path) {
                    panic!("backup failed: {}", err);
                }
                println!("backup was successful!");
            }
            "restore" => {
                let remote_path = args.next().expect("remote path to restore is missing");
                let remote_path = std::path::PathBuf::from(remote_path);

                let cli = Self { key, iv };
                if let Err(err) = cli.restore(remote_path) {
                    panic!("restore failed: {}", err);
                }
                println!("restore successful");
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

    fn restore(&self, remote_path: PathBuf) -> Result<(), String> {
        if !remote_path.is_file() {
            panic!("currently handling files only!")
        }
        let mut file_origin = std::fs::File::open(remote_path.clone()).unwrap();
        let mut decrypted_bytes = Vec::new();
        if let Err(e) = self.decrypt(&mut file_origin, &mut decrypted_bytes) {
            return Err(format!("decrypt failed: {}", e));
        }
        println!("decrypt OK!");

        // let out_path = remove_suffix(remote_path, ".bkp");
        let out_path = path_append(remote_path.clone(), ".ubkp");
        let mut dec_out = File::create(out_path).unwrap();

        if let Err(e) = self.decompress(&mut decrypted_bytes.as_slice(), &mut dec_out) {
            return Err(format!("decompress failed: {}", e));
        }
        println!("decompress OK!");

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

    fn decompress<R: std::io::Read, W: std::io::Write>(
        &self,
        r: &mut R, /* compressed bytes */
        w: &mut W, /* decompressed bytes */
    ) -> Result<(), String> {
        let mut writer = brotli::DecompressorWriter::new(w, 4096 /* buffer size */);

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
        return encrypt(
            r,
            w,
            &Cryptography {
                key: self.key.clone(),
                iv: self.iv.clone(),
            },
        );
    }

    fn decrypt<R: std::io::Read, W: std::io::Write>(
        &self,
        r: &mut R, /* encrypted bytes */
        w: &mut W, /* decrypted bytes */
    ) -> Result<(), String> {
        return decrypt(
            r,
            w,
            &Cryptography {
                key: self.key.clone(),
                iv: self.iv.clone(),
            },
        );
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

fn remove_suffix(path: PathBuf, suffix: &str) -> PathBuf {
    let last_item = path
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

    let last_item = last_item.strip_suffix(suffix).unwrap_or(last_item);
    path.extend(vec![last_item]);
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
