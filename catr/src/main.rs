use clap::Parser;

#[derive(Debug, Parser)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
#[command(version, about)]
/// Rust version of `cat`
struct Args {
    /// Input file(s)
    #[arg(required(true))]
    files: Vec<String>,
    /// Number lines
    #[arg(short('n'), long("number"))]
    number_lines: bool,
    /// Number non-blanck lines
    #[arg(short('b'), long("number-nonblank"), conflicts_with = "number_lines")]
    number_nonblank_lines: bool,
}

mod helpers {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};

    /// Method for performing the main logic of the command-line.
    pub fn run(args: &crate::Args) -> anyhow::Result<()> {
        args.files.iter().for_each(|file| match open(file) {
            Err(err) => eprintln!("Failed to open {}: {}", file, err),
            Ok(handler) => {
                if args.number_lines {
                    let _ = read(handler, |x, i| println!("{:>6}\t{}", i + 1, x));
                } else if args.number_nonblank_lines {
                    let _ = read_b(handler);
                } else {
                    let _ = read(handler, |x, _i| println!("{}", x));
                }
            }
        });
        Ok(())
    }

    /// Private function for dealing the different kinds of files that could
    /// be read. (Until now Stdin and File)
    /// The only condition to open a file is that implements the trait BufRead.
    fn open(filename: &str) -> anyhow::Result<Box<dyn BufRead>> {
        match filename {
            "-" => Ok(Box::new(BufReader::new(io::stdin()))),
            _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
        }
    }

    /// Private function for printing in different formats the text inside the files.
    /// The logic of printing is define by the closure.
    fn read<F>(handler: Box<dyn BufRead>, f: F) -> anyhow::Result<()>
    where
        F: Fn(&str, &usize),
    {
        handler
            .lines()
            .enumerate()
            // For failing lines read we opt for passing an empty string,
            // the error is rare and the alternative is too much aggresive.
            .for_each(|(i, l)| f(&l.unwrap_or(String::from("")), &i));
        Ok(())
    }

    /// Private function for printing the text of the files for the special case of non-blanks
    /// numbering.
    fn read_b(handler: Box<dyn BufRead>) -> anyhow::Result<()> {
        let mut i = 0;

        for line in handler.lines().map(|l| l.unwrap_or(String::from(""))) {
            if !line.is_empty() {
                i = i + 1;
                println!("{:>6}\t{}", i, line)
            } else {
                println!("{}", line)
            }
        }
        Ok(())
    }
}

fn main() {
    if let Err(e) = helpers::run(&Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
