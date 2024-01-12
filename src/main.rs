use chrono::Local;
use clap::{App, Arg};
use debot_db::TransactionLog;
use debot_market_analyzer::MarketData;
use debot_market_analyzer::TradingStrategy;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // Init logging
    env_logger::init();

    let matches = App::new("Back Tester")
        .arg(
            Arg::with_name("input_files_dir")
                .short('i')
                .long("input")
                .help("Sets a custom directory for input files")
                .takes_value(true)
                .default_value("input_files"),
        )
        .arg(
            Arg::with_name("output_files_dir")
                .short('o')
                .long("output")
                .help("Sets a custom directory for output files")
                .takes_value(true)
                .default_value("output_files"),
        )
        .arg(
            Arg::with_name("remote")
                .short('r')
                .long("remote")
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

    let input_files_dir = matches.value_of("input_files_dir").unwrap();
    let output_files_dir = matches.value_of("output_files_dir").unwrap();

    if matches.is_present("remote") {
        download_files(input_files_dir).await;
    }

    if matches.is_present("execute") {
        run_tests(input_files_dir, output_files_dir).expect("execution failed");
    }
}

fn run_tests(input_files_dir: &str, output_files_dir: &str) -> Result<()> {
    let entries = fs::read_dir(input_files_dir)?;
    let output_path = PathBuf::from(output_files_dir);

    for entry in entries {
        let entry = entry?;
        let file_path = entry.path();

        if let Some(extension) = file_path.extension() {
            if extension == "txt" {
                backtest(file_path, output_path.clone());
            }
        }
    }

    Ok(())
}

async fn download_files(test_files_dir: &str) {
    // Set up the DB handler
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
    let transaction_log = TransactionLog::new(1000, 1000, 1000, &mongodb_uri, &db_name).await;

    // Download price data
    let db = transaction_log.get_db().await.expect("db is none");
    let market_data = TransactionLog::get_price_market_data(&db).await;

    // Save prices as a file
    for (_, price_points_map) in market_data {
        for (token_name, price_points) in price_points_map {
            // Format the current timestamp
            let timestamp = Local::now().format("%y%m%d-%H%M%S").to_string();

            // Create a file path with the timestamp
            let file_name = format!("{}-{}.txt", token_name, timestamp);
            let file_path = Path::new(test_files_dir).join(file_name);

            let mut file = File::create(&file_path).expect("Unable to create file");

            for price_point in price_points {
                writeln!(file, "{}", price_point.price).expect("Unable to write to file");
            }
        }
    }
}

fn backtest(test_file_path: PathBuf, output_dir_path: PathBuf) {
    log::info!("Testing file: {:?}", test_file_path);

    // Extract the token name from the file name
    let file_stem = test_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    // Create output file path
    let output_file_path = output_dir_path.join(format!("{}.out", file_stem));

    // Open the output file for writing
    let mut output_file = match File::create(&output_file_path) {
        Ok(file) => file,
        Err(error) => {
            log::error!("Error creating output file: {:?}", error);
            return;
        }
    };

    // Read the price data from the file
    let file = match File::open(&test_file_path) {
        Ok(file) => file,
        Err(error) => {
            log::error!("Error opening file: {:?}", error);
            return;
        }
    };
    let reader = io::BufReader::new(file);

    let mut prices = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => match line.parse::<f64>() {
                Ok(price) => prices.push(price),
                Err(_) => println!("Invalid line: {}", line),
            },
            Err(error) => println!("Error reading line: {:?}", error),
        }
    }

    let short_period = 15 * 4;
    let long_period = 60 * 4;
    let trading_period = 60;
    let mut market_data = MarketData::new(
        "backtester".to_owned(),
        short_period * 60,
        long_period * 60,
        trading_period * 60,
        60 * 60 * 24,
        0,
        0.0,
    );

    for price in prices {
        market_data.add_price(Some(price), None);
        let market_condition = market_data.assess_market_condition();
        let (rsi, rsi_short, rsi_long, is_expanding, trend_type, is_breakout, is_crossover, adx) =
            market_data.get_market_detail();

        let mut open_action_trendfollow =
            market_data.is_open_signaled(TradingStrategy::TrendFollow);
        let open_action_trendfollow = if open_action_trendfollow.len() == 0 {
            1.0
        } else {
            match open_action_trendfollow.pop().unwrap() {
                debot_market_analyzer::TradeAction::BuyOpen(_) => 1.025,
                debot_market_analyzer::TradeAction::SellOpen(_) => 0.975,
                _ => 1.0,
            }
        };

        // Write the price and market condition to the output file
        if let Err(e) = writeln!(
            output_file,
            "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
            price,
            market_condition.to_numeric(),
            rsi,
            rsi_short,
            rsi_long,
            is_expanding,
            trend_type,
            is_breakout,
            is_crossover,
            adx,
            open_action_trendfollow,
        ) {
            log::error!("Error writing to file: {}", e);
            return;
        }
    }
}
