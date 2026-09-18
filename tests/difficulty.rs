use std::panic::{self, UnwindSafe};

use rosu_pp::{
    Beatmap, Difficulty,
    catch::{Catch, CatchDifficultyAttributes},
    mania::{Mania, ManiaDifficultyAttributes},
    osu::{Osu, OsuDifficultyAttributes},
    taiko::{Taiko, TaikoDifficultyAttributes},
};

use self::common::*;

mod common;

macro_rules! test_cases {
    ( $mode:ident: $path:ident {
        $( $( $mods:ident )+ => {
            $( $key:ident: $value:literal $( , )? )*
        } $( ; )? )*
    } ) => {
        let map = Beatmap::from_path(common::$path).unwrap();

        $(
            let mods = 0 $( + $mods )*;
            let expected = test_cases!(@$mode { $( $key: $value, )* });

            let actual = Difficulty::new()
                .mods(mods)
                .calculate_for_mode::<$mode>(&map)
                .unwrap();

            run(&actual, &expected, mods);
        )*
    };
    ( @Osu {
        aim: $aim:literal,
        aim_difficult_slider_count: $aim_difficult_slider_count:literal,
        speed: $speed:literal,
        reading: $reading:literal,
        reading_difficult_note_count: $reading_difficult_note_count:literal,
        flashlight: $flashlight:literal,
        slider_factor: $slider_factor:literal,
        aim_top_weighted_slider_factor: $aim_top_weighted_slider_factor:literal,
        speed_top_weighted_slider_factor: $speed_top_weighted_slider_factor:literal,
        speed_note_count: $speed_note_count:literal,
        aim_difficult_strain_count: $aim_difficult_strain_count:literal,
        speed_difficult_strain_count: $speed_difficult_strain_count:literal,
        nested_score_per_object: $nested_score_per_object:literal,
        legacy_score_base_multiplier: $legacy_score_base_multiplier:literal,
        maximum_legacy_combo_score: $maximum_legacy_combo_score:literal,
        ar: $ar:literal,
        great_hit_window: $great_hit_window:literal,
        ok_hit_window: $ok_hit_window:literal,
        meh_hit_window: $meh_hit_window:literal,
        hp: $hp:literal,
        n_circles: $n_circles:literal,
        n_sliders: $n_sliders:literal,
        n_large_ticks: $n_large_ticks:literal,
        n_spinners: $n_spinners:literal,
        stars: $stars:literal,
        max_combo: $max_combo:literal,
    }) => {
        OsuDifficultyAttributes {
            aim: $aim,
            aim_difficult_slider_count: $aim_difficult_slider_count,
            speed: $speed,
            reading: $reading,
            reading_difficult_note_count: $reading_difficult_note_count,
            flashlight: $flashlight,
            slider_factor: $slider_factor,
            aim_top_weighted_slider_factor: $aim_top_weighted_slider_factor,
            speed_top_weighted_slider_factor: $speed_top_weighted_slider_factor,
            speed_note_count: $speed_note_count,
            aim_difficult_strain_count: $aim_difficult_strain_count,
            speed_difficult_strain_count: $speed_difficult_strain_count,
            nested_score_per_object: $nested_score_per_object,
            legacy_score_base_multiplier: $legacy_score_base_multiplier,
            maximum_legacy_combo_score: $maximum_legacy_combo_score,
            ar: $ar,
            great_hit_window: $great_hit_window,
            ok_hit_window: $ok_hit_window,
            meh_hit_window: $meh_hit_window,
            hp: $hp,
            n_circles: $n_circles,
            n_sliders: $n_sliders,
            n_large_ticks: $n_large_ticks,
            n_spinners: $n_spinners,
            stars: $stars,
            max_combo: $max_combo,
        }
    };
    ( @Taiko {
        stamina: $stamina:literal,
        rhythm: $rhythm:literal,
        color: $color:literal,
        reading: $reading:literal,
        great_hit_window: $great_hit_window:literal,
        ok_hit_window: $ok_hit_window:literal,
        mono_stamina_factor: $mono_stamina_factor:literal,
        mechanical_difficulty: $mechanical_difficulty:literal,
        consistency_factor: $consistency_factor:literal,
        stars: $stars:literal,
        max_combo: $max_combo:literal,
        is_convert: $is_convert:literal,
    }) => {
        TaikoDifficultyAttributes {
            stamina: $stamina,
            rhythm: $rhythm,
            color: $color,
            reading: $reading,
            great_hit_window: $great_hit_window,
            ok_hit_window: $ok_hit_window,
            mono_stamina_factor: $mono_stamina_factor,
            mechanical_difficulty: $mechanical_difficulty,
            consistency_factor: $consistency_factor,
            stars: $stars,
            max_combo: $max_combo,
            is_convert: $is_convert,
        }
    };
    ( @Catch {
        stars: $stars:literal,
        preempt: $preempt:literal,
        n_fruits: $n_fruits:literal,
        n_droplets: $n_droplets:literal,
        n_tiny_droplets: $n_tiny_droplets:literal,
        is_convert: $is_convert:literal,
    }) => {
        CatchDifficultyAttributes {
            stars: $stars,
            preempt: $preempt,
            n_fruits: $n_fruits,
            n_droplets: $n_droplets,
            n_tiny_droplets: $n_tiny_droplets,
            is_convert: $is_convert,
        }
    };
    ( @Mania {
        stars: $stars:literal,
        n_objects: $n_objects:literal,
        n_hold_notes: $n_hold_notes:literal,
        max_combo: $max_combo:literal,
        is_convert: $is_convert:literal,
    }) => {
        ManiaDifficultyAttributes {
            stars: $stars,
            n_objects: $n_objects,
            n_hold_notes: $n_hold_notes,
            max_combo: $max_combo,
            is_convert: $is_convert,
        }
    }
}

