use rosu_pp::{
    Beatmap,
    Difficulty,
    Performance,
};

#[test]
fn test_all_four_benchmark_maps() {
    // 1. osu! standard
    let map_osu = Beatmap::from_path("./resources/5525390.osu").expect("load osu map");
    let diff_osu = Difficulty::new().calculate(&map_osu);
    let perf_osu = Performance::new(diff_osu.clone()).calculate();
    println!("=== osu! Standard (5525390) ===");
    println!("Stars: {:.2} (expected 6.24)", diff_osu.stars());
    println!("PP: {:.2}", perf_osu.pp());

    // 2. osu!taiko
    let map_taiko = Beatmap::from_path("./resources/5727828.osu").expect("load taiko map");
    let diff_taiko = Difficulty::new().calculate(&map_taiko);
    let perf_taiko = Performance::new(diff_taiko.clone()).calculate();
    println!("=== osu!taiko (5727828) ===");
    println!("Stars: {:.2} (expected 7.59)", diff_taiko.stars());
    println!("PP: {:.2}", perf_taiko.pp());

    // 3. osu!catch
    let map_catch = Beatmap::from_path("./resources/4384622.osu").expect("load catch map");
    let diff_catch = Difficulty::new().calculate(&map_catch);
    let perf_catch = Performance::new(diff_catch.clone()).calculate();
    println!("=== osu!catch (4384622) ===");
    println!("Stars: {:.2} (expected 6.02)", diff_catch.stars());
    println!("PP: {:.2}", perf_catch.pp());

    // 4. osu!mania
    let map_mania = Beatmap::from_path("./resources/5873946.osu").expect("load mania map");
    let diff_mania = Difficulty::new().calculate(&map_mania);
    let perf_mania = Performance::new(diff_mania.clone()).calculate();
    println!("=== osu!mania (5873946) ===");
    println!("Stars: {:.2} (expected 5.91)", diff_mania.stars());
    println!("PP: {:.2}", perf_mania.pp());
}
