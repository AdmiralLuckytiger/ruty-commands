use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(about, version)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
///Rust verion of `find`
struct Args {
    /// Search paths
    #[arg(default_values_t = vec![".".to_string()], value_name= "PATH")]
    paths: Vec<String>,

    /// Name
    #[arg(short, long("name"), value_name = "NAME", num_args=0..)]
    names: Vec<regex::Regex>,

    /// Entry type
    #[arg(short('t'), long("type"), value_name = "TYPE", num_args=0..)]
    entry_types: Vec<EntryType>,
}

#[derive(Debug, Clone, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

impl ValueEnum for EntryType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Dir, Self::File, Self::Link]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Dir => clap::builder::PossibleValue::new("d"),
            Self::File => clap::builder::PossibleValue::new("f"),
            Self::Link => clap::builder::PossibleValue::new("l"),
        })
    }
}

impl EntryType {
    fn type_of_path(entry: &std::path::Path) -> Option<Self> {
        match entry {
            p if p.is_symlink() => Some(EntryType::Link),
            p if p.is_dir() => Some(EntryType::Dir),
            p if p.is_file() => Some(EntryType::File),
            _ => None,
        }
    }
}

mod helpers {
    use walkdir::WalkDir;

    pub fn run(args: crate::Args) -> anyhow::Result<()> {
        for path in args.paths {
            for entry in WalkDir::new(path) {
                match entry {
                    Err(e) => eprintln!("{e}"),
                    Ok(entry) => {
                        let path = entry.path().display().to_string();
                        let file = entry.file_name().to_string_lossy().into_owned();
                        let entry_type = match crate::EntryType::type_of_path(&entry.path()) {
                            Some(t) => t,
                            None => break,
                        };

                        if check_type(&args.entry_types, &entry_type)
                            && check_match(&args.names, &file)
                        {
                            println!("{}", &path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn check_match(set: &Vec<regex::Regex>, hay: &str) -> bool {
        if set.is_empty() {
            return true;
        } else {
            if set.iter().any(|re| re.is_match(&hay)) {
                return true;
            }
        }
        false
    }

    fn check_type(file_types: &Vec<crate::EntryType>, entry_type: &crate::EntryType) -> bool {
        if file_types.is_empty() {
            return true;
        } else {
            if file_types.iter().any(|t| t == entry_type) {
                return true;
            }
        }
        false
    }
}

fn main() {
    if let Err(e) = helpers::run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
