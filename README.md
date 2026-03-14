# run_tests

A simple test runner for compiled executables.

## Usage

```bash
run_tests <executable> <tests_dir> [test_number]
```

- `executable`: Path to the program to test
- `tests_dir`: Directory containing `.in` and `.out` test files
- `test_number` (optional): Run a specific test

## Test File Format

Test files should be named with a 3-digit suffix:
- `xxx_001.in` - input file
- `xxx_001.out` - expected output

Example: `01_test_001.in` and `01_test_001.out`

## Example

```bash
run_tests ./my_program tests/
run_tests ./my_program tests/ 1
```

## Output

```
Running 3 test(s)...

test test_001 ... ok
test test_002 ... ok
test test_003 ... FAILED

test result: 2 passed; 1 failed
```

## Build

```bash
cargo build --release
```
