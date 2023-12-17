use clap::{App, Arg};
use reqwest::blocking::get;
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let matches = App::new("Back Tester")
        .arg(
            Arg::with_name("directory")
                .short('d')
                .long("directory")
                .value_name("DIRECTORY")
                .help("Sets a custom directory for test files")
                .takes_value(true)
                .default_value("testsets"),
        )
        .arg(
            Arg::with_name("remote")
                .short('r')
                .long("remote")
                .value_name("URL")
                .help("download test files")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("execute")
                .short('x')
                .long("execute")
                .help("Execute backtests")
                .takes_value(false),
        )
        .get_matches();

    let directory = matches.value_of("directory").unwrap();

    if let Some(url) = matches.value_of("remote") {
        download_files(url);
    }

    if matches.is_present("execute") {
        run_tests(directory);
    }
}

fn run_tests(directory: &str) {
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                backtest(entry.path());
            }
        }
    } else {
        eprintln!("Could not read directory: {}", directory);
    }
}

fn download_files(url: &str) {
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");

    match get(url) {
        Ok(response) => {
            if let Ok(contents) = response.text() {
                // ここでファイルの内容を処理し、保存します
                println!("Downloaded data: {}", contents);
            } else {
                eprintln!("Failed to read response text");
            }
        }
        Err(_) => {
            eprintln!("Failed to download file");
        }
    }
}

fn backtest(test_file_path: PathBuf) {
    println!("Testing file: {:?}", test_file_path);
}
