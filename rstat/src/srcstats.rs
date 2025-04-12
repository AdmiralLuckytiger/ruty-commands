pub mod errors;

use std::ffi;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path;
use std::vec;
use std::fmt;
use binaryornot;

use errors::StatsError;

/// Code metrics definition
pub struct SrcStats {
    number_of_files:    u32,
    lines_of_code:      u32,
    comments:           u32,
    blanks:             u32,
}

/// Binary medtrics definition
pub struct BinStats {
    number_of_files: u32,
    weight:          u32,
}

impl fmt::Display for SrcStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " >>> number_of_files: {},
                >>> loc: {},
                >>> comments: {},
                >>> blanks: {}",
         self.number_of_files, self.lines_of_code, self.comments, self.blanks)
    }
}

impl fmt::Display for BinStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " >>> number_of_files: {},
                >>> weight: {} KB", self.number_of_files, self.weight)
    }
}

/// Calculate source metrics for single file
fn get_src_stats_for_file(file: &path::Path) -> Result<SrcStats, StatsError> {
    let file_contents = fs::read_to_string(file)?;

    let mut loc = 0;
    let mut blanks = 0;
    let mut comments =0;

    for line in file_contents.lines() {
        if line.len() == 0 {
            blanks += 1;
        } else if line.trim_start().starts_with("//") {
            comments += 1;
        } else {
            loc += 1;
        }
    }

    Ok(SrcStats {
        number_of_files: u32::try_from(file_contents.lines().count())?,
        lines_of_code: loc,
        comments,
        blanks  
    })
}

/// Calculate binary metrics for single file
fn get_bin_stats_for_file(file: &path::Path) -> Result<BinStats, StatsError> {
    let weight = file.metadata().unwrap().len()/1000;

    Ok(BinStats {
        number_of_files: 0,
        weight: u32::try_from(weight)?,
        
    })
}

/// Calculate source metrics for all files in a directory root
pub fn get_summary_src_stats(folder: &path::Path) -> Result<SrcStats, StatsError> {

    let mut total_loc = 0;
    let mut total_comments = 0;
    let mut total_blanks = 0;

    let mut dir_entries: Vec<path::PathBuf> = vec![folder.to_path_buf()]; 
    let mut file_entries: Vec<fs::DirEntry> = vec![];

    // Recursively iterate over directory entries to get flat
    // list of .rs file
    while let Some(entry) = dir_entries.pop() {
        for inner_entry in fs::read_dir(&entry)? {
            if let Ok(entry) = inner_entry {
                if entry.path().is_dir() {
                    dir_entries.push(entry.path());
                } else {
                    if entry.path().extension() == Some(ffi::OsStr::new("rs")) {
                        file_entries.push(entry);
                    }
                }
            }
        }
    }
    
    let file_count = file_entries.len();

    // Compute stats
    for entry in file_entries {
        let stat = get_src_stats_for_file(&entry.path())?;

        total_blanks += stat.blanks;
        total_comments += stat.comments;
        total_loc += stat.lines_of_code;
    }

    Ok(SrcStats {
        number_of_files: u32::try_from(file_count)?,
        lines_of_code: total_loc,
        comments: total_comments,
        blanks: total_blanks,
    })
}

/// Calculate binary metrics for all files in a directory root
pub fn get_summary_bin_stats(folder: &path::Path) -> Result<BinStats, StatsError> {

    let mut total_weight: u32 = 0;

    let mut dir_entries: Vec<path::PathBuf> = vec![folder.to_path_buf()]; 
    let mut bin_entries: Vec<fs::DirEntry> = vec![];

    // Recursively iterate over directory entries to get flat
    // list of binary file
    while let Some(entry) = dir_entries.pop() {
        for inner_entry in fs::read_dir(&entry)? {
            if let Ok(entry) = inner_entry {
                if entry.path().is_dir() {
                    dir_entries.push(entry.path());
                } else {
                    if entry.path().extension() == None {
                        match binaryornot::is_binary(entry.path()) {
                            Ok(bool) => {
                                if bool {
                                    // Check if the file has the correct permissions
                                    if entry.metadata()?.permissions().mode() & 0o111 != 0 {
                                        bin_entries.push(entry);
                                    }
                                }
                            },
                            Err(_) => {},
                        }
                    }
                }
            }
        }
    }

    let bin_count = bin_entries.len();

    // Compute stats
    for entry in bin_entries {
        let stat = get_bin_stats_for_file(&entry.path())?;

        total_weight += stat.weight;
    }

    Ok(BinStats {
        number_of_files: u32::try_from(bin_count)?,
        weight: total_weight,
    })
}