#[test]
fn basic_osu() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Osu: OSU {
            NM => {
                aim: 3.278637247177605,
                aim_difficult_slider_count: 192.52700309204852,
                speed: 2.4917265153109014,
                reading: 0.8291853888320672,
                reading_difficult_note_count: 35.039687374657035,
                flashlight: 0.0,
                slider_factor: 0.9630387364631106,
                aim_top_weighted_slider_factor: 1.5242018497163852,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                aim_difficult_strain_count: 124.69578526285657,
                speed_difficult_strain_count: 81.74921671931915,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.004366039467877,
                max_combo: 909,
            };
            HD => {
                aim: 3.278637247177605,
                aim_difficult_slider_count: 192.52700309204852,
                speed: 2.4917265153109014,
                reading: 2.1860538839546106,
                reading_difficult_note_count: 135.65520344400448,
                flashlight: 0.0,
                slider_factor: 0.9630387364631106,
                aim_top_weighted_slider_factor: 1.5242018497163852,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                aim_difficult_strain_count: 124.69578526285657,
                speed_difficult_strain_count: 81.74921671931915,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.309845567239826,
                max_combo: 909,
            };
            HR => {
                aim: 3.79923016723658,
                aim_difficult_slider_count: 191.8309436417523,
                speed: 2.4917265153109014,
                reading: 0.9144659053428965,
                reading_difficult_note_count: 38.45400233005823,
                flashlight: 0.0,
                slider_factor: 0.9475984697095592,
                aim_top_weighted_slider_factor: 1.5100784855077343,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                aim_difficult_strain_count: 119.62905278228544,
                speed_difficult_strain_count: 81.74921671931915,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 10.0,
                great_hit_window: 19.5,
                ok_hit_window: 59.5,
                meh_hit_window: 99.5,
                hp: 7.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.713670477914205,
                max_combo: 909,
            };
            DT => {
                aim: 4.693555645360275,
                aim_difficult_slider_count: 207.9741594999943,
                speed: 3.674242476685812,
                reading: 2.01355217783768,
                reading_difficult_note_count: 190.7406956089773,
                flashlight: 0.0,
                slider_factor: 0.9674909299164448,
                aim_top_weighted_slider_factor: 1.4768880971198521,
                speed_top_weighted_slider_factor: 0.6387657108874913,
                speed_note_count: 211.4547877995673,
                aim_difficult_strain_count: 144.311220047471,
                speed_difficult_strain_count: 86.6096904172161,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 10.533333460489908,
                great_hit_window: 17.666666666666668,
                ok_hit_window: 45.666666666666664,
                meh_hit_window: 73.66666666666667,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 8.761310471544146,
                max_combo: 909,
            };
            FL => {
                aim: 3.278637247177605,
                aim_difficult_slider_count: 192.52700309204852,
                speed: 2.4917265153109014,
                reading: 0.8291853888320672,
                reading_difficult_note_count: 35.039687374657035,
                flashlight: 2.3448026685696757,
                slider_factor: 0.9630387364631106,
                aim_top_weighted_slider_factor: 1.5242018497163852,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                aim_difficult_strain_count: 124.69578526285657,
                speed_difficult_strain_count: 81.74921671931915,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 7.035836648591728,
                max_combo: 909,
            };
            HD EZ => {
                aim: 2.7625480665542375,
                aim_difficult_slider_count: 196.89037772576094,
                speed: 2.39953606797714,
                reading: 3.556094797209706,
                reading_difficult_note_count: 124.69190736811521,
                flashlight: 0.0,
                slider_factor: 0.9796908442807478,
                aim_top_weighted_slider_factor: 1.5627391895895972,
                speed_top_weighted_slider_factor: 0.5369556941433561,
                speed_note_count: 192.26494513961634,
                aim_difficult_strain_count: 129.38152438285374,
                speed_difficult_strain_count: 84.40439013801313,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 3.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 4.650000095367432,
                great_hit_window: 52.5,
                ok_hit_window: 103.5,
                meh_hit_window: 154.5,
                hp: 2.5,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.894294750828443,
                max_combo: 909,
            };
            HD FL => {
                aim: 3.278637247177605,
                aim_difficult_slider_count: 192.52700309204852,
                speed: 2.4917265153109014,
                reading: 2.1860538839546106,
                reading_difficult_note_count: 135.65520344400448,
                flashlight: 2.714310188135725,
                slider_factor: 0.9630387364631106,
                aim_top_weighted_slider_factor: 1.5242018497163852,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                aim_difficult_strain_count: 124.69578526285657,
                speed_difficult_strain_count: 81.74921671931915,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 7.546544719956669,
                max_combo: 909,
            };
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

#[test]
fn basic_taiko() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Taiko: TAIKO {
            NM => {
                stamina: 2.0922161217174127,
                rhythm: 0.15668593055086977,
                color: 0.6655024155624444,
                reading: 1.70963310068196e-5,
                great_hit_window: 34.5,
                ok_hit_window: 79.5,
                mono_stamina_factor: 2.585220903145618e-7,
                mechanical_difficulty: 2.757718537279857,
                consistency_factor: 0.6314855249538754,
                stars: 2.9144215641617333,
                max_combo: 289,
                is_convert: false,
            };
            HR => {
                stamina: 1.7849266630109486,
                rhythm: 0.134624894028623,
                color: 0.5677582700493313,
                reading: 0.5084212133728931,
                great_hit_window: 28.5,
                ok_hit_window: 67.5,
                mono_stamina_factor: 2.585220903145618e-7,
                mechanical_difficulty: 2.3526849330602797,
                consistency_factor: 0.6322607099238484,
                stars: 2.9957310404617963,
                max_combo: 289,
                is_convert: false,
            };
            DT => {
                stamina: 2.568005759306185,
                rhythm: 0.570672991636439,
                color: 0.7218921897845048,
                reading: 0.19075845923759416,
                great_hit_window: 23.0,
                ok_hit_window: 53.0,
                mono_stamina_factor: 2.465693827167051e-7,
                mechanical_difficulty: 3.2898979490906894,
                consistency_factor: 0.6202990548778025,
                stars: 4.051329399964723,
                max_combo: 289,
                is_convert: false,
            };
        }
    };
}

