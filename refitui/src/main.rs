mod textviewer;

use structopt::StructOpt;


#[derive(Debug,StructOpt)]
#[structopt(
    name    = "refitui",
    about   = "Basic terminal text viewer implemented in rust",
    author  = "Author: Eduardo",
    version = "1.0.0",
)]
struct Command {
    // This option specified the path to the file to be printed in the terminal
    // This option is positional, meaning it is the first unadorned string you provide
    file: String,
}

fn main() {
    // Get arguments from command line
    let opt: Command = Command::from_args();

    // Check if file exists. If not, print error
    // message and exit process
    if !std::path::Path::new(&opt.file).exists() {
        eprintln!("File does not exists");
        std::process::exit(0);
    } 

    // Open file and load into struct
    println!("{}", termion::cursor::Show);

    // Iniatialize viewer 
    let mut viewer = textviewer::TextViewer::init(&opt.file);
    viewer.show_document();
    viewer.run();
}
