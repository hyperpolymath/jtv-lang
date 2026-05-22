// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Julia the Viper - Parser Benchmark
// Measures LOC/sec on synthetic JTV programs exercising the Harvard architecture
// (data language + control language).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use jtv_core::parse_program;

/// Generate a synthetic JTV program with the given number of lines.
/// Exercises both the data language (addition-only expressions) and
/// the control language (loops, conditionals, functions, modules).
fn generate_synthetic_program(lines: usize) -> String {
    let mut code = String::with_capacity(lines * 40);

    // Module header (~10% of lines)
    code.push_str("module Benchmark {\n");

    // Pure data functions (~15% of lines)
    let pure_fn_count = lines / 7;
    for i in 0..pure_fn_count {
        code.push_str(&format!(
            "    @pure fn add{}(a: Int, b: Int): Int {{\n        return a + b\n    }}\n\n",
            i
        ));
    }

    // Total function (~5% of lines)
    code.push_str("    @total fn identity(x: Int): Int {\n        return x\n    }\n\n");

    // Impure function with control flow (~20% of lines)
    code.push_str("    fn accumulate(n: Int): Int {\n");
    code.push_str("        result = 0\n");
    code.push_str("        for i in 1..n {\n");
    code.push_str("            result = result + i\n");
    code.push_str("        }\n");
    code.push_str("        return result\n");
    code.push_str("    }\n\n");

    // Close module
    code.push_str("}\n\n");
    code.push_str("import Benchmark\n\n");

    // Fill remaining lines with mixed statements
    let remaining = lines.saturating_sub(code.lines().count());
    for i in 0..remaining {
        match i % 8 {
            0 => code.push_str(&format!("x{} = {} + {}\n", i, i * 3, i * 7)),
            1 => code.push_str(&format!("y{} = Benchmark.add0({}, {})\n", i, i, i + 1)),
            2 => {
                code.push_str(&format!("if x{} > 0 {{\n", i.saturating_sub(2)));
                code.push_str(&format!("    z{} = x{} + 1\n", i, i.saturating_sub(2)));
                code.push_str("}\n");
            }
            3 => code.push_str(&format!("print({})\n", i)),
            4 => {
                code.push_str("reverse {\n");
                code.push_str(&format!("    acc += {}\n", i));
                code.push_str("}\n");
            }
            5 => {
                code.push_str(&format!("for j{} in 0..{} {{\n", i, i % 10 + 1));
                code.push_str(&format!("    sum{} = sum{} + j{}\n", i, i, i));
                code.push_str("}\n");
            }
            6 => code.push_str(&format!("w{} = [{}, {}, {}]\n", i, i, i + 1, i + 2)),
            7 => code.push_str(&format!("t{} = ({}, {})\n", i, i, i + 1)),
            _ => unreachable!(),
        }
    }

    code
}

fn bench_parse_simple_addition(c: &mut Criterion) {
    c.bench_function("parse simple addition", |b| {
        b.iter(|| {
            parse_program(black_box("x = 5 + 3")).unwrap();
        });
    });
}

fn bench_parse_function(c: &mut Criterion) {
    let code = r#"
        fn add(a: Int, b: Int): Int {
            return a + b
        }
    "#;

    c.bench_function("parse function", |b| {
        b.iter(|| {
            parse_program(black_box(code)).unwrap();
        });
    });
}

fn bench_parse_complex_program(c: &mut Criterion) {
    let code = r#"
        module Math {
            @pure fn add(a: Int, b: Int): Int {
                return a + b
            }

            fn factorial(n: Int): Int {
                result = 1
                for i in 2..n+1 {
                    result = result + i
                }
                return result
            }
        }

        import Math

        x = Math.add(5, 3)
        y = Math.factorial(10)
    "#;

    c.bench_function("parse complex program", |b| {
        b.iter(|| {
            parse_program(black_box(code)).unwrap();
        });
    });
}

fn bench_loc_per_second(c: &mut Criterion) {
    let mut group = c.benchmark_group("LOC/sec throughput");

    for size in [50, 100, 250, 500, 1000] {
        let code = generate_synthetic_program(size);
        let loc = code.lines().count() as u64;

        group.throughput(Throughput::Elements(loc));
        group.bench_with_input(BenchmarkId::from_parameter(size), &code, |b, code| {
            b.iter(|| {
                parse_program(black_box(code)).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_harvard_data_language_only(c: &mut Criterion) {
    // Pure data expressions: addition-only, no control flow.
    // Tests the total (provably halting) subset of the grammar.
    let mut group = c.benchmark_group("data language expressions");

    let expressions = [
        ("single_literal", "x = 42"),
        ("binary_add", "x = 1 + 2"),
        ("chain_5", "x = 1 + 2 + 3 + 4 + 5"),
        ("chain_10", "x = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10"),
        ("nested_parens", "x = ((1 + 2) + (3 + 4)) + ((5 + 6) + (7 + 8))"),
        ("mixed_numbers", "x = 42 + 3.14 + 0xFF + 0b1010"),
        ("list_literal", "x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"),
        ("function_calls", "x = add(1, 2) + multiply(3, 4)"),
    ];

    for (name, code) in &expressions {
        group.bench_function(*name, |b| {
            b.iter(|| {
                parse_program(black_box(code)).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_harvard_control_language(c: &mut Criterion) {
    // Control language: Turing-complete side with loops and conditionals.
    let mut group = c.benchmark_group("control language constructs");

    let constructs = [
        (
            "while_loop",
            "x = 0\nwhile x < 100 { x = x + 1 }",
        ),
        (
            "for_loop",
            "sum = 0\nfor i in 0..100 { sum = sum + i }",
        ),
        (
            "if_else",
            "if x > 0 { y = 1 } else { y = 0 }",
        ),
        (
            "nested_if",
            "if x > 0 { if y > 0 { z = 1 } else { z = 2 } } else { z = 3 }",
        ),
        (
            "reverse_block",
            "reverse { x += 1 y += 2 x += 3 y += 4 }",
        ),
    ];

    for (name, code) in &constructs {
        group.bench_function(*name, |b| {
            b.iter(|| {
                parse_program(black_box(code)).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_simple_addition,
    bench_parse_function,
    bench_parse_complex_program,
    bench_loc_per_second,
    bench_harvard_data_language_only,
    bench_harvard_control_language,
);
criterion_main!(benches);
