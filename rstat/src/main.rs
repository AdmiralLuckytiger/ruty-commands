mod srcstats; 

use std::path::PathBuf;

use structopt::{self, StructOpt};

use srcstats::{errors::StatsError, get_summary_bin_stats, get_summary_src_stats};

#[derive(Debug, structopt::StructOpt)]
#[structopt(name="rstat", about="Rust source statistics. Given a directory, it will generate a file count of Rust sources files
, and source code metrics such as the number of blanks, comments, and actual lines of code within the directory structure. Futhermore
, you could also analyse the binary files generated obtenening the total weight and number of binary files in a given folder.")]
enum Opt {
    #[structopt(about = "Anlyse the source files."  ,help = "Specify folder to analyse it's content.")]
    Src {
        /// srcfolder: directory with the Rust files
        #[structopt()]
        src_folder: PathBuf,
    },
    #[structopt(about = "Analyse the binary files", help = "Specify folder to analyse it's content.")]
    Bin {
        /// binfolder: directory with the Rust files
        #[structopt()]
        bin_folder: PathBuf,
    }  
}

/// DONE: Add bin for binary analisys
fn main() -> Result<(), StatsError>{
    // 1. Accepts user inputs from the commandline
    let opt = Opt::from_args();

    // 2. Invokes the appropiate method to compute the source code metrics
     match opt {
        // 3. Display the result to the user
        Opt::Src { src_folder} => {
            match get_summary_src_stats(&src_folder) {
                Ok(stats) => {
                    println!("Summary stats: {}", stats);
                },
                // 4. In the event of errors, a suitable error message is displayed to the use.
                Err(e) => {
                    eprintln!("{}", e.warn);
                }
            }

        },
        // 3. Display the result to the user 
        Opt::Bin { bin_folder } => {
            match get_summary_bin_stats(&bin_folder) {
                Ok(stats) => {
                    println!("Summary stats: {}", stats);
                },
                // 4. In the event of errors, a suitable error message is displayed to the use.
                Err(e) => {
                    println!("{}", e.warn);
                }
            }
        } 
    }

    Ok(())
}
