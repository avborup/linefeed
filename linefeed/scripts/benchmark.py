import argparse
import os
import re
import statistics
import subprocess
import sys
from typing import TypedDict


class BenchmarkStats(TypedDict):
    min: float
    max: float
    mean: float
    stdev: float
    runs: int


class SummaryData(BenchmarkStats):
    file_name: str


# Number of times to run each benchmark
RUNS = 4


def make_pair_for_day(day: int) -> tuple[str, str]:
    script_path = f"tests/linefeed/advent_of_code_2020/day{day:02d}.lf"
    input_path = f"tests/linefeed/advent_of_code_2020/inputs/day{day:02d}-secret.txt"
    return (script_path, input_path)


BENCHMARKS: list[tuple[str, str]] = [
    make_pair_for_day(day) for day in range(1, 13)
]


def run_benchmark(
    linefeed_script: str,
    stdin_file: str,
    runs: int
) -> list[float] | None:
    """
    Runs the benchmark for the given Linefeed script.

    Args:
        linefeed_script: The path to the Linefeed script to benchmark.
        stdin_file: The path to the file to use as stdin.
        runs: The number of times to run the benchmark.

    Returns:
        A list of run times in milliseconds, or None if an error occurred.
    """
    run_times: list[float] = []
    print(
        f"\nBenchmarking '{linefeed_script}' with stdin from '{stdin_file}'...")

    for i in range(runs):
        try:
            with open(stdin_file, "r") as f:
                stdin_content = f.read()

            process = subprocess.run(
                [
                    "cargo",
                    "run",
                    "--release",
                    "--",
                    linefeed_script,
                ],
                input=stdin_content,
                capture_output=True,
                text=True,
                check=True,
            )

            stderr = process.stderr

            match = re.search(r"Run time: (\d+(?:\.\d+)?)(s|ms|µs)", stderr)
            if match:
                value_str, unit = match.groups()
                value = float(value_str)

                if unit == "ms":
                    run_time = value
                elif unit == "µs":
                    run_time = value / 1_000
                else:  # seconds
                    run_time = value * 1_000

                run_times.append(run_time)
                print(f"  Run {i + 1}/{runs}: {run_time:.3f}ms")
            else:
                print(
                    f"Error: Could not find 'Run time' in stderr for run {i + 1}.",
                    file=sys.stderr,
                )
                print(f"Stderr:\n{stderr}", file=sys.stderr)
                return None

        except FileNotFoundError:
            print(
                f"Error: Input file '{stdin_file}' not found.", file=sys.stderr)
            return None
        except subprocess.CalledProcessError as e:
            print(f"Error running Linefeed script: {e}", file=sys.stderr)
            print(f"Stderr:\n{e.stderr}", file=sys.stderr)
            return None
        except Exception as e:
            print(f"An unexpected error occurred: {e}", file=sys.stderr)
            return None

    return run_times


def calculate_and_print_statistics(run_times: list[float]) -> BenchmarkStats | None:
    """
    Calculates and prints the statistics for the given run times.

    Args:
        run_times: A list of run times in milliseconds.

    Returns:
        A dictionary of the calculated statistics, or None.
    """
    if not run_times:
        print("No successful runs to analyze.")
        return None

    mean = statistics.mean(run_times)
    if len(run_times) > 1:
        stdev = statistics.stdev(run_times)
    else:
        stdev = 0

    stats: BenchmarkStats = {
        "min": min(run_times),
        "max": max(run_times),
        "mean": mean,
        "stdev": stdev,
        "runs": len(run_times),
    }

    print("\n  --- Statistics ---")
    print(f"  Total runs: {stats['runs']}")
    print(f"  Min time:   {stats['min']:.3f}ms")
    print(f"  Max time:   {stats['max']:.3f}ms")
    print(f"  Mean time:  {stats['mean']:.3f}ms")
    print(f"  Std dev:    {stats['stdev']:.3f}ms")
    print("  ------------------")
    return stats


def print_summary_table(summary_data: list[SummaryData]):
    """
    Prints a summary table of all benchmark results.

    Args:
        summary_data: A list of dictionaries, each containing the results
                      for one benchmark.
    """
    print("\n--- Benchmark Summary ---")

    # Header
    header = f"| {'File':<25} | {'Mean (ms)':>10} | {'Min (ms)':>10} | {'Max (ms)':>10} | {'Std Dev':>14} |"
    print(header)
    print(f"|{'-'*27}|{'-'*12}|{'-'*12}|{'-'*12}|{'-'*16}|")

    # Rows
    for result in summary_data:
        row = (
            f"| {result['file_name']:<25} "
            f"| {result['mean']:>10.3f} "
            f"| {result['min']:>10.3f} "
            f"| {result['max']:>10.3f} "
            f"| {result['stdev']:>14.3f} |"
        )
        print(row)

    print("-------------------------\n")


def write_summary_csv(summary_data: list[SummaryData], file_path: str):
    """
    Writes the benchmark summary to a padded CSV file.

    Args:
        summary_data: A list of benchmark result dictionaries.
        file_path: The path to the output CSV file.
    """
    header = ["File", "Mean (ms)", "Min (ms)", "Max (ms)", "Std Dev (ms)"]

    data_grid = [header] + [
        [
            d['file_name'],
            f"{d['mean']:>9.3f}",
            f"{d['min']:>9.3f}",
            f"{d['max']:>9.3f}",
            f"{d['stdev']:>9.3f}",
        ]
        for d in summary_data
    ]

    col_widths = [max(len(item) for item in col) for col in zip(*data_grid)]

    try:
        with open(file_path, 'w', newline='') as f:
            for row in data_grid:
                formatted_cells = [cell.ljust(width)
                                   for cell, width in zip(row, col_widths)]
                line = ", ".join(formatted_cells)
                f.write(line + '\n')
        print(f"\nSuccessfully wrote benchmark summary to '{file_path}'")
    except IOError as e:
        print(f"\nError writing to file '{file_path}': {e}", file=sys.stderr)


def main():
    """
    Main function to run all configured benchmarks.
    """
    parser = argparse.ArgumentParser(
        description="A benchmark script for Linefeed scripts."
    )
    parser.add_argument(
        "--csv",
        type=str,
        help="Path to write the benchmark summary to a CSV file.",
    )
    args = parser.parse_args()

    summary_data: list[SummaryData] = []
    print(f"Starting benchmark suite. Running each benchmark {RUNS} times.")

    for linefeed_script, stdin_file in BENCHMARKS:
        run_times = run_benchmark(linefeed_script, stdin_file, RUNS)
        if run_times:
            stats = calculate_and_print_statistics(run_times)
            if stats:
                summary_item = SummaryData(
                    file_name=os.path.basename(linefeed_script), **stats)
                summary_data.append(summary_item)

    if summary_data:
        print_summary_table(summary_data)
        if args.csv:
            write_summary_csv(summary_data, args.csv)

    print("Benchmark suite finished.")


if __name__ == "__main__":
    main()