#[test]
fn convert_taiko() {
    test_cases! {
        Taiko: OSU {
            NM => {
                stamina: 2.2228533914222286,
                rhythm: 0.6028550725402863,
                color: 0.8456679526084883,
                reading: 1.0811962040551348,
                great_hit_window: 22.5,
                ok_hit_window: 56.5,
                mono_stamina_factor: 0.0014311041774359666,
                mechanical_difficulty: 3.068521344030717,
                consistency_factor: 0.6875624057993176,
                stars: 4.752572620626138,
                max_combo: 908,
                is_convert: true,
            };
            HR => {
                stamina: 2.3106156684479586,
                rhythm: 0.640061492397074,
                color: 0.8790564547089896,
                reading: 1.4462258521404312,
                great_hit_window: 19.5,
                ok_hit_window: 49.5,
                mono_stamina_factor: 0.0014311041774359666,
                mechanical_difficulty: 3.1896721231569485,
                consistency_factor: 0.679047690894666,
                stars: 5.275959467694453,
                max_combo: 908,
                is_convert: true,
            };
            DT => {
                stamina: 3.252902032191341,
                rhythm: 0.9584853559461861,
                color: 1.0979910141819806,
                reading: 1.8067255325802154,
                great_hit_window: 15.0,
                ok_hit_window: 37.666666666666664,
                mono_stamina_factor: 0.0014418086037955797,
                mechanical_difficulty: 4.350893046373321,
                consistency_factor: 0.6748291725050156,
                stars: 7.116103934899722,
                max_combo: 908,
                is_convert: true,
            };
        }
    };
}

