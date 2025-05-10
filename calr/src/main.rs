use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, version)]
#[command(author = "Eduardo Palou de Comasema Jaume")]
/// Rust version of `cal`
struct Cli {
    #[arg(value_name("YEAR"), value_parser = clap::value_parser!(i32).range(1..=9999))]
    year: Option<i32>,

    #[arg(short)]
    /// Month name or number (1-12)
    month: Option<String>,

    #[arg(short('y'), long("year"), conflicts_with_all(["month", "year"]))]
    /// Show whole current year
    show_current_year: bool,
}

mod helpers {
    use chrono::{Datelike, Local, NaiveDate, Weekday};

    const LINE_WIDTH: usize = 22;

    const MONTH_NAMES: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    pub fn run(args: crate::Cli) -> anyhow::Result<()> {
        let today = Local::now().date_naive();
        let mut month = args.month.map(parse_month).transpose()?;
        let mut year = args.year;

        if args.show_current_year {
            month = None;
            year = Some(today.year_ce().1 as i32);
        } else if month.is_none() && year.is_none() {
            month = Some(today.month0() + 1);
            year = Some(today.year_ce().1 as i32);
        }

        let year = year.unwrap_or(today.year_ce().1 as i32);

        match month {
            None => {
                print_whole_year(year, today);
            }
            Some(m) => {
                print_month(year, m, today);
            }
        }

        Ok(())
    }

    pub fn print_whole_year(year: i32, today: NaiveDate) {
        println!("                            {}", year);
        for m in 0..4 {
            let col1 = format_month(year, m * 3 + 1, false, today);
            let col2 = format_month(year, m * 3 + 2, false, today);
            let col3 = format_month(year, m * 3 + 3, false, today);

            let mut output: Vec<String> = Vec::new();

            for row in 0..8 {
                output.push(col1.get(row).expect("Valid index").clone());
                output.push(col2.get(row).expect("Valid index").clone());
                output.push(col3.get(row).expect("Valid index").clone());

                println!("{}", output.join(""));
                output.clear();
            }

            if m != 3 {
                println!();
            }
        }
    }

    pub fn print_month(year: i32, month: u32, today: NaiveDate) {
        let output = format_month(year, month, true, today);
        output.iter().for_each(|r| println!("{}", r));
    }

    pub fn parse_month(month: String) -> anyhow::Result<u32> {
        match month.parse::<u32>() {
            Err(_) => {
                let num_candidates = MONTH_NAMES
                    .iter()
                    .filter(|m| m.to_lowercase().starts_with(&month.to_lowercase()))
                    .count();

                if num_candidates > 1 {
                    anyhow::bail!(r#"Invalid month "{}""#, month);
                }

                match MONTH_NAMES
                    .iter()
                    .enumerate()
                    .find(|(_, m)| m.to_lowercase().starts_with(&month.to_lowercase()))
                {
                    None => {
                        anyhow::bail!(r#"Invalid month "{}""#, month);
                    }
                    Some((i, _)) => Ok((i + 1) as u32),
                }
            }
            Ok(val) => {
                if (1..=12).contains(&val) {
                    Ok(val)
                } else {
                    anyhow::bail!(r#"month "{}" not in the range 1 through 12"#, val);
                }
            }
        }
    }

    pub fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();

        // Store Header row
        output.push(generate_header_row(year, month, print_year));

        // Store weekday_row
        output.push("Su Mo Tu We Th Fr Sa  ".to_string());

        // Format row of days
        let mut week_row: Vec<String> = Vec::new();
        let first_day_in_month = NaiveDate::from_ymd_opt(year, month, 1).expect("Valid data");
        let days_in_month = first_day_in_month.num_days_in_month() as usize;

        first_day_in_month
            .iter_days()
            .take(days_in_month)
            .for_each(|d| {
                let day: String = if d.day() < 10 {
                    format!(" {}", d.day())
                } else {
                    format!("{}", d.day())
                };

                if today == d {
                    let style = ansi_term::Style::new().reverse();
                    week_row.push(style.paint(day).to_string());
                } else {
                    week_row.push(day);
                }

                if d.weekday() == Weekday::Sat {
                    let num_d = 7 - week_row.len();

                    // Pad days in week
                    for _ in 0..num_d {
                        week_row.insert(0, String::from("  "));
                    }

                    week_row.push(String::from(" "));

                    // Push to output
                    output.push(week_row.join(" "));

                    // Clean vector
                    week_row.clear();
                } else if d.day() == days_in_month as u32 {
                    let num_d = 7 - week_row.len();

                    // Pad days in week
                    for _ in 0..num_d {
                        week_row.push(String::from("  "));
                    }

                    week_row.push(String::from(" "));

                    // Push to output
                    output.push(week_row.join(" "));

                    // Clean vector
                    week_row.clear();
                }
            });

        while output.len() < 8 {
            output.push("                      ".to_string());
        }

        output
    }

    #[allow(dead_code)]
    fn generate_header_row(year: i32, month: u32, print_year: bool) -> String {
        let mut header_row: String = "                   ".to_string();

        // Format header row
        let header = if print_year {
            format!(
                "{} {}",
                MONTH_NAMES
                    .get((month - 1) as usize)
                    .expect("Month is previously checked"),
                year
            )
        } else {
            MONTH_NAMES
                .get((month - 1) as usize)
                .expect("Month is previously checked")
                .to_string()
        };

        let n = (20 - header.len()) / 2;
        header_row.insert_str(n, &header);
        header_row.truncate(22);

        header_row
    }

    #[allow(dead_code)]
    pub fn last_day_in_month(year: i32, month: u32) -> Option<NaiveDate> {
        // The first day of the next month ...
        let (y, m): (i32, i32) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month as i32 + 1)
        };

        match NaiveDate::from_ymd_opt(y, m as u32, 1) {
            None => None,
            Some(d) => {
                // ...is preceded by the last day of the original month
                d.pred_opt()
            }
        }
    }
}

fn main() {
    if let Err(e) = helpers::run(Cli::parse()) {
        eprint!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::{format_month, last_day_in_month, parse_month};
    use chrono::prelude::*;

    #[test]
    fn test_parse_month() {
        let res = parse_month("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "0" not in the range 1 through 12"#
        );

        let res = parse_month("13".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "13" not in the range 1 through 12"#
        );

        let res = parse_month("foo".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid month "foo""#);
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31)
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29)
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30)
        );
    }
}
