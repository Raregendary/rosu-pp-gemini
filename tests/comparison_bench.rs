use std::time::{Duration, Instant};
use rosu_pp::{Beatmap, Difficulty, Performance};

struct Stats {
    median: Duration,
    mean: Duration,
}

fn calc_stats(mut times: Vec<Duration>) -> Stats {
    times.sort();
    let median = times[times.len() / 2];
    let sum: Duration = times.iter().sum();
    let mean = sum / times.len() as u32;
    Stats { median, mean }
}

fn format_duration(d: Duration) -> String {
    let micros = d.as_secs_f64() * 1_000_000.0;
    if micros >= 1000.0 {
        format!("{:.2}ms", micros / 1000.0)
    } else {
        format!("{:.2}µs", micros)
    }
}

#[test]
fn test_bench_comparison() {
    let target_paths = [
        ("5525390.osu (osu! standard)", "./resources/5525390.osu"),
        ("3700073.osu (osu! standard)", "./resources/3700073.osu"),
        ("5727828.osu (osu!taiko)", "./resources/5727828.osu"),
        ("4384622.osu (osu!catch)", "./resources/4384622.osu"),
        ("5873946.osu (osu!mania)", "./resources/5873946.osu"),
    ];

    println!("\n================ rosu-pp Benchmark ================");

    for (name, path) in target_paths {
        println!("\n--- Benchmarking: {} ---", name);

        // Warmup decode
        for _ in 0..5 {
            let _ = Beatmap::from_path(path).unwrap();
        }

        // Decode 50 iters
        let decode_iters = 50;
        let mut decode_times = Vec::with_capacity(decode_iters);
        for _ in 0..decode_iters {
            let t0 = Instant::now();
            let _ = Beatmap::from_path(path).unwrap();
            decode_times.push(t0.elapsed());
        }

        let map = Beatmap::from_path(path).unwrap();

        // Warmup difficulty
        for _ in 0..5 {
            let _ = Difficulty::new().calculate(&map);
        }

        // Difficulty 50 iters
        let diff_iters = 50;
        let mut diff_times = Vec::with_capacity(diff_iters);
        for _ in 0..diff_iters {
            let t0 = Instant::now();
            let _ = Difficulty::new().calculate(&map);
            diff_times.push(t0.elapsed());
        }

        let diff_attrs = Difficulty::new().calculate(&map);

        // Warmup performance
        for _ in 0..20 {
            let _ = Performance::new(diff_attrs.clone()).calculate();
        }

        // Performance 500 iters
        let perf_iters = 500;
        let mut perf_times = Vec::with_capacity(perf_iters);
        for _ in 0..perf_iters {
            let t0 = Instant::now();
            let _ = Performance::new(diff_attrs.clone()).calculate();
            perf_times.push(t0.elapsed());
        }

        let perf_attrs = Performance::new(diff_attrs.clone()).calculate();

        let decode_stats = calc_stats(decode_times);
        let diff_stats = calc_stats(diff_times);
        let perf_stats = calc_stats(perf_times);

        println!("Stars:       {:.2}", diff_attrs.stars());
        println!("PP:          {:.2}", perf_attrs.pp());
        println!(
            "Decoding:    Median: {} | Mean: {}",
            format_duration(decode_stats.median),
            format_duration(decode_stats.mean)
        );
        println!(
            "Difficulty:  Median: {} | Mean: {}",
            format_duration(diff_stats.median),
            format_duration(diff_stats.mean)
        );
        println!(
            "Performance: Median: {} | Mean: {}",
            format_duration(perf_stats.median),
            format_duration(perf_stats.mean)
        );
    }
}
