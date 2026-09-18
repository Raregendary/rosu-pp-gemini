use std::panic::{self, UnwindSafe};

use rosu_pp::{
    Beatmap,
    catch::{CatchPerformance, CatchPerformanceAttributes},
    mania::{ManiaPerformance, ManiaPerformanceAttributes},
    osu::{OsuPerformance, OsuPerformanceAttributes},
    taiko::{TaikoPerformance, TaikoPerformanceAttributes},
};

use self::common::*;

mod common;

macro_rules! test_cases {
    ( $mode:ident: $path:ident {
        $( $( $mods:ident )+ => {
            $( $key:ident: $value:expr $( , )? )*
        } ;)*
    } ) => {
        let map = Beatmap::from_path(common::$path).unwrap();

        $(
            let mods = 0 $( + $mods )*;
            let (calc, expected) = test_cases!(@$mode { map, $( $key: $value, )* });
            let actual = calc.mods(mods).calculate().unwrap();
            run(&actual, &expected, mods);
        )*
    };
    ( @Osu {
        $map:ident,
        pp: $pp:expr,
        pp_acc: $pp_acc:expr,
        pp_aim: $pp_aim:expr,
        pp_flashlight: $pp_flashlight:expr,
        pp_reading: $pp_reading:expr,
        pp_speed: $pp_speed:expr,
        effective_miss_count: $effective_miss_count:expr,
        speed_deviation: $speed_deviation:expr,
        combo_based_estimated_miss_count: $combo_based_estimated_miss_count:expr,
        score_based_estimated_miss_count: $score_based_estimated_miss_count:expr,
        aim_estimated_slider_breaks: $aim_estimated_slider_breaks:expr,
        speed_estimated_slider_breaks: $speed_estimated_slider_breaks:expr,
    }) => {
        (
            OsuPerformance::from(&$map).lazer(true),
            OsuPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_aim: $pp_aim,
                pp_flashlight: $pp_flashlight,
                pp_reading: $pp_reading,
                pp_speed: $pp_speed,
                effective_miss_count: $effective_miss_count,
                speed_deviation: $speed_deviation,
                combo_based_estimated_miss_count: $combo_based_estimated_miss_count,
                score_based_estimated_miss_count: $score_based_estimated_miss_count,
                aim_estimated_slider_breaks: $aim_estimated_slider_breaks,
                speed_estimated_slider_breaks: $speed_estimated_slider_breaks,
                ..Default::default()
            },
        )
    };
    ( @Taiko {
        $map: ident,
        pp: $pp:expr,
        pp_acc: $pp_acc:expr,
        pp_difficulty: $pp_difficulty:expr,
        estimated_unstable_rate: $estimated_unstable_rate:expr,
    }) => {
        (
            TaikoPerformance::from(&$map),
            TaikoPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_difficulty: $pp_difficulty,
                estimated_unstable_rate: $estimated_unstable_rate,
                ..Default::default()
            },
        )
    };
    ( @Catch {
        $map:ident,
        pp: $pp:expr,
    }) => {
        (
            CatchPerformance::from(&$map),
            CatchPerformanceAttributes {
                pp: $pp,
                ..Default::default()
            },
        )
    };
    ( @Mania {
        $map:ident,
        pp: $pp:expr,
        pp_difficulty: $pp_difficulty:expr,
    }) => {
        (
            ManiaPerformance::from(&$map),
            ManiaPerformanceAttributes {
                pp: $pp,
                pp_difficulty: $pp_difficulty,
                ..Default::default()
            },
        )
    };
}

