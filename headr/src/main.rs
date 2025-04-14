use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `head`
struct Args {
    /// Input file(s)
    #[arg(
        default_value = "-",
        num_args=0..,
        value_name = "FILE"
    )]
    files: Vec<String>,

    /// Number of lines
    #[arg(
        short('n'),
        long("lines"),
        default_value_t = 10,
        conflicts_with = "bytes",
        value_name = "LINES",
        value_parser(clap::value_parser!(u64).range(1..)),
    )]
    lines: u64,

    /// Number of bytes
    #[arg(
        short('c'),
        long("bytes"),
        value_name = "BYTES",
        value_parser(clap::value_parser!(u64).range(1..)),
    )]
    bytes: Option<u64>,
}

mod helper {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Read};

    /// Command line main logic
    pub fn run(args: crate::Args) -> anyhow::Result<()> {
        for (i, filename) in args.files.iter().enumerate() {
            match open(&filename) {
                Err(err) => eprintln!("{}: {}", filename, err),
                Ok(mut handler) => {
                    if args.files.len() > 1 {
                        let _ = print_header(&filename, i);
                    }
                    match args.bytes {
                        None => print_lines(&mut handler, args.lines)?,
                        Some(n) => print_bytes(&mut handler, n)?,
                    }
                }
            }
        }
        Ok(())
    }

    fn open(filename: &str) -> anyhow::Result<Box<dyn BufRead>> {
        match filename {
            "-" => Ok(Box::new(BufReader::new(io::stdin()))),
            _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
        }
    }

    fn print_header(filename: &str, line_num: usize) -> anyhow::Result<()> {
        if line_num == 0 {
            print!("==> {} <==\n", &filename);
        } else {
            print!("\n==> {} <==\n", &filename);
        }

        Ok(())
    }

    fn print_lines(handler: &mut Box<dyn BufRead>, num_lines: u64) -> anyhow::Result<()> {
        let mut buff = String::new();

        for _ in 0..num_lines {
            let bytes = handler.read_line(&mut buff)?;

            if bytes == 0 {
                return Ok(());
            }

            print!("{}", &buff);
            buff.clear();
        }

        Ok(())
    }

    fn print_bytes(handler: &mut Box<dyn BufRead>, num_bytes: u64) -> anyhow::Result<()> {
        let output: Vec<u8> = handler
            .bytes()
            .take(num_bytes as usize)
            .map(|c| c.unwrap_or(b' '))
            .collect();

        print!("{}", String::from_utf8_lossy(&output));
        Ok(())
    }
}

fn main() {
    if let Err(e) = helper::run(Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
