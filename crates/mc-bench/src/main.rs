use std::env;
use std::fs;

use mc_bench::{build_competitiveness_plan, run_benchmarks, BenchmarkSuite};

fn main() {
    let args = parse_args();
    let report = run_benchmarks(args.suite);
    let json = serde_json::to_string_pretty(&report)
        .expect("benchmark report serialization should succeed");

    if let Some(path) = args.output {
        fs::write(&path, json).expect("writing benchmark output should succeed");
        println!("Benchmark report written to {path}");
        if args.suite == BenchmarkSuite::Full {
            let plan_path = "benchmarks/improvement-plan.md";
            let plan = build_competitiveness_plan(&report);
            fs::write(plan_path, plan).expect("writing competitiveness plan should succeed");
            println!("Competitiveness plan written to {plan_path}");
        } else {
            println!("Compact profile selected; competitiveness plan was not overwritten");
        }
    } else {
        println!("{json}");
    }
}

#[derive(Debug, Clone)]
struct Args {
    output: Option<String>,
    suite: BenchmarkSuite,
}

fn parse_args() -> Args {
    let mut output = None;
    let mut suite = BenchmarkSuite::Full;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output = args.next();
            }
            "--compact" => {
                suite = BenchmarkSuite::Compact;
            }
            "--profile" => {
                let profile = args.next().expect("--profile requires a value");
                suite = match profile.as_str() {
                    "full" => BenchmarkSuite::Full,
                    "compact" => BenchmarkSuite::Compact,
                    _ => panic!(
                        "unknown benchmark profile '{profile}', expected 'full' or 'compact'"
                    ),
                };
            }
            _ => {}
        }
    }
    Args { output, suite }
}
