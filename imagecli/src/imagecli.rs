mod imagix;

use std::path::PathBuf;

use imagix::{error::ImagixError, resize::{self, process_resize_request}, stats::get_stats};

use structopt::StructOpt;

// Define commandline arguments in a struct
#[derive(StructOpt, Debug)]
#[structopt(
    name = "resize",
    about = "This is a tool for image resizing and stats",
    help = "Specify subcommand resize or stats. For help,
     type imagecli resize --help or imagecli stats --help"
)]
enum CommandLine {
    #[structopt(help = "
        Specify size(small/medium/large),
        mode(single/all) and srcfolder")]
    Resize {
        #[structopt(long)]
        size: resize::SizeOption,
        #[structopt(long)]
        mode: resize::Mode,
        #[structopt(long)]
        srcfolder: PathBuf,
    },
    #[structopt(help = "Specify srcfolder")]
    Stats {
        #[structopt(long, parse(from_os_str))]
        srcfolder: PathBuf,
    },
}

fn main() {
    let args: CommandLine = CommandLine::from_args();

    match args {
        CommandLine::Resize {
            size,
            mode,
            mut srcfolder 
        } => {
            match process_resize_request(size, mode,  &mut srcfolder) {
                Ok(_) => println!("Image resized succesfully"),
                Err(e) => {
                    match e {
                        ImagixError::FileIOError(e) => {
                            eprintln!("{}", e);
                        },
                        ImagixError::FormatError(e) => {
                            eprintln!("{}", e);
                        },
                        ImagixError::ImageResizingError(e) => {
                            eprintln!("{}", e);
                        },
                        ImagixError::UserInputError(e) => {
                            eprintln!("{}", e);
                        },
                    }
                }
            }
        }
        CommandLine::Stats { srcfolder } => {
            match get_stats(srcfolder) {
                Ok((count, size )) => {
                    println!("Found {:?} image files with aggreate size of {:?} KB", count, size);
                }
                Err (e) => {
                    match e {
                        ImagixError::FileIOError(e) => {
                            eprintln!("{}", e);
                        },
                        ImagixError::FormatError(e) => {
                            eprintln!("{}", e);
                        },
                        ImagixError::ImageResizingError(e) => {
                            eprintln!("{}", e);
                        },
                        ImagixError::UserInputError(e) => {
                            eprintln!("{}", e);
                        },
                    }
                }
            }
        }
    }
}
