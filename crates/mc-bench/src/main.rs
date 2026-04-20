use std::env;
use std::fs;

use mc_bench::run_default_benchmarks;

fn main() {
    let output = parse_output_path();
    let report = run_default_benchmarks();
    let json = serde_json::to_string_pretty(&report)
        .expect("benchmark report serialization should succeed");

    if let Some(path) = output {
        fs::write(&path, json).expect("writing benchmark output should succeed");
        println!("Benchmark report written to {path}");
    } else {
        println!("{json}");
    }
}

fn parse_output_path() -> Option<String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output" {
            return args.next();
        }
    }
    None
}
