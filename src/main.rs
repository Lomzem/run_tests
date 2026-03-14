use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command as ProcCommand, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const DEFAULT_TIMEOUT_MS: u64 = 10000;

#[derive(Parser)]
#[command(name = "run_tests")]
#[command(about = "Run tests for a compiled executable", long_about = None)]
struct Args {
    #[arg(help = "Path to the executable to test")]
    executable: String,

    #[arg(help = "Path to the directory containing test files")]
    tests_dir: String,
}

struct TestCase {
    input_file: String,
    output_file: String,
}

fn extract_test_number(filename: &str) -> Option<String> {
    let path = Path::new(filename);
    let stem = path.file_stem()?.to_str()?;
    let digits: String = stem.chars().rev().take(3).collect();
    let digits: String = digits.chars().rev().collect();
    if digits.chars().all(|c| c.is_ascii_digit()) {
        Some(digits)
    } else {
        None
    }
}

fn discover_tests(tests_dir: &Path) -> Vec<TestCase> {
    let mut in_files: HashMap<String, String> = HashMap::new();
    let mut out_files: HashMap<String, String> = HashMap::new();

    let entries = fs::read_dir(tests_dir).expect("Failed to read tests directory");

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.ends_with(".in") {
                if let Some(num) = extract_test_number(filename) {
                    in_files.insert(num, filename.to_string());
                }
            } else if filename.ends_with(".out") {
                if let Some(num) = extract_test_number(filename) {
                    out_files.insert(num, filename.to_string());
                }
            }
        }
    }

    let mut tests: Vec<TestCase> = in_files
        .into_iter()
        .filter_map(|(num, input_file)| {
            out_files.get(&num).map(|output_file| TestCase {
                input_file,
                output_file: output_file.clone(),
            })
        })
        .collect();

    tests.sort_by(|a, b| a.input_file.cmp(&b.input_file));
    tests
}

fn run_test(executable: &Path, input_path: &Path, output_path: &Path, timeout_ms: u64) -> bool {
    let input_content = fs::read(input_path).expect("Failed to read input file");
    let expected_output = fs::read(output_path).expect("Failed to read output file");

    let mut child = ProcCommand::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(&input_content)
                .expect("Failed to write to stdin");
        }
    }

    let status = match child.wait_timeout(Duration::from_millis(timeout_ms)) {
        Ok(Some(s)) => s,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        Err(_) => return false,
    };

    use std::io::Read;
    let mut stdout_buf = Vec::new();
    if let Some(ref mut stdout) = child.stdout {
        stdout.read_to_end(&mut stdout_buf).unwrap();
    }

    let mut stderr_buf = Vec::new();
    if let Some(ref mut stderr) = child.stderr {
        stderr.read_to_end(&mut stderr_buf).unwrap();
    }

    if !status.success() || !stderr_buf.is_empty() {
        return false;
    }

    stdout_buf == expected_output
}

fn main() {
    let args = Args::parse();

    let executable = Path::new(&args.executable);
    if !executable.exists() {
        eprintln!("Error: Executable '{}' not found", args.executable);
        std::process::exit(1);
    }

    let tests_dir = Path::new(&args.tests_dir);
    if !tests_dir.is_dir() {
        eprintln!("Error: Tests directory '{}' not found", args.tests_dir);
        std::process::exit(1);
    }

    let tests = discover_tests(tests_dir);

    if tests.is_empty() {
        println!("No tests found");
        std::process::exit(0);
    }

    println!("Running {} test(s)...\n", tests.len());

    let mut passed = 0;
    let mut failed = 0;

    for test in &tests {
        let input_path = tests_dir.join(&test.input_file);
        let output_path = tests_dir.join(&test.output_file);

        let test_name = test.input_file.replace(".in", "");
        let result = run_test(executable, &input_path, &output_path, DEFAULT_TIMEOUT_MS);

        if result {
            println!("test {} ... ok", test_name);
            passed += 1;
        } else {
            println!("test {} ... FAILED", test_name);
            failed += 1;
        }
    }

    println!("\ntest result: {} passed; {} failed", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
