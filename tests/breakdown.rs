use std::time::Instant;
use rosu_pp::{Beatmap, Difficulty, Performance};

#[test]
fn test_breakdown() {
    let maps = [
        ("osu! Standard (5525390, 6.24*)", "./resources/5525390.osu"),
        ("osu! Standard (3700073, 8.73*)", "./resources/3700073.osu"),
        ("osu!taiko (5727828, 7.59*)", "./resources/5727828.osu"),
        ("osu!catch (4384622, 6.02*)", "./resources/4384622.osu"),
        ("osu!mania (5873946, 5.91*)", "./resources/5873946.osu"),
    ];

    for (name, path) in maps {
        println!("\n================ {} ================", name);

        // Warmup
        let _ = Beatmap::from_path(path).unwrap();

        // 1. Decode timing (100 iters)
        let iters = 50;
        let t0 = Instant::now();
        for _ in 0..iters {
            let _ = Beatmap::from_path(path).unwrap();
        }
        let decode_time = t0.elapsed() / iters;

        let map = Beatmap::from_path(path).unwrap();

        // 2. Difficulty timing (50 iters)
        let t0 = Instant::now();
        for _ in 0..iters {
            let _ = Difficulty::new().calculate(&map);
        }
        let diff_time = t0.elapsed() / iters;

        let diff_attrs = Difficulty::new().calculate(&map);

        // 3. Performance timing (500 iters)
        let p_iters = 500;
        let t0 = Instant::now();
        for _ in 0..p_iters {
            let _ = Performance::new(diff_attrs.clone()).calculate();
        }
        let perf_time = t0.elapsed() / p_iters;

        println!("Decode:      {:?}", decode_time);
        println!("Difficulty:  {:?}", diff_time);
        println!("Performance: {:?}", perf_time);
    }
}
