use rosu_pp::{Beatmap, Difficulty, any::Strains};

#[test]
fn test_osu_strains_lengths_and_timeline() {
    let maps = [
        ("./resources/5525390.osu", "5525390"),
        ("./resources/3700073.osu", "3700073"),
    ];

    let mod_combinations = [
        ("NM", 0),
        ("HD", 8),
        ("HR", 16),
        ("DT", 64),
        ("FL", 1024),
        ("HDDT", 8 + 64),
    ];

    for (map_path, map_id) in maps {
        let map = Beatmap::from_path(map_path).expect("load osu map");

        for (mod_name, mods) in mod_combinations {
            let strains = Difficulty::new().mods(mods).strains(&map);
            let osu_strains = match strains {
                Strains::Osu(s) => s,
                _ => panic!("expected osu strains"),
            };

            let n_aim = osu_strains.aim.len();
            let n_aim_no_sliders = osu_strains.aim_no_sliders.len();
            let n_speed = osu_strains.speed.len();
            let n_flashlight = osu_strains.flashlight.len();

            // 1. All 4 skill strain vectors must have identical length
            assert_eq!(
                n_aim, n_aim_no_sliders,
                "aim and aim_no_sliders length mismatch on map {map_id} with {mod_name}"
            );
            assert_eq!(
                n_aim, n_speed,
                "aim and speed length mismatch on map {map_id} with {mod_name}"
            );
            assert_eq!(
                n_aim, n_flashlight,
                "aim and flashlight length mismatch on map {map_id} with {mod_name}"
            );

            // 2. Length must be non-zero and span the map
            assert!(n_aim > 10, "strain count too small: {n_aim}");

            // 3. Strains should not be sorted in descending order (which was the bug)
            // Verify that strains vary and peak later in the map, not at index 0.
            let max_aim_idx = osu_strains
                .aim
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .unwrap()
                .0;
            assert!(
                max_aim_idx > 0,
                "aim maximum should not be at index 0 for map {map_id} with {mod_name}"
            );

            let max_speed_idx = osu_strains
                .speed
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .unwrap()
                .0;
            assert!(
                max_speed_idx > 0,
                "speed maximum should not be at index 0 for map {map_id} with {mod_name}"
            );

            println!(
                "Map {map_id} ({mod_name}): {n_aim} sections. Max aim at section {max_aim_idx}, max speed at section {max_speed_idx}"
            );
        }
    }
}
