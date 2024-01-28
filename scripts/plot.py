#!/usr/bin/env python3

import os
import argparse
import matplotlib.pyplot as plt
import matplotlib
matplotlib.use('TkAgg')


def plot_time_series_data(file_path):
    prices = []
    indicators = []
    prices = []
    price_shorts = []
    price_longs = []
    is_expandings = []
    trend_types = []
    is_breakouts = []
    is_crossovers = []
    adxs = []
    open_signal_trendfollows = []
    open_action_grid_trades = []
    is_range_bounds = []
    performances = []
    spreads = []

    with open(file_path, 'r') as file:
        for line in file:
            parts = line.strip().split(',')
            if len(parts) == 4:
                try:
                    price = float(parts[0])
                    is_crossover = float(parts[1])
                    performance = float(parts[2])
                    spread = float(parts[3]) / 10.0
                    prices.append(price)
                    is_crossovers.append(is_crossover)
                    performances.append(performance)
                    spreads.append(spread)

                except ValueError:
                    print(f"Invalid data in file {file_path}: {line}")
                    continue
            else:
                print(f"Invalid data in file {file_path}")
                raise Exception

    # Normalize prices
    min_price = min(prices)
    max_price = max(prices)
    normalized_prices = [(price - min_price) /
                         (max_price - min_price) for price in prices]

    min_value = 0
    max_value = 1.5
    yticks = [i/10 for i in range(int(min_value*10), int(max_value*10)+1)]

    plt.figure(figsize=(10, 6))

    mng = plt.get_current_fig_manager()
    mng.resize(*mng.window.maxsize())  # For Linux, uncomment this line

    plt.yticks(yticks)

    for ytick in yticks:
        plt.axhline(y=ytick, color='gray', linestyle='--', linewidth=0.5)

    plt.plot(normalized_prices, label='Normalized Price')
    plt.plot(is_crossovers, label='CrossOver')
    plt.plot(performances, label='Performance')
    # plt.plot(spreads, label='Spread')
    # plt.plot(open_action_grid_trades, label='Grid Trades')
    plt.title(f"Time Series Data for {os.path.basename(file_path)}")
    plt.xlabel("Time")
    plt.ylabel("Normalized Value")
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.subplots_adjust(right=0.85)
    plt.show()


def plot_all_files_in_directory(directory):
    for filename in os.listdir(directory):
        if filename.endswith(".out"):
            file_path = os.path.join(directory, filename)
            plot_time_series_data(file_path)


def main():
    parser = argparse.ArgumentParser(
        description="Plot time series data from text files in a directory")
    parser.add_argument("-d", "--directory", type=str, default="output_files")
    args = parser.parse_args()

    plot_all_files_in_directory(args.directory)


if __name__ == "__main__":
    main()
