import re
import statistics
import subprocess
import sys
from typing import List, Optional, Tuple

# --- Configuration ---

# Number of times to run each benchmark
RUNS = 5

# List of benchmarks to run. Each entry is a tuple:
# (path_to_linefeed_script, path_to_stdin_file)
BENCHMARKS: List[Tuple[str, str]] = [
    (
        "tests/linefeed/advent_of_code_2020/day01.lf",
        "tests/linefeed/advent_of_code_2020/inputs/day01.txt",
    ),
    (
        "tests/linefeed/advent_of_code_2020/day02.lf",
        "tests/linefeed/advent_of_code_2020/inputs/day02.txt",
    ),
    (
        "tests/linefeed/advent_of_code_2020/day03.lf",
        "tests/linefeed/advent_of_code_2020/inputs/day03.txt",
    ),
]


def run_benchmark(
    linefeed_script: str,
    stdin_file: str,
    runs: int
) -> Optional[List[float]]:
    """
    Runs the benchmark for the given Linefeed script.

    Args:
        linefeed_script: The path to the Linefeed script to benchmark.
        stdin_file: The path to the file to use as stdin.
        runs: The number of times to run the benchmark.

    Returns:
        A list of run times in seconds, or None if an error occurred.
    """
    run_times: List[float] = []
    print(f"\nBenchmarking '{linefeed_script}' with stdin from '{stdin_file}'...")

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
                else: # seconds
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
            print(f"Error: Input file '{stdin_file}' not found.", file=sys.stderr)
            return None
        except subprocess.CalledProcessError as e:
            print(f"Error running Linefeed script: {e}", file=sys.stderr)
            print(f"Stderr:\n{e.stderr}", file=sys.stderr)
            return None
        except Exception as e:
            print(f"An unexpected error occurred: {e}", file=sys.stderr)
            return None

    return run_times

def print_statistics(run_times: List[float]):
    """
    Prints the statistics for the given run times.

    Args:
        run_times: A list of run times in seconds.
    """
    if not run_times:
        print("No successful runs to analyze.")
        return

    mean = statistics.mean(run_times)
    if len(run_times) > 1:
        stdev = statistics.stdev(run_times)
    else:
        stdev = 0

    print("\n  --- Statistics ---")
    print(f"  Total runs: {len(run_times)}")
    print(f"  Min time:   {min(run_times):.3f}ms")
    print(f"  Max time:   {max(run_times):.3f}ms")
    print(f"  Mean time:  {mean:.3f}ms")
    print(f"  Std dev:    {stdev:.3f}ms")
    print("  ------------------")


def main():
    """
    Main function to run all configured benchmarks.
    """
    print(f"Starting benchmark suite. Running each benchmark {RUNS} times.")
    for linefeed_script, stdin_file in BENCHMARKS:
        run_times = run_benchmark(linefeed_script, stdin_file, RUNS)
        if run_times:
            print_statistics(run_times)
    print("\nBenchmark suite finished.")


if __name__ == "__main__":
    main()
