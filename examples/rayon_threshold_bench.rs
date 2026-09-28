use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use rosu_map::section::general::GameMode;
use rosu_pp::{Beatmap, Difficulty};

struct BenchmarkResult {
    file_name: String,
    object_count: usize,
    normal_ms: f64,
    rayon_ms: f64,
    diff_pct: f64,
}

fn collect_osu_files(dir: &Path, list: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_osu_files(&path, list);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("osu") {
                list.push(path);
            }
        }
    }
}

fn main() {
    let songs_dir = Path::new("..").join("example-osu-files");
    let mut files = Vec::new();
    collect_osu_files(&songs_dir, &mut files);

    println!("Found {} .osu files to test.", files.len());
    if files.is_empty() {
        eprintln!("No files found in {:?}", songs_dir);
        return;
    }

    // Warm up Rayon thread pool
    #[cfg(feature = "rayon")]
    {
        rayon::ThreadPoolBuilder::new().build_global().ok();
        rayon::spawn(|| {});
    }

    let mut results: Vec<BenchmarkResult> = Vec::new();
    let iters = 3;

    for (i, path) in files.iter().enumerate() {
        if i % 100 == 0 && i > 0 {
            println!("Processed {} / {} maps...", i, files.len());
        }

        let Ok(map) = Beatmap::from_path(path) else {
            continue;
        };

        if map.mode != GameMode::Osu {
            continue;
        }

        let object_count = map.hit_objects.len();
        if object_count == 0 {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let diff_seq = Difficulty::new().parallel(false);
        let diff_par = Difficulty::new().parallel(true);

        // Warmup
        let _ = diff_seq.calculate(&map);
        let _ = diff_par.calculate(&map);

        // Benchmark normal (sequential)
        let mut seq_dur = Duration::ZERO;
        for _ in 0..iters {
            let t0 = Instant::now();
            let _ = diff_seq.calculate(&map);
            seq_dur += t0.elapsed();
        }
        let normal_ms = (seq_dur.as_secs_f64() * 1000.0) / (iters as f64);

        // Benchmark rayon (parallel)
        let mut par_dur = Duration::ZERO;
        for _ in 0..iters {
            let t0 = Instant::now();
            let _ = diff_par.calculate(&map);
            par_dur += t0.elapsed();
        }
        let rayon_ms = (par_dur.as_secs_f64() * 1000.0) / (iters as f64);

        // diff %: positive means rayon is faster by X%
        let diff_pct = if normal_ms > 0.0 {
            ((normal_ms - rayon_ms) / normal_ms) * 100.0
        } else {
            0.0
        };

        results.push(BenchmarkResult {
            file_name,
            object_count,
            normal_ms,
            rayon_ms,
            diff_pct,
        });
    }

    // Sort by object count ascending
    results.sort_by_key(|r| r.object_count);

    // Save to CSV
    let csv_path = Path::new("..").join("rayon_benchmark_results.csv");
    let file = File::create(&csv_path).expect("Failed to create CSV output file");
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "beatmap,object count,normal implementation (ms),rayon (ms),diff%"
    )
    .unwrap();
    for r in &results {
        // Escape commas in beatmap file names
        let escaped_name = format!("\"{}\"", r.file_name.replace('\"', "\"\""));
        writeln!(
            writer,
            "{},{},{:.4},{:.4},{:.2}%",
            escaped_name, r.object_count, r.normal_ms, r.rayon_ms, r.diff_pct
        )
        .unwrap();
    }
    writer.flush().unwrap();

    println!("\nCSV saved to: {:?}", csv_path);
    println!("\n================ SUMMARY BY OBJECT COUNT BUCKETS ================");
    println!(
        "{:<15} {:<10} {:<15} {:<15} {:<10}",
        "Bucket", "Maps", "Avg Normal(ms)", "Avg Rayon(ms)", "Avg Speedup"
    );
    println!("{}", "-".repeat(70));

    let buckets = [
        (0, 150, "0 - 150"),
        (150, 300, "150 - 300"),
        (300, 500, "300 - 500"),
        (500, 1000, "500 - 1000"),
        (1000, 1500, "1000 - 1500"),
        (1500, 2500, "1500 - 2500"),
        (2500, usize::MAX, "2500+"),
    ];

    for (low, high, label) in buckets {
        let matching: Vec<&BenchmarkResult> = results
            .iter()
            .filter(|r| r.object_count >= low && r.object_count < high)
            .collect();

        if matching.is_empty() {
            continue;
        }

        let count = matching.len();
        let avg_normal: f64 = matching.iter().map(|r| r.normal_ms).sum::<f64>() / count as f64;
        let avg_rayon: f64 = matching.iter().map(|r| r.rayon_ms).sum::<f64>() / count as f64;
        let avg_diff: f64 = matching.iter().map(|r| r.diff_pct).sum::<f64>() / count as f64;

        println!(
            "{:<15} {:<10} {:<15.3} {:<15.3} {:>+.1}%",
            label, count, avg_normal, avg_rayon, avg_diff
        );
    }
}
