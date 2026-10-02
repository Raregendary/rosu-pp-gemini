use rosu_pp::{
    Beatmap, Difficulty, any::Performance, catch::CatchPerformance, mania::ManiaPerformance,
    osu::OsuPerformance, taiko::TaikoPerformance,
};

mod common;

use self::common::*;

fn osu_pp(mods: u32, ignore_nf: bool, misses: u32) -> f64 {
    let map = Beatmap::from_path(OSU).unwrap();
    let difficulty = Difficulty::new().mods(mods);

    let max_combo = difficulty.calculate(&map).max_combo();

    OsuPerformance::new(&map)
        .difficulty(difficulty)
        .mods(mods)
        .misses(misses)
        .combo(max_combo.saturating_sub(misses))
        .ignore_nf(ignore_nf)
        .calculate()
        .unwrap()
        .pp
}

fn catch_pp(mods: u32, ignore_nf: bool, misses: u32) -> f64 {
    let map = Beatmap::from_path(CATCH).unwrap();
    let difficulty = Difficulty::new().mods(mods);

    let max_combo = difficulty.calculate(&map).max_combo();

    CatchPerformance::new(&map)
        .difficulty(difficulty)
        .mods(mods)
        .misses(misses)
        .combo(max_combo.saturating_sub(misses))
        .ignore_nf(ignore_nf)
        .calculate()
        .unwrap()
        .pp
}

fn mania_pp(mods: u32, ignore_nf: bool, misses: u32) -> f64 {
    let map = Beatmap::from_path(MANIA).unwrap();

    ManiaPerformance::new(&map)
        .mods(mods)
        .misses(misses)
        .ignore_nf(ignore_nf)
        .calculate()
        .unwrap()
        .pp
}

#[test]
fn default_equals_old_behavior() {
    for misses in [0, 5] {
        let map = Beatmap::from_path(OSU).unwrap();
        let plain = OsuPerformance::new(&map)
            .mods(NF)
            .misses(misses)
            .calculate()
            .unwrap()
            .pp;
        assert_eq_float(osu_pp(NF, false, misses), plain);

        let map = Beatmap::from_path(CATCH).unwrap();
        let plain = CatchPerformance::new(&map)
            .mods(NF)
            .misses(misses)
            .calculate()
            .unwrap()
            .pp;
        assert_eq_float(catch_pp(NF, false, misses), plain);

        let map = Beatmap::from_path(MANIA).unwrap();
        let plain = ManiaPerformance::new(&map)
            .mods(NF)
            .misses(misses)
            .calculate()
            .unwrap()
            .pp;
        assert_eq_float(mania_pp(NF, false, misses), plain);
    }
}

#[test]
fn ignore_nf_matches_no_nf_mods() {
    // The NF penalty only kicks in with misses, so use missy scores.
    for misses in [1, 5] {
        assert_eq_float(osu_pp(NF, true, misses), osu_pp(NM, false, misses));
        assert_eq_float(catch_pp(NF, true, misses), catch_pp(NM, false, misses));
        assert_eq_float(mania_pp(NF, true, misses), mania_pp(NM, false, misses));
    }

    // NF without misses is already penalty-free.
    assert_eq_float(osu_pp(NF, true, 0), osu_pp(NM, false, 0));
    assert_eq_float(catch_pp(NF, true, 0), catch_pp(NM, false, 0));
    assert_eq_float(mania_pp(NF, true, 0), mania_pp(NM, false, 0));
}

#[test]
fn ignore_nf_without_nf_is_noop() {
    for misses in [0, 5] {
        assert_eq_float(osu_pp(NM, true, misses), osu_pp(NM, false, misses));
        assert_eq_float(catch_pp(NM, true, misses), catch_pp(NM, false, misses));
        assert_eq_float(mania_pp(NM, true, misses), mania_pp(NM, false, misses));

        assert_eq_float(osu_pp(HD, true, misses), osu_pp(HD, false, misses));
        assert_eq_float(catch_pp(HD, true, misses), catch_pp(HD, false, misses));
        assert_eq_float(mania_pp(HD, true, misses), mania_pp(HD, false, misses));
    }
}

#[test]
fn penalty_applies_and_toggle_restores() {
    for misses in [1, 5] {
        let penalized = osu_pp(NF, false, misses);
        let unpenalized = osu_pp(NF, true, misses);
        assert!(penalized < unpenalized, "osu misses={misses}");

        let penalized = catch_pp(NF, false, misses);
        let unpenalized = catch_pp(NF, true, misses);
        assert!(penalized < unpenalized, "catch misses={misses}");

        let penalized = mania_pp(NF, false, misses);
        let unpenalized = mania_pp(NF, true, misses);
        assert!(penalized < unpenalized, "mania misses={misses}");
    }

    // Toggling back to `false` restores the penalty.
    let toggled = OsuPerformance::new(Beatmap::from_path(OSU).unwrap())
        .mods(NF)
        .misses(5)
        .ignore_nf(true)
        .ignore_nf(false)
        .calculate()
        .unwrap()
        .pp;
    assert_eq_float(toggled, osu_pp(NF, false, 5));
}

#[test]
fn generic_performance_forwards_flag() {
    let map = Beatmap::from_path(OSU).unwrap();
    let penalized = Performance::new(&map).mods(NF).misses(5).calculate().pp();
    let unpenalized = Performance::new(&map)
        .mods(NF)
        .misses(5)
        .ignore_nf(true)
        .calculate()
        .pp();
    let no_nf = Performance::new(&map).misses(5).calculate().pp();
    assert!(penalized < unpenalized);
    assert_eq_float(unpenalized, no_nf);

    let map = Beatmap::from_path(CATCH).unwrap();
    let penalized = Performance::new(&map).mods(NF).misses(5).calculate().pp();
    let unpenalized = Performance::new(&map)
        .mods(NF)
        .misses(5)
        .ignore_nf(true)
        .calculate()
        .pp();
    let no_nf = Performance::new(&map).misses(5).calculate().pp();
    assert!(penalized < unpenalized);
    assert_eq_float(unpenalized, no_nf);

    let map = Beatmap::from_path(MANIA).unwrap();
    let penalized = Performance::new(&map).mods(NF).misses(5).calculate().pp();
    let unpenalized = Performance::new(&map)
        .mods(NF)
        .misses(5)
        .ignore_nf(true)
        .calculate()
        .pp();
    let no_nf = Performance::new(&map).misses(5).calculate().pp();
    assert!(penalized < unpenalized);
    assert_eq_float(unpenalized, no_nf);
}

#[test]
fn taiko_flag_is_noop() {
    let map = Beatmap::from_path(TAIKO).unwrap();

    let plain = TaikoPerformance::new(&map)
        .mods(NF)
        .misses(5)
        .calculate()
        .unwrap()
        .pp;
    let ignored = TaikoPerformance::new(&map)
        .mods(NF)
        .misses(5)
        .ignore_nf(true)
        .calculate()
        .unwrap()
        .pp;
    assert_eq_float(plain, ignored);
}
