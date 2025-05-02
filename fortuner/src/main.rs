use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `fortune`
pub struct Cli {
    #[arg(value_name = "FILE", required = true)]
    /// Input files or directories
    sources: Vec<String>,

    #[arg(short('m'), long, value_name = "PATTERN")]
    /// Pattern
    pattern: Option<String>,

    #[arg(short, long)]
    /// Case-insensitive pattern matching
    insensitive: bool,

    #[arg(short, long, value_name = "SEED", value_parser = clap::value_parser!(u64))]
    /// Random seed
    seed: Option<u64>,
}

mod helpers {
    use std::ffi::OsStr;
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader};
    use std::path::{self, PathBuf};

    use rand::{SeedableRng, seq::IndexedRandom};

    #[derive(Debug)]
    pub struct Fortune {
        pub source: String,
        pub text: String,
    }

    pub fn run(args: crate::Cli) -> anyhow::Result<()> {
        let pattern = args
            .pattern
            .map(|val: String| {
                regex::RegexBuilder::new(&val)
                    .case_insensitive(args.insensitive)
                    .build()
                    .map_err(|_| anyhow::anyhow!(r#"Invalid --pattern "{}""#, val))
            })
            .transpose()?;

        let files = find_files(&args.sources)?;

        let fortunes = read_fortunes(&files)?;

        if fortunes.is_empty() {
            println!("No fortunes found");
            return Ok(());
        }

        match pattern {
            Some(re) => {
                let mut sources: Vec<String> = Vec::new();

                for fortune in fortunes {
                    // Print all the fortunes matching the pattern
                    if re.is_match(&fortune.text) {
                        if !sources.contains(&fortune.source) {
                            sources.push(fortune.source);
                        }
                        println!("{}", fortune.text);
                        println!("%");
                    }
                }

                for source in sources {
                    eprintln!("({})", source);
                    eprintln!("%");
                }
            }
            _ => {
                if let Some(f) = pick_fortune(&fortunes, args.seed) {
                    println!("{f}");
                }
            }
        }

        Ok(())
    }

    pub fn find_files(paths: &[String]) -> anyhow::Result<Vec<path::PathBuf>> {
        let mut files: Vec<path::PathBuf> = Vec::new();

        for path in paths {
            match fs::metadata(path) {
                Err(e) => anyhow::bail!("{path}: {e}"),
                Ok(_) => files.extend(
                    walkdir::WalkDir::new(path)
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter(|e| {
                            if let Ok(metadata) = e.metadata() {
                                e.file_type().is_file()
                                    && e.path().extension() != Some(OsStr::new("dat"))
                                    && metadata.len() > 0
                            } else {
                                false
                            }
                        })
                        .map(|e| e.path().into()),
                ),
            }
        }

        files.sort();
        files.dedup();

        Ok(files)
    }

    pub fn read_fortunes(paths: &[PathBuf]) -> anyhow::Result<Vec<Fortune>> {
        let mut output: Vec<Fortune> = Vec::new();

        for path in paths {
            let mut buf: Vec<u8> = Vec::new();
            let mut reader = BufReader::new(File::open(path)?);

            loop {
                let n = reader.read_until(b'%', &mut buf)?;

                if n == 0 {
                    break;
                }

                let source = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let mut text = String::from_utf8_lossy(&buf).into_owned();

                text.pop();
                let text = text.trim().to_string();

                if !text.is_empty() {
                    output.push(Fortune { source, text });
                }

                buf.clear();
            }
        }
        Ok(output)
    }

    pub fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
        let mut rng = match seed {
            None => rand::rngs::StdRng::from_rng(&mut rand::rng()),
            Some(state) => rand::rngs::StdRng::seed_from_u64(state),
        };

        fortunes.choose(&mut rng).map(|f| f.text.clone())
    }
}

fn main() {
    if let Err(e) = helpers::run(Cli::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::{Fortune, find_files, pick_fortune, read_fortunes};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 4);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // Parses all the fortunes without a filter
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Filters for matching text
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
