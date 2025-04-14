use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `runiq`
struct Args {
    /// Input file
    #[arg(default_value_t = String::from("-"))]
    in_file: String,

    /// Output file
    //#[arg(short('o'), long("output"))]
    #[arg()]
    out_file: Option<String>,

    /// Show counts
    #[arg(short('c'), long)]
    count: bool,
}

mod helpers {
    pub fn run(args: crate::Args) -> anyhow::Result<()> {
        let mut file =
            open(&args.in_file).map_err(|e| anyhow::anyhow!("{}: {}", args.in_file, e))?;

        let mut out_file = write(args.out_file.clone()).map_err(|e| {
            anyhow::anyhow!(
                "{}: {}",
                args.out_file.clone().unwrap_or("stdout".to_string()),
                e
            )
        })?;

        let mut line = String::new();
        let mut previous_line: Option<String> = None;

        let mut cnt: u64 = 1;

        loop {
            let bytes = file.read_line(&mut line)?;

            if previous_line.clone().unwrap_or(String::new()).trim_end() == line.clone().trim_end()
            {
                cnt = cnt + 1;
            } else {
                match previous_line {
                    Some(line) => {
                        if args.count {
                            write!(out_file, "{}", format!("{:>4} {}", cnt, line))?;
                        } else {
                            write!(out_file, "{}", format!("{}", line))?;
                        }
                    }
                    None => {}
                }

                cnt = 1;
                previous_line = Some(line.clone());
            }

            line.clear();

            if bytes == 0 {
                out_file.flush()?;
                break;
            }
        }

        Ok(())
    }

    fn open(filename: &str) -> anyhow::Result<Box<dyn std::io::BufRead>> {
        match filename {
            "-" => Ok(Box::new(std::io::BufReader::new(std::io::stdin()))),
            _ => Ok(Box::new(std::io::BufReader::new(std::fs::File::open(
                filename,
            )?))),
        }
    }

    fn write(filename: Option<String>) -> anyhow::Result<Box<dyn std::io::Write>> {
        match filename {
            Some(file) => Ok(Box::new(std::io::BufWriter::new(std::fs::File::create(
                file,
            )?))),
            None => Ok(Box::new(std::io::BufWriter::new(std::io::stdout()))),
        }
    }
}

fn main() {
    if let Err(e) = helpers::run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
