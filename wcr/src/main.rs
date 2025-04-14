use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, version = "0.1.0", author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `wc`
struct Args {
    /// Input file(s)
    #[arg(default_value = "-")]
    files: Vec<String>,

    /// Show line count
    #[arg(short('l'), long)]
    lines: bool,

    /// Show word count
    #[arg(short('w'), long)]
    words: bool,

    /// Show byte count
    #[arg(short('c'), long)]
    bytes: bool,

    /// Show character count
    #[arg(short('m'), long, conflicts_with = "bytes")]
    chars: bool,
}

mod counter_logic {
    #[derive(Debug, PartialEq, Clone)]
    pub struct FileInfo {
        pub num_lines: usize,
        pub num_words: usize,
        pub num_bytes: usize,
        pub num_chars: usize,
    }

    impl std::ops::Add<FileInfo> for FileInfo {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            FileInfo {
                num_lines: self.num_lines + rhs.num_lines,
                num_words: self.num_words + rhs.num_words,
                num_bytes: self.num_bytes + rhs.num_bytes,
                num_chars: self.num_chars + rhs.num_chars,
            }
        }
    }

    pub fn count<B>(mut file: B) -> anyhow::Result<FileInfo>
    where
        B: std::io::BufRead,
    {
        let mut num_lines = 0;
        let mut num_words = 0;
        let mut num_bytes = 0;
        let mut num_chars = 0;
        let mut line = String::new();

        loop {
            if file.read_line(&mut line)? == 0 {
                break;
            }

            num_lines = num_lines + 1;
            num_words = num_words + line.split_whitespace().count();
            num_bytes = num_bytes + line.bytes().count();
            num_chars = num_chars + line.chars().count();

            line.clear();
        }

        Ok(FileInfo {
            num_lines,
            num_words,
            num_bytes,
            num_chars,
        })
    }
}

mod helper {
    /// Helper function that encapsulte the main logic of the command line tool.
    pub fn run(args: &crate::Args) -> anyhow::Result<()> {
        let args = logic_arg(args);
        let mut files_info = crate::counter_logic::FileInfo {
            num_lines: 0,
            num_words: 0,
            num_bytes: 0,
            num_chars: 0,
        };

        for filename in args.files.iter() {
            match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(handler) => {
                    let file_info = crate::counter_logic::count(handler)?;

                    files_info = files_info + file_info.clone();

                    print_result(
                        &file_info, filename, args.lines, args.words, args.chars, args.bytes,
                    )?;
                }
            }
        }

        if args.files.len() > 1 {
            print_result(
                &files_info,
                "total",
                args.lines,
                args.words,
                args.chars,
                args.bytes,
            )?;
        }

        Ok(())
    }

    /// Helper function that manages creating handlers to be processed.
    fn open(filename: &str) -> anyhow::Result<Box<dyn std::io::BufRead>> {
        match filename {
            "-" => Ok(Box::new(std::io::BufReader::new(std::io::stdin()))),
            _ => Ok(Box::new(std::io::BufReader::new(std::fs::File::open(
                filename,
            )?))),
        }
    }

    /// Helper function that expands the logic of the crate clap to the needs of
    /// the command line tool.
    fn logic_arg(args: &crate::Args) -> crate::Args {
        if !(args.lines || args.words || args.chars || args.bytes) {
            crate::Args {
                files: args.files.clone(),
                lines: true,
                words: true,
                bytes: true,
                chars: false,
            }
        } else {
            crate::Args {
                files: args.files.clone(),
                lines: args.lines,
                words: args.words,
                bytes: args.bytes,
                chars: args.chars,
            }
        }
    }

    /// Helper function to print results in the format that we want.
    fn print_result(
        input: &crate::counter_logic::FileInfo,
        filename: &str,
        line: bool,
        word: bool,
        chars: bool,
        bytes: bool,
    ) -> anyhow::Result<()> {
        let mut result = String::new();

        if line {
            result.push_str(&format!("{:>8}", input.num_lines));
        }

        if word {
            result.push_str(&format!("{:>8}", input.num_words));
        }

        if chars {
            result.push_str(&format!("{:>8}", input.num_chars));
        }

        if bytes {
            result.push_str(&format!("{:>8}", input.num_bytes));
        }

        if filename != "-" {
            result.push_str(&format!(" {}", filename));
        }

        println!("{}", result);

        Ok(())
    }
}

fn main() {
    if let Err(e) = helper::run(&Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use crate::counter_logic;
    use std::io;

    #[test]
    fn test_count() {
        let text = "I don't want the world.\nI just want your half.\r\n";
        let info = counter_logic::count(io::Cursor::new(text));
        assert!(info.is_ok());
        let expected = counter_logic::FileInfo {
            num_lines: 2,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
