use clap::Parser;

#[derive(Parser, Debug)]
#[command(about, version = "0.1.0", author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `tail`
struct Cli {
    #[arg(value_name = "FILE", required = true)]
    /// Input file(s)
    files: Vec<String>,

    #[arg(short('n'), long, default_value = "10", conflicts_with = "bytes")]
    /// Number of lines
    lines: String,

    #[arg(short('c'), long)]
    /// Number of bytes
    bytes: Option<String>,

    #[arg(short, long)]
    /// Suppress headers
    quiet: bool,
}

mod helpers {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

    #[derive(PartialEq, Debug)]
    pub enum TakeValue {
        PlusZero,
        TakeNum(i64),
    }

    pub fn run(args: super::Cli) -> anyhow::Result<()> {
        let lines =
            parse_num(args.lines).map_err(|e| anyhow::anyhow!("illegal line count -- {}", e))?;

        let bytes = args
            .bytes
            .map(parse_num)
            .transpose()
            .map_err(|e| anyhow::anyhow!("illegal byte count -- {}", e))?;

        let num_files = args.files.len();

        for (i, filename) in args.files.iter().enumerate() {
            if num_files > 1 && !args.quiet {
                if i == 0 {
                    print!("==> {filename} <==\n");
                } else {
                    print!("\n==> {filename} <==\n");
                }
            }
            match File::open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(handler) => {
                    let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                    match bytes {
                        None => {
                            print_lines(BufReader::new(handler), &lines, total_lines)?;
                        }
                        Some(ref bytes) => {
                            print_bytes(BufReader::new(handler), &bytes, total_bytes)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn parse_num(val: String) -> anyhow::Result<TakeValue> {
        if val == "+0" {
            return Ok(TakeValue::PlusZero);
        }

        match val.parse::<i64>() {
            Err(_) => anyhow::bail!("{}", val),
            Ok(v) => {
                if !val.contains("+") && !val.contains("-") {
                    return Ok(TakeValue::TakeNum(-v));
                }

                Ok(TakeValue::TakeNum(v))
            }
        }
    }

    pub fn count_lines_bytes(filename: &str) -> anyhow::Result<(i64, i64)> {
        let mut num_lines: i64 = 0;
        let mut num_bytes: i64 = 0;

        match File::open(filename) {
            Err(e) => anyhow::bail!("{}", e),
            Ok(hanler) => {
                let mut reader = BufReader::new(hanler);
                let mut buff: String = String::new();

                while let Ok(val) = reader.read_line(&mut buff) {
                    if val != 0 {
                        num_lines += 1;

                        buff.bytes().for_each(|_| num_bytes += 1);

                        buff.clear();
                    } else {
                        break;
                    }
                }
            }
        }
        Ok((num_lines, num_bytes))
    }

    fn print_lines<T: BufRead>(
        mut file: T,
        num_lines: &TakeValue,
        total_lines: i64,
    ) -> anyhow::Result<()> {
        let mut buff: String = String::new();
        let mut cnt: u64 = 0;

        match get_start_index(num_lines, total_lines) {
            None => {}
            Some(start_index) => {
                while let Ok(n) = file.read_line(&mut buff) {
                    if n == 0 {
                        break;
                    }

                    cnt += 1;

                    if cnt > start_index {
                        print!("{}", buff);
                    }

                    buff.clear();
                }
            }
        }
        Ok(())
    }

    fn print_bytes<T: Read + Seek>(
        mut file: T,
        num_bytes: &TakeValue,
        total_bytes: i64,
    ) -> anyhow::Result<()> {
        match get_start_index(num_bytes, total_bytes) {
            None => {}
            Some(start_index) => {
                file.seek(SeekFrom::Start(start_index))
                    .map_err(|e| anyhow::anyhow!("{e}"))?;

                let mut reader = BufReader::new(file);
                let mut buff: Vec<u8> = Vec::new();

                while let Ok(n) = reader.read_until(b'\n', &mut buff) {
                    if n == 0 {
                        break;
                    }

                    print!("{}", String::from_utf8_lossy(&buff));

                    buff.clear();
                }
            }
        }
        Ok(())
    }

    pub fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
        match take_val {
            TakeValue::PlusZero => {
                if total > 0 {
                    Some(0)
                } else {
                    None
                }
            }
            TakeValue::TakeNum(ind) => {
                let pos: i64 = *ind;
                let abs_pos: i64 = pos.abs();

                if pos == 0 || total == 0 {
                    return None;
                }

                if abs_pos >= total && pos.is_negative() {
                    return Some(0);
                }

                if abs_pos <= total {
                    if pos.is_positive() {
                        return Some((pos - 1) as u64);
                    } else {
                        return Some((total + pos) as u64);
                    }
                }

                None
            }
        }
    }
}

fn main() {
    if let Err(e) = helpers::run(Cli::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 1);
        assert_eq!(bytes, 24);

        let res = count_lines_bytes("tests/inputs/twelve.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 12);
        assert_eq!(bytes, 63);
    }

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num((i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