#[test]
fn basic_osu() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Osu: OSU {
            NM => {
                pp: 316.6254351541815,
                pp_acc: 98.99847982709288,
                pp_aim: 148.7526082898852,
                pp_flashlight: 0.0,
                pp_reading: 2.280420379604552,
                pp_speed: 61.34653468094172,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.70045116819282),
                combo_based_estimated_miss_count: 0.0,
                score_based_estimated_miss_count: None,
                aim_estimated_slider_breaks: 0.0,
                speed_estimated_slider_breaks: 0.0,
            };
            HD => {
                pp: 350.16237814936767,
                pp_acc: 98.99847982709288,
                pp_aim: 148.7526082898852,
                pp_flashlight: 0.0,
                pp_reading: 41.78713337596433,
                pp_speed: 61.34653468094172,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.70045116819282),
                combo_based_estimated_miss_count: 0.0,
                score_based_estimated_miss_count: None,
                aim_estimated_slider_breaks: 0.0,
                speed_estimated_slider_breaks: 0.0,
            };
            EZ HD => {
                pp: 330.90282434522277,
                pp_acc: 16.05545397996135,
                pp_aim: 88.9844281768963,
                pp_flashlight: 0.0,
                pp_reading: 179.87879953078564,
                pp_speed: 40.6734499653221,
                effective_miss_count: 0.0,
                speed_deviation: Some(23.04067407536297),
                combo_based_estimated_miss_count: 0.0,
                score_based_estimated_miss_count: None,
                aim_estimated_slider_breaks: 0.0,
                speed_estimated_slider_breaks: 0.0,
            };
            HR => {
                pp: 468.2189304543129,
                pp_acc: 161.55575439788055,
                pp_aim: 231.45752213382556,
                pp_flashlight: 0.0,
                pp_reading: 3.0588807427220495,
                pp_speed: 61.86844909389747,
                effective_miss_count: 0.0,
                speed_deviation: Some(8.609766678538842),
                combo_based_estimated_miss_count: 0.0,
                score_based_estimated_miss_count: None,
                aim_estimated_slider_breaks: 0.0,
                speed_estimated_slider_breaks: 0.0,
            };
            DT => {
                pp: 860.8294797019728,
                pp_acc: 183.66566616694254,
                pp_aim: 436.40568318181255,
                pp_flashlight: 0.0,
                pp_reading: 32.65492236884264,
                pp_speed: 198.3528992693602,
                effective_miss_count: 0.0,
                speed_deviation: Some(7.66444640194172),
                combo_based_estimated_miss_count: 0.0,
                score_based_estimated_miss_count: None,
                aim_estimated_slider_breaks: 0.0,
                speed_estimated_slider_breaks: 0.0,
            };
            FL => {
                pp: 444.52132461795543,
                pp_acc: 98.99847982709288,
                pp_aim: 148.7526082898852,
                pp_flashlight: 137.45248886328682,
                pp_reading: 2.280420379604552,
                pp_speed: 61.34653468094172,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.70045116819282),
                combo_based_estimated_miss_count: 0.0,
                score_based_estimated_miss_count: None,
                aim_estimated_slider_breaks: 0.0,
                speed_estimated_slider_breaks: 0.0,
            };
            HD FL => {
                pp: 524.2329425897452,
                pp_acc: 98.99847982709288,
                pp_aim: 148.7526082898852,
                pp_flashlight: 184.18699493543485,
                pp_reading: 41.78713337596433,
                pp_speed: 61.34653468094172,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.70045116819282),
                combo_based_estimated_miss_count: 0.0,
                score_based_estimated_miss_count: None,
                aim_estimated_slider_breaks: 0.0,
                speed_estimated_slider_breaks: 0.0,
            };
        }
    };
}

#[test]
fn basic_taiko() {
    test_cases! {
        Taiko: TAIKO {
            NM => {
                pp: 130.26636361095524,
                pp_acc: 96.78147528038077,
                pp_difficulty: 33.48488833057447,
                estimated_unstable_rate: Some(146.32383579722838),
            };
            HD => {
                pp: 138.1946720235953,
                pp_acc: 104.04008592640933,
                pp_difficulty: 34.15458609718596,
                estimated_unstable_rate: Some(146.32383579722838),
            };
            HR => {
                pp: 166.7024624392237,
                pp_acc: 130.19970760626504,
                pp_difficulty: 36.50275483295865,
                estimated_unstable_rate: Some(120.87621218031911),
            };
            DT => {
                pp: 265.5965861527288,
                pp_acc: 173.1717092863732,
                pp_difficulty: 92.42487686635559,
                estimated_unstable_rate: Some(97.54922386481893),
            };
        }
    };
}

#[test]
fn convert_taiko() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Taiko: OSU {
            NM => {
                pp: 372.44587607195757,
                pp_acc: 219.22939331130098,
                pp_difficulty: 153.2164827606566,
                estimated_unstable_rate: Some(81.74165086164194),
            };
            HD => {
                pp: 373.2119584857609,
                pp_acc: 219.22939331130098,
                pp_difficulty: 153.9825651744599,
                estimated_unstable_rate: Some(81.74165086164194),
            };
            HR => {
                pp: 452.2496503558974,
                pp_acc: 257.6827645468263,
                pp_difficulty: 194.56688580907107,
                estimated_unstable_rate: Some(70.8427640800897),
            };
            DT => {
                pp: 769.1303989592802,
                pp_acc: 383.9741667379376,
                pp_difficulty: 385.1562322213426,
                estimated_unstable_rate: Some(54.494433907761305),
            };
        }
    }
}

