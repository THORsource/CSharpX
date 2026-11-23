import csv
import os
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BUILD = ROOT / "build"
RUNS = 9
SEARCHES = 1000
COUNT = 1000

def expected_primes():
    limit = 10000
    is_prime = [True] * limit
    is_prime[0:2] = [False, False]
    for candidate in range(2, limit):
        if is_prime[candidate]:
            for multiple in range(candidate * candidate, limit, candidate):
                is_prime[multiple] = False
    return [index for index, prime in enumerate(is_prime) if prime][:COUNT]

def main():
    commands = {
        "C": [str(BUILD / "primes-c.exe")],
        "C#": ["dotnet", str(BUILD / "dotnet/Primes.dll")],
        "Rust": [str(BUILD / "primes-rust.exe")],
        "CSharpX": [str(BUILD / "primes-csx.exe")],
    }
    programs = list(commands)
    expected = expected_primes()
    env = dict(os.environ, DOTNET_TieredCompilation="0")
    timings = {name: [] for name in programs}
    rows = []
    for round_index in range(RUNS):
        for offset in range(len(programs)):
            name = programs[(round_index + offset) % len(programs)]
            output = subprocess.check_output(commands[name], cwd=ROOT, env=env, timeout=120)
            ticks, frequency, checksum, *primes = map(int, output.splitlines())
            if primes != expected or checksum != sum(expected) * SEARCHES:
                raise ValueError(f"{name} produced an incorrect prime list or checksum")
            if min(ticks, frequency) <= 0:
                raise ValueError(f"{name} produced an invalid timing")
            microseconds = 1_000_000 * ticks / frequency / SEARCHES
            timings[name].append(microseconds)
            rows.append((round_index + 1, name, ticks, frequency, checksum, f"{microseconds:.6f}"))
    with (BUILD / "messwerte.csv").open("w", newline="", encoding="utf-8") as file:
        writer = csv.writer(file)
        writer.writerow(("round", "language", "ticks", "frequency_hz", "checksum", "microseconds_per_search"))
        writer.writerows(rows)
    for name, values in timings.items():
        print(f"{name:8} {statistics.median(values):.2f} {min(values):.2f} {max(values):.2f}")
    print(f"Verified {COUNT} primes, last {expected[-1]}, sum {sum(expected)}")

if __name__ == "__main__":
    main()
