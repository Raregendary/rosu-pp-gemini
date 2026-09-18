use rosu_pp::{Beatmap, Difficulty, Performance};

fn run_map(name: &str, path: &str) {
    let map = Beatmap::from_path(path).unwrap();
    println!("\n### {}", name);
    println!("| Mod | Star Rating | Max Combo | 100% FC PP |");
    println!("| :--- | :--- | :--- | :--- |");

    let mods_list: Vec<(&str, u32, bool)> = vec![
        ("NM (NoMod)", 0, true),
        ("HD", 8, true),
        ("HR", 16, true),
        ("DT", 64, true),
        ("FL", 1024, true),
        ("EZ", 2, true),
        ("HT", 256, true),
        ("CL (Classic)", 0, false),
        ("HDHR", 8 + 16, true),
        ("HDDT", 8 + 64, true),
    ];

    for (mod_name, mod_bits, lazer) in mods_list {
        let diff = Difficulty::new().mods(mod_bits).calculate(&map);
        let stars = diff.stars();
        let max_combo = diff.max_combo();
        let perf = Performance::new(diff)
            .mods(mod_bits)
            .lazer(lazer)
            .calculate();
        let pp = perf.pp();
        println!("| **{}** | {:.2}★ | {} | **{:.2} pp** |", mod_name, stars, max_combo, pp);
    }
}

#[test]
fn test_all_mods() {
    run_map("osu! Standard: LE SSERAFIM - CRAZY [DADADA] (ID: 5525390, 6.24★)", "./resources/5525390.osu");
    run_map("osu! Standard: Kardashev - Cellar of Ghosts [Remnants] (ID: 3700073, 8.73★)", "./resources/3700073.osu");
    run_map("osu!taiko: tezuka x Aoi - Small Cloud Sugar Candy [Bittersweet Remedy] (ID: 5727828, 7.59★)", "./resources/5727828.osu");
    run_map("osu!catch: Tektheist - Nerv [Where am I?] (ID: 4384622, 6.02★)", "./resources/4384622.osu");
    run_map("osu!mania: Laur - Sound Chimera [[4K] Tryambakam // feat. Auros] (ID: 5873946, 5.91★)", "./resources/5873946.osu");
}
