use std::{hint::black_box, io::sink, sync::Arc};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wn::{ast::Stmt, lexer::tokenizar, parser::parsear};
use wn_diagnostics::SourceFile;
use wn_vm::{chunk::Chunk, compiler::compilar, vm::VM};

struct BenchCase {
    name: &'static str,
    workload_units: u64,
    src: &'static str,
}

const BENCH_CASES: &[BenchCase] = &[
    BenchCase {
        name: "loop_sum_100k",
        workload_units: 100_000,
        src: r#"
wea i = 0
wea suma = 0

mientras (i < 100000) {
  suma = suma + i
  i = i + 1
}

suma
"#,
    },
    BenchCase {
        name: "fib_iter_35",
        workload_units: 35,
        src: r#"
wea a = 0
wea b = 1
wea i = 0

mientras (i < 35) {
  wea tmp = b
  b = a + b
  a = tmp
  i = i + 1
}

a
"#,
    },
    BenchCase {
        name: "function_calls_10k",
        workload_units: 10_000,
        src: r#"
pega sumar_hasta(n) {
  wea total = 0
  wea i = 0
  mientras (i < n) {
    total = total + i
    i = i + 1
  }
  devolver total
}

wea acc = 0
wea i = 0
mientras (i < 10000) {
  acc = acc + sumar_hasta(10)
  i = i + 1
}

acc
"#,
    },
    BenchCase {
        name: "closure_calls_10k",
        workload_units: 10_000,
        src: r#"
pega crear_sumador(base) {
  pega sumar(x) {
    devolver base + x
  }
  devolver sumar
}

wea sumar5 = crear_sumador(5)
wea acc = 0
wea i = 0
mientras (i < 10000) {
  acc = acc + sumar5(i)
  i = i + 1
}

acc
"#,
    },
    BenchCase {
        name: "list_iteration_1k",
        workload_units: 1_000,
        src: r#"
wea xs = [
  1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
  11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
  21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
  31, 32, 33, 34, 35, 36, 37, 38, 39, 40
]
wea outer = 0
wea total = 0

mientras (outer < 1000) {
  para (x en xs) {
    total = total + x
  }
  outer = outer + 1
}

total
"#,
    },
];

fn parse_src(src: &str, filename: &str) -> Vec<Stmt> {
    let tokens = tokenizar(src).expect("tokenizacion de benchmark");
    parsear(tokens, src, filename).expect("parseo de benchmark")
}

fn compile_parsed(stmts: &[Stmt], src: &str, filename: &str) -> Chunk {
    let source = Arc::new(SourceFile::new(filename, src));
    compilar(stmts, source).expect("compilacion de benchmark")
}

fn compile_src(src: &str, filename: &str) -> Chunk {
    let stmts = parse_src(src, filename);
    compile_parsed(&stmts, src, filename)
}

fn bench_vm_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_compile");

    for case in BENCH_CASES {
        let filename = format!("<bench-compile:{}>", case.name);
        let stmts = parse_src(case.src, &filename);
        group.throughput(Throughput::Elements(case.workload_units));
        group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |b, case| {
            b.iter(|| {
                black_box(compile_parsed(
                    black_box(&stmts),
                    black_box(case.src),
                    black_box(&filename),
                ));
            })
        });
    }

    group.finish();
}

fn bench_vm_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_run");

    for case in BENCH_CASES {
        let filename = format!("<bench-run:{}>", case.name);
        let chunk = compile_src(case.src, &filename);
        group.throughput(Throughput::Elements(case.workload_units));
        group.bench_with_input(BenchmarkId::from_parameter(case.name), case, |b, case| {
            b.iter(|| {
                let mut vm = VM::con_salida(sink());
                let result = vm.run(black_box(&chunk)).expect("ejecucion de benchmark");
                black_box((case.workload_units, result));
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_vm_compile, bench_vm_run);
criterion_main!(benches);