#[test]
fn basic_catch() {
    test_cases! {
        Catch: CATCH {
            NM => { pp: 112.72215339177879 };
            HD => { pp: 135.26658407013454 };
            HD HR => { pp: 231.1954012763412 };
            DT => { pp: 245.48176596381523 };
        }
    };
}

#[test]
fn convert_catch() {
    test_cases! {
        Catch: OSU {
            NM => { pp: 232.34402311853054 };
            HD => { pp: 256.159282164472 };
            HD HR => { pp: 327.3523805137957 };
            DT => { pp: 502.8408227990554 };
        }
    };
}

#[test]
fn basic_mania() {
    test_cases! {
        Mania: MANIA {
            NM => { pp: 108.92297471705167, pp_difficulty: 108.92297471705167 };
            EZ => { pp: 54.46148735852584, pp_difficulty: 108.92297471705167 };
            DT => { pp: 224.52717042937203, pp_difficulty: 224.52717042937203 };
        }
    };
}

#[test]
fn convert_mania() {
    test_cases! {
        Mania: OSU {
            NM => { pp: 101.39189449271568, pp_difficulty: 101.39189449271568 };
            EZ => { pp: 50.69594724635784, pp_difficulty: 101.39189449271568 };
            DT => { pp: 198.46891237015896, pp_difficulty: 198.46891237015896 };
        }
    };
}

fn run<A>(actual: &A, expected: &A, mods: u32)
where
    A: AssertEq,
    for<'a> &'a A: UnwindSafe,
{
    if panic::catch_unwind(|| actual.assert_eq(expected)).is_err() {
        panic!("Mods: {mods}");
    }
}

impl AssertEq for OsuPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            difficulty: _,
            pp,
            pp_acc,
            pp_aim,
            pp_flashlight,
            pp_reading,
            pp_speed,
            effective_miss_count,
            speed_deviation,
            combo_based_estimated_miss_count,
            score_based_estimated_miss_count,
            aim_estimated_slider_breaks,
            speed_estimated_slider_breaks,
        } = self;

        assert_eq_float(*pp, expected.pp);
        assert_eq_float(*pp_acc, expected.pp_acc);
        assert_eq_float(*pp_aim, expected.pp_aim);
        assert_eq_float(*pp_flashlight, expected.pp_flashlight);
        assert_eq_float(*pp_reading, expected.pp_reading);
        assert_eq_float(*pp_speed, expected.pp_speed);
        assert_eq_float(*effective_miss_count, expected.effective_miss_count);
        assert_eq_option(*speed_deviation, expected.speed_deviation);
        assert_eq_float(
            *combo_based_estimated_miss_count,
            expected.combo_based_estimated_miss_count,
        );
        assert_eq_option(
            *score_based_estimated_miss_count,
            expected.score_based_estimated_miss_count,
        );
        assert_eq_float(
            *aim_estimated_slider_breaks,
            expected.aim_estimated_slider_breaks,
        );
        assert_eq_float(
            *speed_estimated_slider_breaks,
            expected.speed_estimated_slider_breaks,
        );
    }
}

impl AssertEq for TaikoPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            difficulty: _,
            pp,
            pp_acc,
            pp_difficulty,
            estimated_unstable_rate,
        } = self;

        assert_eq_float(*pp, expected.pp);
        assert_eq_float(*pp_acc, expected.pp_acc);
        assert_eq_float(*pp_difficulty, expected.pp_difficulty);
        assert_eq_option(*estimated_unstable_rate, expected.estimated_unstable_rate);
    }
}

impl AssertEq for CatchPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self { difficulty: _, pp } = self;

        assert_eq_float(*pp, expected.pp);
    }
}

impl AssertEq for ManiaPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            difficulty: _,
            pp,
            pp_difficulty,
        } = self;

        assert_eq_float(*pp_difficulty, expected.pp_difficulty);
        assert_eq_float(*pp, expected.pp);
    }
}
