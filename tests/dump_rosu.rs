use std::fs::File;
use std::io::Write;
use rosu_pp::{Beatmap, Difficulty, Performance};

#[test]
fn test_dump_rosu_values() {
    let maps = [
        ("5525390", "./resources/5525390.osu"),
        ("3700073", "./resources/3700073.osu"),
        ("5727828", "./resources/5727828.osu"),
        ("4384622", "./resources/4384622.osu"),
        ("5873946", "./resources/5873946.osu"),
    ];

    let mods_list: Vec<(&str, u32, bool)> = vec![
        ("NM", 0, true),
        ("HD", 8, true),
        ("HR", 16, true),
        ("DT", 64, true),
        ("FL", 1024, true),
        ("EZ", 2, true),
        ("HT", 256, true),
        ("CL", 0, false),
        ("HDHR", 8 + 16, true),
        ("HDDT", 8 + 64, true),
    ];

    let mut out = String::new();
    out.push_str("{\n");

    for (map_idx, (map_id, path)) in maps.iter().enumerate() {
        let map = Beatmap::from_path(path).unwrap();
        out.push_str(&format!("  \"{}\": {{\n", map_id));

        for (mod_idx, (mod_name, mod_bits, lazer)) in mods_list.iter().enumerate() {
            let diff = Difficulty::new().mods(*mod_bits).calculate(&map);
            let stars = diff.stars();
            let perf = Performance::new(diff)
                .mods(*mod_bits)
                .lazer(*lazer)
                .calculate();
            let pp = perf.pp();

            let comma = if mod_idx + 1 == mods_list.len() { "" } else { "," };
            out.push_str(&format!(
                "    \"{}\": {{ \"stars\": {:.6}, \"pp\": {:.6} }}{}\n",
                mod_name, stars, pp, comma
            ));
        }

        let map_comma = if map_idx + 1 == maps.len() { "" } else { "," };
        out.push_str(&format!("  }}{}\n", map_comma));
    }

    out.push_str("}\n");

    let mut file = File::create("rosu_results.json").unwrap();
    file.write_all(out.as_bytes()).unwrap();
    println!("Wrote rosu_results.json successfully");
}
