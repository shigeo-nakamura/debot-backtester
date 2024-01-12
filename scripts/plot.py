#!/usr/bin/env python3

import os
import argparse
import matplotlib.pyplot as plt
import matplotlib
matplotlib.use('TkAgg')


def plot_time_series_data(file_path):
    prices = []
    indicators = []
    rsis = []
    rsi_shorts = []
    rsi_longs = []
    is_expandings = []
    trend_types = []
    is_breakouts = []
    is_crossovers = []
    adxs = []
    open_signal_trendfollows = []

    with open(file_path, 'r') as file:
        for line in file:
            parts = line.strip().split(',')
            if len(parts) == 11:
                try:
                    price = float(parts[0])
                    indicator = float(parts[1])
                    rsi = float(parts[2])
                    rsi_short = float(parts[3])
                    rsi_long = float(parts[4])
                    is_expanding = float(parts[5])
                    trend_type = float(parts[6])
                    is_breakout = float(parts[7])
                    is_crossover = float(parts[8])
                    adx = float(parts[9])
                    open_signal_trendfollow = float(parts[10])

                    prices.append(price)
                    indicators.append(indicator)
                    rsis.append(rsi)
                    rsi_shorts.append(rsi_short)
                    rsi_longs.append(rsi_long)
                    is_expandings.append(is_expanding)
                    trend_types.append(trend_type)
                    is_breakouts.append(is_breakout)
                    is_crossovers.append(is_crossover)
                    adxs.append(adx)
                    open_signal_trendfollows.append(open_signal_trendfollow)

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
    max_value = 1
    yticks = [i/10 for i in range(int(min_value*10), int(max_value*10)+1)]

    plt.figure(figsize=(10, 6))

    mng = plt.get_current_fig_manager()
    mng.resize(*mng.window.maxsize())  # For Linux, uncomment this line

    plt.yticks(yticks)

    for ytick in yticks:
        plt.axhline(y=ytick, color='gray', linestyle='--', linewidth=0.5)

    plt.plot(normalized_prices, label='Normalized Price')
    plt.plot(indicators, label='Technical Indicator')
    plt.plot(rsis, label='RSI')
    plt.plot(rsi_shorts, label='RSI Short')
    plt.plot(rsi_longs, label='RSI Long')
    # plt.plot(is_expandings, label='Is expanding')
    plt.plot(trend_types, label='Trend type')
    plt.plot(is_breakouts, label='BreakOUt')
    plt.plot(is_crossovers, label='CrossOver')
    plt.plot(adxs, label='Adx')
    plt.plot(open_signal_trendfollows, label='Open signal Trendfollow')
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
