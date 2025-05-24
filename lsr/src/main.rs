use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, version)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `ls`
pub struct Cli {
    #[arg(value_name("PATH"), default_value("."))]
    /// Files and/or directories
    paths: Vec<String>,

    #[arg(short, long)]
    /// Long listing
    long: bool,

    #[arg(short('a'), long("all"))]
    /// Show all files
    show_hidden: bool,
}

mod helpers {
    use std::{fs, os::unix::fs::MetadataExt, path};

    use tabular::{Row, Table};

    pub fn run(args: super::Cli) -> anyhow::Result<()> {
        let paths = find_files(&args.paths, args.show_hidden)?;

        if args.long {
            print!("{}", format_output(&paths)?);
        } else {
            paths.iter().for_each(|path| println!("{}", path.display()));
        }

        Ok(())
    }

    pub fn find_files(paths: &[String], show_hidden: bool) -> anyhow::Result<Vec<path::PathBuf>> {
        let mut ouput: Vec<path::PathBuf> = Vec::new();

        for path in paths {
            if let Err(e) = fs::metadata(path) {
                eprintln!("{path}: {e}");
                continue;
            }

            let path = std::path::Path::new(path);

            if path.is_file() {
                ouput.push(path::PathBuf::from(path));
            } else if path.is_dir() {
                fs::read_dir(path)?
                    .into_iter()
                    .for_each(|entry| match entry {
                        Ok(direntry) => {
                            let path = direntry.path();

                            if show_hidden {
                                ouput.push(path);
                            } else if let Some(entry_name) = path.file_name() {
                                if let Some(name) = entry_name.to_str() {
                                    if !name.starts_with(".") {
                                        ouput.push(path);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{e}")
                        }
                    });
            }
        }

        Ok(ouput)
    }

    #[allow(dead_code)]
    pub fn format_output(paths: &[path::PathBuf]) -> anyhow::Result<String> {
        //                       1   2     3     4     5     6     7     8
        let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
        let mut table = Table::new(fmt);

        for path in paths {
            if let Ok(metadata) = fs::metadata(path) {
                let user_name = match users::get_user_by_uid(metadata.uid()) {
                    Some(user) => {
                        if let Some(user_name) = user.name().to_str() {
                            String::from(user_name)
                        } else {
                            eprintln!("{}: Missing owner.", path.display());
                            "????".to_string()
                        }
                    }
                    None => {
                        eprintln!("{}: Missing owner.", path.display());
                        "????".to_string()
                    }
                };

                let group_name = match users::get_group_by_gid(metadata.gid()) {
                    Some(group) => {
                        if let Some(group_name) = group.name().to_str() {
                            String::from(group_name)
                        } else {
                            eprintln!("{}: Missing group.", path.display());
                            "????".to_string()
                        }
                    }
                    None => {
                        eprintln!("{}: Missing group.", path.display());
                        "????".to_string()
                    }
                };

                table.add_row(
                    Row::new()
                        .with_cell(if metadata.is_dir() { "d" } else { "-" }) // 1 "d" or "-"
                        .with_cell(format_mode(metadata.mode())) // 2 permissions
                        .with_cell(metadata.nlink()) // 3 number of links
                        .with_cell(user_name) // 4 user name
                        .with_cell(group_name) // 5 group name
                        .with_cell(metadata.len()) // 6 size
                        .with_cell(last_modified(&metadata)) // 7 modifications
                        .with_cell(path.display()), // 8 path
                );
            }
        }

        Ok(format!("{table}"))
    }

    #[allow(dead_code)]
    fn last_modified(metadata: &fs::Metadata) -> String {
        if let Ok(time) = metadata.modified() {
            let (sec, nsec) = match time.duration_since(std::time::UNIX_EPOCH) {
                Ok(duration) => (duration.as_secs() as i64, duration.subsec_nanos()),
                Err(e) => {
                    let dur = e.duration();
                    let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());

                    if nsec == 0 {
                        (-sec, 0)
                    } else {
                        (-sec - 1, 1_000_000_000 - nsec)
                    }
                }
            };

            let dt = chrono::DateTime::from_timestamp(sec, nsec).expect("Valid timespant");

            format!("{}", dt.format("%B %e %R"))
        } else {
            String::from("Not supported for this platform")
        }
    }

    pub fn format_mode(mode: u32) -> String {
        let uread = if mode & 0o400 != 0 { "r" } else { "-" };
        let uwrite = if mode & 0o200 != 0 { "w" } else { "-" };
        let uexecute = if mode & 0o100 != 0 { "x" } else { "-" };

        let gread = if mode & 0o040 != 0 { "r" } else { "-" };
        let gwrite = if mode & 0o020 != 0 { "w" } else { "-" };
        let gexecute = if mode & 0o010 != 0 { "x" } else { "-" };

        let oread = if mode & 0o004 != 0 { "r" } else { "-" };
        let owrite = if mode & 0o002 != 0 { "w" } else { "-" };
        let oexecute = if mode & 0o001 != 0 { "x" } else { "-" };

        format!("{uread}{uwrite}{uexecute}{gread}{gwrite}{gexecute}{oread}{owrite}{oexecute}")
    }
}

fn main() {
    if let Err(e) = helpers::run(Cli::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use crate::helpers::{find_files, format_mode, format_output};
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(!parts.is_empty() && parts.len() <= 10);

        let perms = parts.first().unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}
