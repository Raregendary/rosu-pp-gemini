use rosu_pp::{
    Beatmap,
    osu::OsuPerformance,
};

#[test]
fn test_3700073() {
    let map = Beatmap::from_path("./resources/3700073.osu").expect("load map");

    let perf_stable = OsuPerformance::new(&map)
        .mods(8) // HD
        .lazer(false) // stable / not in lazer
        .combo(3986)
        .n300(3340)
        .n100(59)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    let perf_lazer = OsuPerformance::new(&map)
        .mods(8) // HD
        .lazer(true) // lazer
        .combo(3986)
        .n300(3340)
        .n100(59)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("=== 3700073 HD Stable (lazer(false)) ===");
    println!("Total PP: {:.4}", perf_stable.pp);
    println!("Aim PP: {:.4}", perf_stable.pp_aim);
    println!("Speed PP: {:.4}", perf_stable.pp_speed);
    println!("Accuracy PP: {:.4}", perf_stable.pp_acc);
    println!("Reading PP: {:.4}", perf_stable.pp_reading);

    println!("=== 3700073 HD Lazer (lazer(true)) ===");
    println!("Total PP: {:.4}", perf_lazer.pp);
    println!("Aim PP: {:.4}", perf_lazer.pp_aim);
    println!("Speed PP: {:.4}", perf_lazer.pp_speed);
    println!("Accuracy PP: {:.4}", perf_lazer.pp_acc);
    println!("Reading PP: {:.4}", perf_lazer.pp_reading);
    println!("Stars: {:.4}", perf_lazer.difficulty.stars);
    println!("Aim diff: {:.4}", perf_lazer.difficulty.aim);
    println!("Speed diff: {:.4}", perf_lazer.difficulty.speed);
    println!("Reading diff: {:.4}", perf_lazer.difficulty.reading);
    println!("Speed deviation: {:?}", perf_lazer.speed_deviation);

    assert!((perf_stable.pp - 1016.34).abs() < 0.1, "Stable PP mismatch: {}", perf_stable.pp);
    assert!((perf_lazer.pp - 1022.52).abs() < 0.1, "Lazer PP mismatch: {}", perf_lazer.pp);
}

#[test]
fn test_4904540() {
    let map = Beatmap::from_path("./resources/4904540.osu").expect("load map");

    let perf = OsuPerformance::new(&map)
        .lazer(true)
        .combo(2666)
        .n300(2224)
        .n100(14)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("=== 4904540 NM Lazer Performance ===");
    println!("Total PP: {:.4}", perf.pp);
    println!("Aim PP: {:.4}", perf.pp_aim);
    println!("Speed PP: {:.4}", perf.pp_speed);
    println!("Accuracy PP: {:.4}", perf.pp_acc);
    println!("Reading PP: {:.4}", perf.pp_reading);
    println!("Flashlight PP: {:.4}", perf.pp_flashlight);
    println!("Stars: {:.4}", perf.difficulty.stars);
    println!("Aim difficulty: {:.4}", perf.difficulty.aim);
    println!("Speed difficulty: {:.4}", perf.difficulty.speed);
    println!("Reading difficulty: {:.4}", perf.difficulty.reading);
    println!("Effective miss count: {:.4}", perf.effective_miss_count);
    println!("Speed deviation: {:?}", perf.speed_deviation);
    println!("Aim estimated slider breaks: {:.4}", perf.aim_estimated_slider_breaks);
    println!("Speed estimated slider breaks: {:.4}", perf.speed_estimated_slider_breaks);

    assert!((perf.pp - 1261.85).abs() < 0.1, "PP mismatch: {}", perf.pp);
    assert!((perf.pp_aim - 751.47).abs() < 0.1, "Aim PP mismatch: {}", perf.pp_aim);
    assert!((perf.pp_speed - 266.10).abs() < 0.1, "Speed PP mismatch: {}", perf.pp_speed);
    assert!((perf.pp_acc - 184.63).abs() < 0.1, "Acc PP mismatch: {}", perf.pp_acc);
    assert!((perf.pp_reading - 29.62).abs() < 0.1, "Reading PP mismatch: {}", perf.pp_reading);
    assert!((perf.difficulty.stars - 9.68).abs() < 0.05, "Star rating mismatch: {}", perf.difficulty.stars);
}
