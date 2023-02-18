use core::panic;
use std::io::{BufWriter, Write};

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
        let file_origin = std::fs::File::open(path.clone()).unwrap();
        let output_path = {
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
            path.extend(vec![filename.to_owned() + ".bkp"]);
            path
        };

        let file_output = std::fs::File::create(output_path).unwrap();

        match self.compress(file_origin, file_output) {
            Err(e) => panic!("{}", e),
            Ok(_) => {
                println!("hehe");
            }
        }

        Ok(())
    }

    fn compress<R: std::io::Read, W: std::io::Write>(&self, mut r: R, w: W) -> Result<(), String> {
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

    fn encrypt(&self) {}

    fn upload(&self) {}
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
