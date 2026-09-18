use rosu_pp::{
    Beatmap,
    osu::OsuPerformance,
};

#[test]
fn test_4894158_scores() {
    let map = Beatmap::from_path("./resources/4894158.osu").expect("load map");

    // 1. PLOXARU (+HDHR, 3481x, 2767/14/0/0)
    // PerformanceCalculator total: 1051.36 (Aim: 601.91, Speed: 146.34, Acc: 192.41, Reading: 96.91)
    let ploxaru = OsuPerformance::new(&map)
        .mods(8 | 16)
        .combo(3481)
        .n300(2767)
        .n100(14)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("PLOXARU (+HDHR): total={:.2}, aim={:.2}, speed={:.2}, acc={:.2}, reading={:.2}, stars={:.2}",
        ploxaru.pp, ploxaru.pp_aim, ploxaru.pp_speed, ploxaru.pp_acc, ploxaru.pp_reading, ploxaru.difficulty.stars);

    // 2. Aerora (+HD, 3482x, 2760/21/0/0)
    // PerformanceCalculator total: 749.87 (Aim: 345.41, Speed: 146.21, Acc: 172.30, Reading: 86.21)
    let aerora = OsuPerformance::new(&map)
        .mods(8)
        .combo(3482)
        .n300(2760)
        .n100(21)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("Aerora (+HD): total={:.2}, aim={:.2}, speed={:.2}, acc={:.2}, reading={:.2}, stars={:.2}",
        aerora.pp, aerora.pp_aim, aerora.pp_speed, aerora.pp_acc, aerora.pp_reading, aerora.difficulty.stars);

    // 3. evill (+HD, 3480x, 2752/28/1/0)
    // PerformanceCalculator total: 740.76 (Aim: 344.79, Speed: 146.02, Acc: 164.26, Reading: 85.74)
    let evill = OsuPerformance::new(&map)
        .mods(8)
        .combo(3480)
        .n300(2752)
        .n100(28)
        .n50(1)
        .misses(0)
        .calculate()
        .unwrap();

    println!("evill (+HD): total={:.2}, aim={:.2}, speed={:.2}, acc={:.2}, reading={:.2}, stars={:.2}",
        evill.pp, evill.pp_aim, evill.pp_speed, evill.pp_acc, evill.pp_reading, evill.difficulty.stars);

    // 4. Optimism (NM, 3481x, 2773/8/0/0)
    // PerformanceCalculator total: 697.18 (Aim: 346.39, Speed: 146.35, Acc: 185.73, Reading: 7.20)
    let optimism = OsuPerformance::new(&map)
        .combo(3481)
        .n300(2773)
        .n100(8)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("Optimism NM diff attrs: stars={}, aim={}, speed={}, reading={}, aim_slider_count={}, speed_note_count={}",
        optimism.difficulty.stars, optimism.difficulty.aim, optimism.difficulty.speed, optimism.difficulty.reading,
        optimism.difficulty.aim_difficult_slider_count, optimism.difficulty.speed_note_count);

    println!("Optimism (NM): total={:.2}, aim={:.2}, speed={:.2}, acc={:.2}, reading={:.2}, stars={:.2}",
        optimism.pp, optimism.pp_aim, optimism.pp_speed, optimism.pp_acc, optimism.pp_reading, optimism.difficulty.stars);

    // 5. REFANTAZIO (NM, 3481x, 2770/11/0/0)
    // PerformanceCalculator total: 693.74 (Aim: 346.17, Speed: 146.32, Acc: 182.54, Reading: 7.19)
    let refantazio = OsuPerformance::new(&map)
        .combo(3481)
        .n300(2770)
        .n100(11)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("REFANTAZIO (NM): total={:.2}, aim={:.2}, speed={:.2}, acc={:.2}, reading={:.2}, stars={:.2}",
        refantazio.pp, refantazio.pp_aim, refantazio.pp_speed, refantazio.pp_acc, refantazio.pp_reading, refantazio.difficulty.stars);

    // 6. konjihi (NM, 3482x, 2758/23/0/0)
    // PerformanceCalculator total: 680.52 (Aim: 345.26, Speed: 146.18, Acc: 170.31, Reading: 7.13)
    let konjihi = OsuPerformance::new(&map)
        .combo(3482)
        .n300(2758)
        .n100(23)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("konjihi (NM): total={:.2}, aim={:.2}, speed={:.2}, acc={:.2}, reading={:.2}, stars={:.2}",
        konjihi.pp, konjihi.pp_aim, konjihi.pp_speed, konjihi.pp_acc, konjihi.pp_reading, konjihi.difficulty.stars);

    // 7. Rizer (+CL, 3482x, 2780/1/0/0)
    // PerformanceCalculator total: 701.09 (Aim: 346.92, Speed: 146.39, Acc: 189.05, Reading: 7.23)
    let rizer = OsuPerformance::new(&map)
        .lazer(false)
        .combo(3482)
        .n300(2780)
        .n100(1)
        .n50(0)
        .misses(0)
        .calculate()
        .unwrap();

    println!("Rizer (+CL): total={:.2}, aim={:.2}, speed={:.2}, acc={:.2}, reading={:.2}, stars={:.2}",
        rizer.pp, rizer.pp_aim, rizer.pp_speed, rizer.pp_acc, rizer.pp_reading, rizer.difficulty.stars);

    // Check with tolerance of 1.0 (due to floating point / rounding):
    assert!((ploxaru.pp - 1051.36).abs() < 1.0, "PLOXARU pp mismatch: expected 1051.36, got {}", ploxaru.pp);
    assert!((aerora.pp - 749.87).abs() < 1.0, "Aerora pp mismatch: expected 749.87, got {}", aerora.pp);
    assert!((evill.pp - 740.76).abs() < 1.0, "evill pp mismatch: expected 740.76, got {}", evill.pp);
    assert!((optimism.pp - 697.18).abs() < 1.0, "Optimism pp mismatch: expected 697.18, got {}", optimism.pp);
    assert!((refantazio.pp - 693.74).abs() < 1.0, "REFANTAZIO pp mismatch: expected 693.74, got {}", refantazio.pp);
    assert!((konjihi.pp - 680.52).abs() < 1.0, "konjihi pp mismatch: expected 680.52, got {}", konjihi.pp);
    let rizer_expected = 701.09;
    assert!(
        (rizer.pp - rizer_expected).abs() < 1.0,
        "Rizer pp mismatch: expected {rizer_expected}, got {}",
        rizer.pp
    );

    let map2 = Beatmap::from_path("./resources/2785319.osu").expect("load map 2785319");
    for (name, mods_val) in [("NM", 0), ("HD", 8), ("HR", 16), ("DT", 64), ("EZ", 2), ("FL", 1024), ("RX", 128), ("AP", 8192), ("SO", 4096)] {
        let p = OsuPerformance::new(&map2).mods(mods_val).calculate().unwrap();
        println!("{name} => {{");
        println!("    aim: {},", p.difficulty.aim);
        println!("    aim_difficult_slider_count: {},", p.difficulty.aim_difficult_slider_count);
        println!("    speed: {},", p.difficulty.speed);
        println!("    reading: {},", p.difficulty.reading);
        println!("    flashlight: {},", p.difficulty.flashlight);
        println!("    slider_factor: {},", p.difficulty.slider_factor);
        println!("    aim_top_weighted_slider_factor: {},", p.difficulty.aim_top_weighted_slider_factor);
        println!("    speed_top_weighted_slider_factor: {},", p.difficulty.speed_top_weighted_slider_factor);
        println!("    speed_note_count: {},", p.difficulty.speed_note_count);
        println!("    aim_difficult_strain_count: {},", p.difficulty.aim_difficult_strain_count);
        println!("    speed_difficult_strain_count: {},", p.difficulty.speed_difficult_strain_count);
        println!("    reading_difficult_note_count: {},", p.difficulty.reading_difficult_note_count);
        println!("    nested_score_per_object: {},", p.difficulty.nested_score_per_object);
        println!("    legacy_score_base_multiplier: {},", p.difficulty.legacy_score_base_multiplier);
        println!("    maximum_legacy_combo_score: {},", p.difficulty.maximum_legacy_combo_score);
        println!("    ar: {},", p.difficulty.ar);
        println!("    great_hit_window: {},", p.difficulty.great_hit_window);
        println!("    ok_hit_window: {},", p.difficulty.ok_hit_window);
        println!("    meh_hit_window: {},", p.difficulty.meh_hit_window);
        println!("    hp: {},", p.difficulty.hp);
        println!("    n_circles: {},", p.difficulty.n_circles);
        println!("    n_sliders: {},", p.difficulty.n_sliders);
        println!("    n_large_ticks: {},", p.difficulty.n_large_ticks);
        println!("    n_spinners: {},", p.difficulty.n_spinners);
        println!("    stars: {},", p.difficulty.stars);
        println!("    max_combo: {},", p.difficulty.max_combo);
        println!("}};");
    }
}
