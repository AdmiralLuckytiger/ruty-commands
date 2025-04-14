use clap::{Arg, ArgAction, Command, Parser};

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `echo`
struct Args {
    /// Input text
    #[arg(required(true))]
    text: Vec<String>,

    /// Do not print newline
    #[arg(short('n'))]
    omit_newline: bool,
}

fn main() {
    let args = Args::parse();

    dbg!(&args);

    match !args.omit_newline {
        true => {
            println!("{}", args.text.join(" "));
        }
        false => {
            print!("{}", args.text.join(" "));
        }
    }
}
