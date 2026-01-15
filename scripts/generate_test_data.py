#!/usr/bin/env python3
"""
Generate large test dataset for Beefcake memory profiling.
Creates a CSV file with configurable rows and columns.
"""

import csv
import random
import string
import sys
import os
from datetime import datetime, timedelta

def generate_test_csv(
    output_path: str,
    rows: int = 5_000_000,
    num_cols: int = 100,
):
    """Generate a large CSV file with mixed data types."""

    print(f"Generating test dataset: {rows:,} rows × {num_cols} columns")
    print(f"Output: {output_path}")
    print("")

    # Column types distribution
    numeric_cols = num_cols // 3
    categorical_cols = num_cols // 3
    text_cols = num_cols - numeric_cols - categorical_cols

    print(f"Column breakdown:")
    print(f"  - Numeric: {numeric_cols}")
    print(f"  - Categorical: {categorical_cols}")
    print(f"  - Text: {text_cols}")
    print("")

    # Generate column names
    headers = []
    headers += [f"numeric_{i}" for i in range(numeric_cols)]
    headers += [f"category_{i}" for i in range(categorical_cols)]
    headers += [f"text_{i}" for i in range(text_cols)]

    # Categorical value pools
    categories = [
        [f"Cat{i}_{j}" for j in range(random.randint(5, 50))]
        for i in range(categorical_cols)
    ]

    start_time = datetime.now()

    with open(output_path, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(headers)

        for row_num in range(rows):
            row = []

            # Numeric columns (floats with some nulls)
            for _ in range(numeric_cols):
                if random.random() < 0.05:  # 5% nulls
                    row.append('')
                else:
                    row.append(f"{random.gauss(100, 50):.2f}")

            # Categorical columns (low cardinality)
            for i in range(categorical_cols):
                if random.random() < 0.02:  # 2% nulls
                    row.append('')
                else:
                    row.append(random.choice(categories[i]))

            # Text columns (variable length strings)
            for _ in range(text_cols):
                if random.random() < 0.03:  # 3% nulls
                    row.append('')
                else:
                    length = random.randint(10, 50)
                    row.append(''.join(random.choices(string.ascii_letters + string.digits + ' ', k=length)))

            writer.writerow(row)

            # Progress indicator
            if (row_num + 1) % 100_000 == 0:
                elapsed = (datetime.now() - start_time).total_seconds()
                rate = (row_num + 1) / elapsed
                remaining = (rows - row_num - 1) / rate
                print(f"  Progress: {row_num + 1:,} / {rows:,} rows ({(row_num + 1) / rows * 100:.1f}%) - ETA: {remaining:.0f}s")

    elapsed = (datetime.now() - start_time).total_seconds()
    file_size_mb = os.path.getsize(output_path) / (1024 * 1024)

    print("")
    print(f"✓ Complete!")
    print(f"  Time: {elapsed:.1f}s")
    print(f"  File size: {file_size_mb:.1f} MB")
    print(f"  Rows/sec: {rows / elapsed:,.0f}")


if __name__ == "__main__":
    # Default: 5M rows × 100 columns (~2GB file)
    num_rows = 5_000_000
    num_cols = 100
    output = "test_5M_100cols.csv"

    # Parse command line args
    if len(sys.argv) > 1:
        num_rows = int(sys.argv[1])
    if len(sys.argv) > 2:
        num_cols = int(sys.argv[2])
    if len(sys.argv) > 3:
        output = sys.argv[3]

    print("=" * 60)
    print("Beefcake Test Data Generator")
    print("=" * 60)
    print("")

    generate_test_csv(output, num_rows, num_cols)

    print("")
    print(f"Ready to test! Run:")
    print(f"  .\\scripts\\measure_memory.ps1 -ProcessName beefcake -TestFile {output}")
