use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `comm`
struct Cli {
    #[arg(value_name = "FILE1")]
    /// Input file 1
    file1: String,

    #[arg(value_name = "FILE2")]
    /// Input file 2
    file2: String,

    #[arg(short('1'), action=ArgAction::SetFalse)]
    /// Suppress printing of column 1 (lines unique to FILE1)
    show_col1: bool,

    #[arg(short('2'), action=ArgAction::SetFalse)]
    /// Suppress printing of column 2 (lines unique to FILE2)
    show_col2: bool,

    #[arg(short('3'), action=ArgAction::SetFalse)]
    /// Suppress printing of column 3 (lines that appear in both files)
    show_col3: bool,

    #[arg(short('i'))]
    /// Case-insensitive comparison of lines
    insensitive: bool,

    #[arg(short('d'), long("output-delimiter"), default_value_t = String::from("\t"))]
    /// Output delimiter
    delimiter: String,
}

mod helper {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};

    pub fn run(args: super::Cli) -> anyhow::Result<()> {
        if args.file1 == "-" && args.file2 == "-" {
            anyhow::bail!(r#"Both input files cannot be STDIN ("-")"#);
        }

        let fh1 = open(&args.file1)?;
        let fh2 = open(&args.file2)?;

        for (c1, c2, c3) in comm(
            fh1,
            fh2,
            args.show_col1,
            args.show_col2,
            args.show_col3,
            args.insensitive,
        ) {
            print_format(
                &c1,
                &c2,
                &c3,
                args.show_col1,
                args.show_col2,
                args.show_col3,
                &args.delimiter,
            );
        }

        Ok(())
    }

    fn open(filename: &str) -> anyhow::Result<Box<dyn BufRead>> {
        match filename {
            "-" => Ok(Box::new(BufReader::new(io::stdin()))),
            _ => Ok(Box::new(BufReader::new(
                File::open(filename).map_err(|e| anyhow::anyhow!("{}: {}", filename, e))?,
            ))),
        }
    }

    fn comm(
        file1: Box<dyn BufRead>,
        file2: Box<dyn BufRead>,
        show_col1: bool,
        show_col2: bool,
        show_col3: bool,
        insensitive: bool,
    ) -> Vec<(String, String, String)> {
        let mut lines1: Vec<String> = Vec::new();
        let mut lines2: Vec<String> = Vec::new();
        let mut out: Vec<(String, String, String)> = Vec::new();

        file1
            .lines()
            .filter(|l| l.is_ok())
            .map(|l| l.expect("Filtered values"))
            .for_each(|l| lines1.push(l.clone()));

        file2
            .lines()
            .filter(|l| l.is_ok())
            .map(|l| l.expect("Filtered values"))
            .for_each(|l| lines2.push(l.clone()));

        if lines2.len() < lines1.len() {
            lines2.iter().for_each(|l2| {
                if !lines1.iter().any(|l1| equal(&l1, &l2, insensitive)) {
                    if show_col2 {
                        out.push((String::from(""), l2.clone(), String::from("")));
                    }
                }
            });

            lines1.iter().for_each(|l1| {
                if lines2.iter().any(|l2| equal(&l1, &l2, insensitive)) {
                    if show_col3 {
                        out.push((String::from(""), String::from(""), l1.clone()));
                    }
                } else {
                    if show_col1 {
                        out.push((l1.clone(), String::from(""), String::from("")));
                    }
                }
            });
        } else {
            lines1.iter().for_each(|l1| {
                if !lines2.iter().any(|l2| equal(&l1, &l2, insensitive)) {
                    out.push((l1.clone(), String::from(""), String::from("")));
                }
            });

            lines2.iter().for_each(|l2| {
                if lines1.iter().any(|l1| equal(&l1, &l2, insensitive)) {
                    if show_col3 {
                        out.push((String::from(""), String::from(""), l2.clone()));
                    }
                } else {
                    if show_col2 {
                        out.push((String::from(""), l2.clone(), String::from("")));
                    }
                }
            });
        }
        out
    }

    fn print_format(
        col1: &str,
        col2: &str,
        col3: &str,
        show_col1: bool,
        show_col2: bool,
        show_col3: bool,
        delimiter: &str,
    ) {
        let mut output: String = String::new();

        if !col1.is_empty() && show_col1 {
            output.push_str(col1);
        } else if (!col2.is_empty() || !col3.is_empty()) && show_col1 {
            output.push_str(delimiter);
        }

        if !col2.is_empty() && show_col2 {
            output.push_str(col2);
        } else if !col3.is_empty() && show_col2 {
            output.push_str(delimiter);
        }

        if !col3.is_empty() && show_col3 {
            output.push_str(col3);
        }

        print!("{}\n", output);
    }

    fn equal(a: &str, b: &str, insensitive: bool) -> bool {
        if insensitive {
            a.to_lowercase() == b.to_lowercase()
        } else {
            a == b
        }
    }
}

fn main() {
    if let Err(e) = helper::run(Cli::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
