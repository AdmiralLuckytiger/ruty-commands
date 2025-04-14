use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use std::fs;

fn run(args: &[&str], expected_file: &str) -> Result<()> {
    let expected = fs::read_to_string(expected_file)?;
    let output = Command::cargo_bin("echor")?
        .args(args)
        .output()
        .expect("The binary should be compiled without errors and not fail");
    let stdout = String::from_utf8(output.stdout).expect("Not invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn dies_no_args() {
    let mut cmd =
        Command::cargo_bin("echor").expect("The binary should be compiled without errors");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Usage"));
}

#[test]
fn runs() {
    let mut cmd =
        Command::cargo_bin("echor").expect("The binary should be compiled without errors");
    cmd.arg("hello")
        .assert()
        .success()
        .stdout(predicates::str::contains("hello"));
}

#[test]
fn hello1() {
    let outfile = "tests/expected/hello1.txt";
    let expected =
        fs::read_to_string(outfile).expect("The file shoudl exists with the apropiate permissions");
    let mut cmd =
        Command::cargo_bin("echor").expect("This binary should be compiled without errors");
    cmd.arg("Hello there").assert().success().stdout(expected);
}

#[test]
fn hello2() -> Result<()> {
    let expected = fs::read_to_string("tests/expected/hello2.txt")?;
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.args(vec!["Hello", "there"])
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

#[test]
fn hello1_no_newline() -> Result<()> {
    run(&["Hello  there", "-n"], "tests/expected/hello1.n.txt")
}

#[test]
fn hello2_no_newline() -> Result<()> {
    run(&["-n", "Hello", "there"], "tests/expected/hello2.n.txt")
}
