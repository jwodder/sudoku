use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

static PUZZLE: &str = concat!(
    ". . 3 . 2 . 6 . .\n",
    "9 . . 3 . 5 . . 1\n",
    ". . 1 8 . 6 4 . .\n",
    "\n",
    ". . 8 1 . 2 9 . .\n",
    "7 . . . . . . . 8\n",
    ". . 6 7 . 8 2 . .\n",
    "\n",
    ". . 2 6 . 9 5 . .\n",
    "8 . . 2 . 3 . . 9\n",
    ". . 5 . 1 . 3 . .\n",
);

static SOLUTION: &str = concat!(
    "483921657\n",
    "967345821\n",
    "251876493\n",
    "548132976\n",
    "729564138\n",
    "136798245\n",
    "372689514\n",
    "814253769\n",
    "695417382\n",
);

static PRETTY_SOLUTION: &str = concat!(
    "+-----+-----+-----+\n",
    "|4 8 3|9 2 1|6 5 7|\n",
    "|9 6 7|3 4 5|8 2 1|\n",
    "|2 5 1|8 7 6|4 9 3|\n",
    "+-----+-----+-----+\n",
    "|5 4 8|1 3 2|9 7 6|\n",
    "|7 2 9|5 6 4|1 3 8|\n",
    "|1 3 6|7 9 8|2 4 5|\n",
    "+-----+-----+-----+\n",
    "|3 7 2|6 8 9|5 1 4|\n",
    "|8 1 4|2 5 3|7 6 9|\n",
    "|6 9 5|4 1 7|3 8 2|\n",
    "+-----+-----+-----+\n",
);

static UNSOLVABLE: &str = concat!(
    "200900000\n",
    "000000060\n",
    "000001000\n",
    "502600407\n",
    "000004100\n",
    "000098023\n",
    "000003080\n",
    "005010000\n",
    "007000000\n",
);

#[test]
fn test_stdin() {
    Command::cargo_bin("sudoku")
        .unwrap()
        .write_stdin(PUZZLE)
        .assert()
        .success()
        .stdout(SOLUTION);
}

#[test]
fn test_stdin_pretty() {
    Command::cargo_bin("sudoku")
        .unwrap()
        .arg("--pretty")
        .write_stdin(PUZZLE)
        .assert()
        .success()
        .stdout(PRETTY_SOLUTION);
}

#[test]
fn test_unsolvable() {
    Command::cargo_bin("sudoku")
        .unwrap()
        .write_stdin(UNSOLVABLE)
        .assert()
        .failure()
        .stdout("")
        .stderr("No solution\n");
}

#[test]
fn test_unsolvable_pretty() {
    Command::cargo_bin("sudoku")
        .unwrap()
        .arg("--pretty")
        .write_stdin(UNSOLVABLE)
        .assert()
        .failure()
        .stdout("")
        .stderr("No solution\n");
}

#[test]
fn test_infile() {
    let tmpfile = NamedTempFile::new().unwrap();
    fs::write(&tmpfile, PUZZLE).unwrap();
    Command::cargo_bin("sudoku")
        .unwrap()
        .arg(tmpfile.path())
        .assert()
        .success()
        .stdout(SOLUTION);
}

#[test]
fn test_infile_pretty() {
    let tmpfile = NamedTempFile::new().unwrap();
    fs::write(&tmpfile, PUZZLE).unwrap();
    Command::cargo_bin("sudoku")
        .unwrap()
        .arg(tmpfile.path())
        .arg("-P")
        .assert()
        .success()
        .stdout(PRETTY_SOLUTION);
}
