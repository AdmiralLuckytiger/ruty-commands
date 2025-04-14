use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `grep`
struct Cli {
    #[arg(required = true)]
    /// Search pattern
    pattern: String,

    #[arg(default_values=["-"], num_args=0.., value_name="FILE")]
    /// Input file(s)
    files: Vec<String>,

    #[arg(short, long)]
    /// Case-insensitive
    insensitive: bool,

    #[arg(short, long)]
    /// Recursive search
    recursive: bool,

    #[arg(short, long)]
    /// Count occurences
    count: bool,

    #[arg(short('v'), long("invert-match"))]
    /// Invert match
    invert: bool,
}

mod helper {
    use regex::Regex;
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};
    use walkdir::WalkDir;

    pub fn run(args: crate::Cli) -> anyhow::Result<()> {
        let pattern = regex::RegexBuilder::new(&args.pattern)
            .case_insensitive(args.insensitive)
            .build()
            .map_err(|_| anyhow::anyhow!(r#"Invalid pattern "{}""#, args.pattern))?;

        let entries = find_files(&args.files, args.recursive);

        for entry in entries {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(filename) => match open(&filename) {
                    Err(e) => eprintln!("{}: {}", filename, e),
                    Ok(file) => {
                        let matches = find_lines(file, &pattern, args.invert);

                        if args.count {
                            print_output(
                                &args,
                                &filename,
                                &format!("{}\n", matches?.iter().count()),
                            );
                        } else {
                            matches?.iter().for_each(|line| {
                                if !line.is_empty() {
                                    print_output(&args, &filename, &line);
                                }
                            });
                        }
                    }
                },
            }
        }

        Ok(())
    }

    fn print_output(args: &crate::Cli, filename: &str, out: &str) {
        if args.files.len() > 1 || args.recursive {
            print!("{}:{}", filename, out);
        } else {
            print!("{}", out);
        }
    }

    pub fn find_lines<T: BufRead>(
        mut file: T,
        re: &Regex,
        invert: bool,
    ) -> anyhow::Result<Vec<String>> {
        let mut out: Vec<String> = Vec::new();
        let mut hay = String::new();

        loop {
            match file.read_line(&mut hay) {
                Err(e) => {
                    return Err(anyhow::anyhow!("{}", e));
                }
                Ok(n) => {
                    if n == 0 {
                        break;
                    }

                    if re.is_match(&hay) ^ invert {
                        out.push(hay.clone());
                    }

                    hay.clear();
                }
            }
        }

        Ok(out)
    }

    pub fn find_files(paths: &[String], recursive: bool) -> Vec<anyhow::Result<String>> {
        let mut out: Vec<anyhow::Result<String>> = Vec::new();

        for path in paths {
            if path == "-" {
                out.push(Ok("-".to_string()));
            } else {
                for (i, entry) in WalkDir::new(path).into_iter().enumerate() {
                    match entry {
                        Err(e) => {
                            out.push(Err(anyhow::anyhow!("{}: {}", path, e)));
                        }
                        Ok(e) => {
                            if i == 0 && !recursive && e.file_type().is_dir() {
                                out.push(Err(anyhow::anyhow!(
                                    "{} is a directory",
                                    e.path().display()
                                )));
                                break;
                            }

                            if e.file_type().is_file() {
                                out.push(Ok(e.path().display().to_string()));
                            }
                        }
                    }
                }
            }
        }

        out
    }

    fn open(filename: &str) -> anyhow::Result<Box<dyn BufRead>> {
        match filename {
            "-" => Ok(Box::new(BufReader::new(io::stdin()))),
            _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
        }
    }
}

fn main() {
    if let Err(e) = helper::run(Cli::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use crate::helper::*;
    use pretty_assertions::assert_eq;
    use rand::{distr::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        println!("First test");
        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        println!("Second test");
        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        println!("Third test");
        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        println!("Fourth test");
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        println!("Fifth test");
        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
