use core::panic;

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
            }
            _ => panic!("invalid command"),
        }
    }
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
