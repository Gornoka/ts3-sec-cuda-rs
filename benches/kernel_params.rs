use ts3_sec_cuda_rs::hashers::cuda::CudaHasher;
use std::time::{Duration, Instant};

/// Benchmark configuration
const WARMUP_ITERATIONS: usize = 1000;
const BENCHMARK_ITERATIONS: usize = 10000;

struct BenchResult {
    threads_per_block: usize,
    shared_mem_bytes: usize,
    batch_size: usize,
    mean: Duration,
    median: Duration,
    min: Duration,
    max: Duration,
}

fn benchmark_config(
    hasher: &CudaHasher,
    public_key: &str,
    start_counter: u64,
    threads_per_block: usize,
    shared_mem_bytes: Option<usize>,
    batch_size: usize,
) -> Option<BenchResult> {
    let actual_shared_mem = shared_mem_bytes.unwrap_or(threads_per_block * 128);

    // Warmup - test if configuration is valid
    for _ in 0..WARMUP_ITERATIONS {
        if hasher.calculate_levels_optimized_with_params(
            public_key, start_counter, batch_size, threads_per_block, shared_mem_bytes
        ).is_err() {
            return None; // Invalid configuration
        }
    }

    // Benchmark
    let mut timings = Vec::with_capacity(BENCHMARK_ITERATIONS);
    for _ in 0..BENCHMARK_ITERATIONS {
        let start = Instant::now();
        let _ = hasher.calculate_levels_optimized_with_params(
            public_key, start_counter, batch_size, threads_per_block, shared_mem_bytes
        ).unwrap();
        timings.push(start.elapsed());
    }

    // Calculate statistics
    timings.sort();
    let min = timings[0];
    let max = timings[timings.len() - 1];
    let median = timings[timings.len() / 2];
    let mean = timings.iter().sum::<Duration>() / timings.len() as u32;

    Some(BenchResult {
        threads_per_block,
        shared_mem_bytes: actual_shared_mem,
        batch_size,
        mean,
        median,
        min,
        max,
    })
}

fn print_result(result: &BenchResult, label: &str) {
    println!("{:20} threads={:4} shared_mem={:6} batch={:7} mean={:>9.2?} median={:>9.2?} min={:>9.2?} max={:>9.2?} throughput={:>12.2} h/s",
        label,
        result.threads_per_block,
        result.shared_mem_bytes,
        result.batch_size,
        result.mean,
        result.median,
        result.min,
        result.max,
        result.batch_size as f64 / result.mean.as_secs_f64()
    );
}

fn main() {
    println!("\n{}", "=".repeat(160));
    println!("CUDA Kernel Parameter Benchmark - Testing Different Configurations");
    println!("{}", "=".repeat(160));

    // Initialize CUDA hasher
    println!("\nInitializing CUDA hasher...");
    let hasher = CudaHasher::new().expect("Failed to initialize CUDA hasher");
    println!("✓ CUDA hasher initialized\n");

    // Short message (fast path: ~14-16 bytes total)
    let short_key = "test_key_123";
    // Long message (slow path: ~109-113 bytes total)
    let long_key = "ME0DAgcAAgEgAiEAy/hhqSBja7A6FTZG5s+BMnQfCqYyS9sGsbyMKBb7spYCIQCBEtZWrZtewnxuh2hsigJswGHchu3XcaiQDZziMsxTsA==";

    // Test configurations
    let thread_configs = [32, 64, 128, 256];
    let batch_sizes = [50_000, 100_000, 200_000, 500_000, 1_000_000, 2_000_000, 4_000_000, 8_000_000, 16_000_000];

    println!("{}", "=".repeat(160));
    println!("SHORT MESSAGES (Fast Path: Single-block SHA1, ~14-16 bytes)");
    println!("{}", "=".repeat(160));
    let mut short_results = Vec::new();
    for &batch_size in &batch_sizes {
        println!("\nBatch size: {}", batch_size);
        println!("{}", "-".repeat(160));
        for &threads in &thread_configs {
            if let Some(result) = benchmark_config(&hasher, short_key, 0, threads, None, batch_size) {
                print_result(&result, "");
                short_results.push(result);
            } else {
                println!("{:20} threads={:4} shared_mem={:6} batch={:7} SKIPPED (invalid configuration)", "", threads, threads * 128, batch_size);
            }
        }
    }

    println!("\n{}", "=".repeat(160));
    println!("LONG MESSAGES (Slow Path: Multi-block SHA1, ~109-113 bytes)");
    println!("{}", "=".repeat(160));
    let mut long_results = Vec::new();
    for &batch_size in &batch_sizes {
        println!("\nBatch size: {}", batch_size);
        println!("{}", "-".repeat(160));
        for &threads in &thread_configs {
            if let Some(result) = benchmark_config(&hasher, long_key, 0, threads, None, batch_size) {
                print_result(&result, "");
                long_results.push(result);
            } else {
                println!("{:20} threads={:4} shared_mem={:6} batch={:7} SKIPPED (invalid configuration)", "", threads, threads * 128, batch_size);
            }
        }
    }

    // Find best configurations by throughput
    let best_short = short_results.iter()
        .max_by(|a, b| {
            let throughput_a = a.batch_size as f64 / a.mean.as_secs_f64();
            let throughput_b = b.batch_size as f64 / b.mean.as_secs_f64();
            throughput_a.partial_cmp(&throughput_b).unwrap()
        })
        .unwrap();
    let best_long = long_results.iter()
        .max_by(|a, b| {
            let throughput_a = a.batch_size as f64 / a.mean.as_secs_f64();
            let throughput_b = b.batch_size as f64 / b.mean.as_secs_f64();
            throughput_a.partial_cmp(&throughput_b).unwrap()
        })
        .unwrap();

    println!("\n{}", "=".repeat(160));
    println!("SUMMARY - Best Throughput Configurations");
    println!("{}", "=".repeat(160));
    print_result(best_short, "Best (short):");
    print_result(best_long, "Best (long):");
    println!("{}", "=".repeat(160));
    println!("\nBenchmark settings:");
    println!("  Warmup iterations:     {}", WARMUP_ITERATIONS);
    println!("  Benchmark iterations:  {}", BENCHMARK_ITERATIONS);
    println!("{}", "=".repeat(160));
}