#[test]
fn basic_catch() {
    test_cases! {
        Catch: CATCH {
            NM => {
                stars: 3.2340182503279706,
                preempt: 750.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            HR => {
                stars: 4.308291009137178,
                preempt: 450.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            EZ => {
                stars: 4.059198145823293,
                preempt: 1320.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            DT => {
                stars: 4.6192881825873275,
                preempt: 500.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
        }
    };
}

#[test]
fn convert_catch() {
    test_cases! {
        Catch: OSU {
            NM => {
                stars: 4.526991300645072,
                preempt: 554.9999713897705,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            HR => {
                stars: 5.0738627744810545,
                preempt: 450.0,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            EZ => {
                stars: 3.590187752268528,
                preempt: 1241.9999885559082,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            DT => {
                stars: 6.151552522578919,
                preempt: 369.9999809265137,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
        }
    };
}

#[test]
fn basic_mania() {
    test_cases! {
        Mania: MANIA {
            NM => {
                stars: 3.358304846842773,
                n_objects: 594,
                n_hold_notes: 121,
                max_combo: 956,
                is_convert: false,
            };
            DT => {
                stars: 4.6072892053157295,
                n_objects: 594,
                n_hold_notes: 121,
                max_combo: 956,
                is_convert: false,
            };
        }
    };
}

#[test]
fn convert_mania() {
    test_cases! {
        Mania: OSU {
            NM => {
                stars: 3.2033142085672255,
                n_objects: 1046,
                n_hold_notes: 266,
                max_combo: 1381,
                is_convert: true,
            };
            DT => {
                stars: 4.2934063021960185,
                n_objects: 1046,
                n_hold_notes: 266,
                max_combo: 1381,
                is_convert: true,
            };
        }
    };
}



impl AssertEq for OsuDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            aim,
            aim_difficult_slider_count,
            speed,
            reading,
            reading_difficult_note_count,
            flashlight,
            slider_factor,
            aim_top_weighted_slider_factor,
            speed_top_weighted_slider_factor,
            speed_note_count,
            aim_difficult_strain_count,
            speed_difficult_strain_count,
            nested_score_per_object,
            legacy_score_base_multiplier,
            maximum_legacy_combo_score,
            ar,
            great_hit_window,
            ok_hit_window,
            meh_hit_window,
            hp,
            n_circles,
            n_sliders,
            n_large_ticks,
            n_spinners,
            stars,
            max_combo,
        } = self;

        assert_eq_float(*aim, expected.aim);
        assert_eq_float(
            *aim_difficult_slider_count,
            expected.aim_difficult_slider_count,
        );
        assert_eq_float(*speed, expected.speed);
        assert_eq_float(*reading, expected.reading);
        assert_eq_float(
            *reading_difficult_note_count,
            expected.reading_difficult_note_count,
        );
        assert_eq_float(*flashlight, expected.flashlight);
        assert_eq_float(*slider_factor, expected.slider_factor);
        assert_eq_float(
            *aim_top_weighted_slider_factor,
            expected.aim_top_weighted_slider_factor,
        );
        assert_eq_float(
            *speed_top_weighted_slider_factor,
            expected.speed_top_weighted_slider_factor,
        );
        assert_eq_float(*speed_note_count, expected.speed_note_count);
        assert_eq_float(
            *aim_difficult_strain_count,
            expected.aim_difficult_strain_count,
        );
        assert_eq_float(
            *speed_difficult_strain_count,
            expected.speed_difficult_strain_count,
        );
        assert_eq_float(*nested_score_per_object, expected.nested_score_per_object);
        assert_eq_float(
            *legacy_score_base_multiplier,
            expected.legacy_score_base_multiplier,
        );
        assert_eq_float(
            *maximum_legacy_combo_score,
            expected.maximum_legacy_combo_score,
        );
        assert_eq_float(*ar, expected.ar);
        assert_eq_float(*great_hit_window, expected.great_hit_window);
        assert_eq_float(*ok_hit_window, expected.ok_hit_window);
        assert_eq_float(*meh_hit_window, expected.meh_hit_window);
        assert_eq_float(*hp, expected.hp);
        assert_eq!(*n_circles, expected.n_circles);
        assert_eq!(*n_sliders, expected.n_sliders);
        assert_eq!(*n_large_ticks, expected.n_large_ticks);
        assert_eq!(*n_spinners, expected.n_spinners);
        assert_eq_float(*stars, expected.stars);
        assert_eq!(*max_combo, expected.max_combo);
    }
}

impl AssertEq for TaikoDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            stamina,
            rhythm,
            color,
            reading,
            great_hit_window,
            ok_hit_window,
            mono_stamina_factor,
            mechanical_difficulty,
            consistency_factor,
            stars,
            max_combo,
            is_convert,
        } = self;

        assert_eq_float(*stamina, expected.stamina);
        assert_eq_float(*rhythm, expected.rhythm);
        assert_eq_float(*color, expected.color);
        assert_eq_float(*reading, expected.reading);
        assert_eq_float(*great_hit_window, expected.great_hit_window);
        assert_eq_float(*ok_hit_window, expected.ok_hit_window);
        assert_eq_float(*mono_stamina_factor, expected.mono_stamina_factor);
        assert_eq_float(*mechanical_difficulty, expected.mechanical_difficulty);
        assert_eq_float(*consistency_factor, expected.consistency_factor);
        assert_eq_float(*stars, expected.stars);
        assert_eq!(*max_combo, expected.max_combo);
        assert_eq!(*is_convert, expected.is_convert);
    }
}

impl AssertEq for CatchDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            stars,
            preempt,
            n_fruits,
            n_droplets,
            n_tiny_droplets,
            is_convert,
        } = self;

        assert_eq_float(*stars, expected.stars);
        assert_eq_float(*preempt, expected.preempt);
        assert_eq!(*n_fruits, expected.n_fruits);
        assert_eq!(*n_droplets, expected.n_droplets);
        assert_eq!(*n_tiny_droplets, expected.n_tiny_droplets);
        assert_eq!(*is_convert, expected.is_convert);
    }
}

impl AssertEq for ManiaDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            stars,
            n_objects,
            n_hold_notes,
            max_combo,
            is_convert,
        } = self;

        assert_eq_float(*stars, expected.stars);
        assert_eq!(*n_objects, expected.n_objects);
        assert_eq!(*n_hold_notes, expected.n_hold_notes);
        assert_eq!(*max_combo, expected.max_combo);
        assert_eq!(*is_convert, expected.is_convert);
    }
}
