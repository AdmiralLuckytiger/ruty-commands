use clap::{Args, Parser};

#[derive(Debug, Parser)]
#[command(about, version)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `cut`
struct Cli {
    #[arg(value_name = "FILES", default_values = ["-"], num_args=0..)]
    /// Input file(s)
    files: Vec<String>,

    #[arg(short, long, value_name = "DELIMITER", default_value = "\t")]
    /// Field delimeter
    delimiter: String,

    #[command(flatten)]
    extract: ArgsExtract,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct ArgsExtract {
    #[arg(short, long)]
    /// Selected fields
    fields: Option<String>,

    #[arg(short, long)]
    /// Selected bytes
    bytes: Option<String>,

    #[arg(short, long)]
    /// Selected chars
    chars: Option<String>,
}

mod helpers {
    use std::{
        fs::File,
        io::{self, BufRead, BufReader},
        ops::Range,
    };

    use csv::{ReaderBuilder, StringRecord};

    type PositionList = Vec<Range<usize>>;

    #[derive(Debug)]
    enum Extract {
        Fields(PositionList),
        Bytes(PositionList),
        Chars(PositionList),
    }

    pub fn run(args: crate::Cli) -> anyhow::Result<()> {
        if args.delimiter.len() != 1 {
            anyhow::bail!("--delim \"{}\" must be a single byte", args.delimiter);
        }

        let extract = if let Some(fields) = args.extract.fields.map(parse_pos).transpose()? {
            Extract::Fields(fields)
        } else if let Some(bytes) = args.extract.bytes.map(parse_pos).transpose()? {
            Extract::Bytes(bytes)
        } else if let Some(chars) = args.extract.chars.map(parse_pos).transpose()? {
            Extract::Chars(chars)
        } else {
            anyhow::bail!("The extract should have at least one field");
        };

        for filename in &args.files {
            match open(filename) {
                Err(err) => eprintln!("{}: {}", filename, err),
                Ok(handler) => match extract {
                    Extract::Fields(ref field_pos) => {
                        let mut reader = ReaderBuilder::new()
                            .delimiter(*args.delimiter.as_bytes().first().unwrap_or(&b'\t'))
                            .has_headers(false)
                            .from_reader(handler);

                        for record in reader.records() {
                            println!(
                                "{}",
                                extract_fields(&record.unwrap(), &field_pos).join(&args.delimiter)
                            );
                        }
                    }
                    Extract::Bytes(ref byte_pos) => handler.lines().for_each(|l| {
                        println!("{}", extract_bytes(&l.unwrap_or("".to_string()), &byte_pos))
                    }),
                    Extract::Chars(ref chars_pos) => handler.lines().for_each(|l| {
                        println!(
                            "{}",
                            extract_chars(&l.unwrap_or("".to_string()), &chars_pos)
                        )
                    }),
                },
            }
        }

        Ok(())
    }

    pub fn parse_pos(range: String) -> anyhow::Result<PositionList> {
        let mut out: PositionList = Vec::new();

        if range.is_empty() || range.contains('+') {
            anyhow::bail!("illegal list value: \"{}\"", range);
        }

        let ranges = range.split(',');

        for r in ranges {
            let i: Vec<&str> = r.split("-").collect();

            if i.contains(&"0") {
                anyhow::bail!("illegal list value: \"0\"")
            }

            match i.len() {
                1 => {
                    let up = match i[0].parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => anyhow::bail!("illegal list value: \"{}\"", i[0]),
                    };

                    if up <= 0 {
                        anyhow::bail!("illegal list value: \"{}\"", up);
                    }

                    out.push(up - 1..up);
                }
                2 => {
                    let down = match i[0].parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => anyhow::bail!("illegal list value: \"{}\"", range),
                    };

                    let up = match i[1].parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => anyhow::bail!("illegal list value: \"{}\"", range),
                    };

                    if up <= down {
                        anyhow::bail!(
                            "First number in range ({}) must be lower than second number ({})",
                            down,
                            up
                        );
                    }

                    out.push(down - 1..up);
                }
                _ => anyhow::bail!("illegal list value: \"{}\"", range),
            }
        }

        Ok(out)
    }

    fn open(filename: &str) -> anyhow::Result<Box<dyn BufRead>> {
        match filename {
            "-" => Ok(Box::new(BufReader::new(io::stdin()))),
            _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
        }
    }

    pub fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
        let mut out = String::new();

        for ranges in char_pos {
            line.chars().enumerate().for_each(|(i, val)| {
                if ranges.contains(&i) {
                    out.push(val)
                }
            });
        }

        out
    }

    pub fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
        let mut bytes: Vec<u8> = Vec::new();

        for ranges in byte_pos {
            line.bytes().enumerate().for_each(|(i, val)| {
                if ranges.contains(&i) {
                    bytes.push(val)
                }
            });
        }

        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();

        for ranges in field_pos {
            record.into_iter().enumerate().for_each(|(i, val)| {
                if ranges.contains(&i) {
                    fields.push(format!("{}", val));
                }
            });
        }

        fields
    }
}

fn main() {
    if let Err(e) = helpers::run(Cli::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::helpers::*;
    use csv::StringRecord;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("".to_string()).is_err());

        // Zero is an error
        let res = parse_pos("0".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        let res = parse_pos("0-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        // A leading "+" is an error
        let res = parse_pos("+1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "+1""#,);

        let res = parse_pos("+1-2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );

        let res = parse_pos("1-+2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );

        // Any non-number is an error
        let res = parse_pos("a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1,a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1-a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "1-a""#,);

        let res = parse_pos("a-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a-1""#,);

        // Wonky ranges
        let res = parse_pos("-".to_string());
        assert!(res.is_err());

        let res = parse_pos(",".to_string());
        assert!(res.is_err());

        let res = parse_pos("1,".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-1".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-a".to_string());
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }
}
