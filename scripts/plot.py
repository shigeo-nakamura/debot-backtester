#!/usr/bin/env python3

import os
import argparse
import matplotlib.pyplot as plt
import matplotlib
import numpy as np
matplotlib.use('TkAgg')


def plot_time_series_data(file_path):
    prices = []
    open_signals = []
    close_signals = []

    with open(file_path, 'r') as file:
        for line in file:
            parts = line.strip().split(',')
            if len(parts) == 3:
                try:
                    price = float(parts[0])
                    open_signal = float(parts[1])
                    close_signal = float(parts[2])

                    prices.append(price)
                    open_signals.append(open_signal)
                    close_signals.append(close_signal)
                   

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

    min_value = 0.0
    max_value = 1.5
    yticks = [i/20 for i in range(int(min_value*20), int(max_value*20)+1)]

    plt.figure(figsize=(10, 6))

    mng = plt.get_current_fig_manager()
    mng.resize(*mng.window.maxsize())  # For Linux, uncomment this line

    plt.yticks(yticks)

    for ytick in yticks:
        plt.axhline(y=ytick, color='gray', linestyle='--', linewidth=0.5)

    plt.plot(normalized_prices, label='Normalized Price')
    plt.plot(open_signals, label='Open signal')
    plt.plot(close_signals, label='Close signal')

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
