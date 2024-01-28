use chrono::round;
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
    let db_name = env::var("DB_NAME_BACKTEST").expect("DB_NAME_BACKTEST must be set");
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

    let short_period = 1;
    let long_period = 3;
    let trading_period = 1;
    let mut market_data = MarketData::new(
        "backtester".to_owned(),
        short_period * 60,
        long_period * 60,
        trading_period * 60,
        60 * 60 * 24,
        20,
        3,
        0.0,
    );

    let mut previous_price: Option<f64> = None;
    let mut previous_crossover: Option<f64> = None;
    let mut max_price_since_last_crossover: f64 = 0.0;
    let mut min_price_since_last_crossover: f64 = f64::MAX;

    for price in prices {
        market_data.add_price(Some(price), None);
        let (crossover, spread) = market_data.get_market_detail();
        let price_rise: Option<bool>;

        let mut trade_performance = 1.3;

        if let Some(prev_price) = previous_price {
            max_price_since_last_crossover = max_price_since_last_crossover.max(price);
            min_price_since_last_crossover = min_price_since_last_crossover.min(price);

            if crossover != 0.5 {
                if let Some(previous_crossover_val) = previous_crossover {
                    price_rise = if max_price_since_last_crossover > prev_price {
                        Some(true)
                    } else if min_price_since_last_crossover < prev_price {
                        Some(false)
                    } else {
                        None
                    };

                    if let Some(price_rise_val) = price_rise {
                        if previous_crossover_val > 0.5 {
                            if price_rise_val {
                                let up = max_price_since_last_crossover - prev_price;
                                let down = prev_price - min_price_since_last_crossover;
                                if up > spread * 10.0 {
                                    trade_performance += 0.2;
                                }
                            } else {
                                trade_performance -= 0.2;
                            }
                        } else {
                            if price_rise_val {
                                trade_performance -= 0.2;
                            } else {
                                let up = max_price_since_last_crossover - prev_price;
                                let down = prev_price - min_price_since_last_crossover;
                                if down > spread * 10.0 {
                                    trade_performance += 0.2;
                                }
                            }
                        }
                    }

                    max_price_since_last_crossover = price;
                    min_price_since_last_crossover = price;
                }
                previous_crossover = Some(crossover);
            }
        }

        previous_price = Some(price);

        // Write the price and market condition to the output file
        if let Err(e) = writeln!(
            output_file,
            "{}, {}, {}, {}",
            price, crossover, trade_performance, spread
        ) {
            log::error!("Error writing to file: {}", e);
            return;
        }
    }
}
