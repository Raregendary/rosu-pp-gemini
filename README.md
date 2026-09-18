[![crates.io](https://img.shields.io/crates/v/rosu-pp.svg)](https://crates.io/crates/rosu-pp) [![docs](https://docs.rs/rosu-pp/badge.svg)](https://docs.rs/rosu-pp)

# rosu-pp

> [!NOTE]
> **Fork Notice**: This is a temporary fork that is **not planned to be updated for future versions**. The upstream `rosu-pp` maintainers are already actively working on and nearing completion of their own official PR/update for the latest difficulty & PP algorithms. This fork is intended strictly as a one-time solution while the upstream project completes its release.

## Performance Benchmarks & Comparison vs. osu-tools

Benchmark conducted between **`osu-tools`** (.NET 8.0 Release) and **`rosu-pp`** (Rust Release) on official osu! beatmaps above 5–6★ across all four game modes (warmup iterations followed by 50 timed iterations for Decoding & Difficulty, and 500 timed iterations for Performance):

### 1. Decoding (.osu Beatmap Parser)

| Gamemode / Beatmap | `osu-tools` (Median / Mean) | `rosu-pp` (Median / Mean) | Speedup |
| :--- | :--- | :--- | :--- |
| **osu! standard (5525390, 6.24★)** | 15.57 ms / 15.80 ms | **0.82 ms / 0.85 ms** | **~19.0x faster** |
| **osu! standard (3700073, 8.73★)** | 6.35 ms / 8.06 ms | **0.96 ms / 0.96 ms** | **~6.6x faster** |
| **osu!taiko (5727828, 7.59★)** | 4.20 ms / 4.60 ms | **0.59 ms / 0.62 ms** | **~7.1x faster** |
| **osu!catch (4384622, 6.02★)** | 3.59 ms / 4.41 ms | **0.77 ms / 0.81 ms** | **~4.7x faster** |
| **osu!mania (5873946, 5.91★)** | 5.23 ms / 5.73 ms | **1.04 ms / 1.06 ms** | **~5.0x faster** |

### 2. Difficulty Calculation (Star Rating & Skills)

| Gamemode / Beatmap | `osu-tools` (Median / Mean) | `rosu-pp` (Median / Mean) | Speedup |
| :--- | :--- | :--- | :--- |
| **osu! standard (5525390, 6.24★)** | 19.56 ms / 21.55 ms | **2.47 ms / 2.50 ms** | **~7.9x faster** |
| **osu! standard (3700073, 8.73★)** | 15.75 ms / 17.54 ms | **9.77 ms / 9.71 ms** | **~1.6x faster** |
| **osu!taiko (5727828, 7.59★)** | 6.47 ms / 7.39 ms | **1.65 ms / 1.72 ms** | **~3.9x faster** |
| **osu!catch (4384622, 6.02★)** | 6.20 ms / 8.49 ms | **0.42 ms / 0.42 ms** | **~14.8x faster** |
| **osu!mania (5873946, 5.91★)** | 5.34 ms / 6.63 ms | **3.22 ms / 3.21 ms** | **~1.7x faster** |

### 3. Performance (PP) Calculation

| Gamemode / Beatmap | `osu-tools` (Median / Mean) | `rosu-pp` (Median / Mean) | Speedup |
| :--- | :--- | :--- | :--- |
| **osu! standard (5525390, 6.24★)** | 2.00 µs / 2.17 µs | **0.40 µs / 0.38 µs** | **~5.0x faster** |
| **osu! standard (3700073, 8.73★)** | 1.50 µs / 2.01 µs | **0.40 µs / 0.44 µs** | **~3.8x faster** |
| **osu!taiko (5727828, 7.59★)** | 0.30 µs / 0.35 µs | **0.30 µs / 0.32 µs** | **~1.0x (parity)** |
| **osu!catch (4384622, 6.02★)** | 0.50 µs / 0.54 µs | **0.10 µs / 0.11 µs** | **~5.0x faster** |
| **osu!mania (5873946, 5.91★)** | 0.20 µs / 0.19 µs | **0.10 µs / 0.10 µs** | **~2.0x faster** |

---

## Beatmap Calculations Across Game Mods: Comparison vs. osu-tools

Direct comparison between **`osu-tools`** and **`rosu-pp`** for Star Rating and 100% FC PP across different game mods (`NM`, `HD`, `HR`, `DT`, `FL`, `EZ`, `HT`, `CL`, `HDHR`, `HDDT`), rounded to 4 decimal places:

### osu! Standard: [LE SSERAFIM - CRAZY [DADADA]](https://osu.ppy.sh/b/5525390) (Beatmap ID: 5525390)

| Mod | Star Rating (`osu-tools`) | Star Rating (`rosu-pp`) | 100% FC PP (`osu-tools`) | 100% FC PP (`rosu-pp`) |
| :--- | :--- | :--- | :--- | :--- |
| **NM** | 6.2424★ | 6.2424★ | 367.2054 pp | 367.2054 pp |
| **HD** | 6.5029★ | 6.5029★ | 397.7292 pp | 397.7292 pp |
| **HR** | 6.8243★ | 6.8243★ | 505.5194 pp | 505.5194 pp |
| **DT** | 9.3939★ | 9.3939★ | 1067.5463 pp | 1067.5464 pp |
| **FL** | 8.0231★ | 8.0231★ | 632.7455 pp | 632.7455 pp |
| **EZ** | 6.5908★ | 6.5908★ | 306.9029 pp | 306.9029 pp |
| **HT** | 4.9180★ | 4.9180★ | 186.8024 pp | 186.8024 pp |
| **CL** | 6.2424★ | 6.2424★ | 336.4563 pp | 336.4563 pp |
| **HDHR** | 6.9992★ | 6.9992★ | 529.4846 pp | 529.4846 pp |
| **HDDT** | 9.5303★ | 9.5303★ | 1103.2874 pp | 1103.2875 pp |

### osu! Standard (Heavy Map): [Kardashev - Cellar of Ghosts [Remnants]](https://osu.ppy.sh/b/3700073) (Beatmap ID: 3700073)

| Mod | Star Rating (`osu-tools`) | Star Rating (`rosu-pp`) | 100% FC PP (`osu-tools`) | 100% FC PP (`rosu-pp`) |
| :--- | :--- | :--- | :--- | :--- |
| **NM** | 8.7309★ | 8.7309★ | 992.9525 pp | 992.9525 pp |
| **HD** | 9.1045★ | 9.1045★ | 1078.7972 pp | 1078.7972 pp |
| **HR** | 9.6607★ | 9.6607★ | 1334.5754 pp | 1334.5754 pp |
| **DT** | 14.3030★ | 14.3030★ | 3593.0509 pp | 3593.0522 pp |
| **FL** | 11.8383★ | 11.8383★ | 1961.7580 pp | 1961.7580 pp |
| **EZ** | 8.3965★ | 8.3965★ | 654.3604 pp | 654.3604 pp |
| **HT** | 6.6786★ | 6.6786★ | 479.8592 pp | 479.8589 pp |
| **CL** | 8.7309★ | 8.7309★ | 990.9652 pp | 990.9652 pp |
| **HDHR** | 10.0177★ | 10.0177★ | 1434.4691 pp | 1434.4691 pp |
| **HDDT** | 14.4864★ | 14.4864★ | 3704.7785 pp | 3704.7797 pp |

### osu!taiko: [tezuka x Aoi feat. Momohina Nano - Small Cloud Sugar Candy [Bittersweet Remedy]](https://osu.ppy.sh/b/5727828) (Beatmap ID: 5727828)

| Mod | Star Rating (`osu-tools`) | Star Rating (`rosu-pp`) | 100% FC PP (`osu-tools`) | 100% FC PP (`rosu-pp`) |
| :--- | :--- | :--- | :--- | :--- |
| **NM** | 7.5937★ | 7.5937★ | 652.8474 pp | 652.8474 pp |
| **HD** | 7.5937★ | 7.5937★ | 677.0432 pp | 677.0432 pp |
| **HR** | 8.2433★ | 8.2433★ | 871.7036 pp | 871.7036 pp |
| **DT** | 10.5231★ | 10.5231★ | 1444.5382 pp | 1444.5382 pp |
| **FL** | 7.5937★ | 7.5937★ | 675.3635 pp | 675.3635 pp |
| **EZ** | 7.5563★ | 7.5563★ | 567.5302 pp | 567.5302 pp |
| **HT** | 6.0596★ | 6.0596★ | 397.7714 pp | 397.7714 pp |
| **CL** | 7.5937★ | 7.5937★ | 652.8474 pp | 652.8474 pp |
| **HDHR** | 8.2433★ | 8.2433★ | 907.2212 pp | 907.2212 pp |
| **HDDT** | 10.5231★ | 10.5231★ | 1498.2048 pp | 1498.2048 pp |

### osu!catch: [Tektheist - Nerv [Where am I?]](https://osu.ppy.sh/b/4384622) (Beatmap ID: 4384622)

| Mod | Star Rating (`osu-tools`) | Star Rating (`rosu-pp`) | 100% FC PP (`osu-tools`) | 100% FC PP (`rosu-pp`) |
| :--- | :--- | :--- | :--- | :--- |
| **NM** | 6.0173★ | 6.0173★ | 509.3719 pp | 509.3719 pp |
| **HD** | 6.0173★ | 6.0173★ | 553.9420 pp | 553.9420 pp |
| **HR** | 6.6712★ | 6.6712★ | 655.9872 pp | 655.9872 pp |
| **DT** | 8.6802★ | 8.6802★ | 1245.5471 pp | 1245.5471 pp |
| **FL** | 6.0173★ | 6.0173★ | 885.9948 pp | 885.9948 pp |
| **EZ** | 6.5622★ | 6.5622★ | 623.9098 pp | 623.9098 pp |
| **HT** | 4.6702★ | 4.6702★ | 292.1166 pp | 292.1166 pp |
| **CL** | 6.0173★ | 6.0173★ | 509.3719 pp | 509.3719 pp |
| **HDHR** | 6.6712★ | 6.6712★ | 688.7865 pp | 688.7865 pp |
| **HDDT** | 8.6802★ | 8.6802★ | 1274.6098 pp | 1274.6098 pp |

### osu!mania: [Laur - Sound Chimera [[4K] Tryambakam // feat. Auros]](https://osu.ppy.sh/b/5873946) (Beatmap ID: 5873946)

| Mod | Star Rating (`osu-tools`) | Star Rating (`rosu-pp`) | 100% FC PP (`osu-tools`) | 100% FC PP (`rosu-pp`) |
| :--- | :--- | :--- | :--- | :--- |
| **NM** | 5.9052★ | 5.9052★ | 413.6337 pp | 413.6337 pp |
| **HD** | 5.9052★ | 5.9052★ | 413.6337 pp | 413.6337 pp |
| **HR** | 5.9052★ | 5.9052★ | 413.6337 pp | 413.6337 pp |
| **DT** | 8.1950★ | 8.1950★ | 864.2538 pp | 864.2538 pp |
| **FL** | 5.9052★ | 5.9052★ | 413.6337 pp | 413.6337 pp |
| **EZ** | 5.9052★ | 5.9052★ | 206.8168 pp | 206.8168 pp |
| **HT** | 4.7116★ | 4.7116★ | 248.0510 pp | 248.0510 pp |
| **CL** | 5.9052★ | 5.9052★ | 413.6337 pp | 413.6337 pp |
| **HDHR** | 5.9052★ | 5.9052★ | 413.6337 pp | 413.6337 pp |
| **HDDT** | 8.1950★ | 8.1950★ | 864.2538 pp | 864.2538 pp |

---

<!-- cargo-rdme start -->

Library to calculate difficulty and performance attributes for all [osu!] gamemodes.

A large part of `rosu-pp` is a port of [osu!lazer]'s difficulty and performance calculation
with emphasis on a precise translation to Rust for the most [accurate results](#accuracy)
while also providing a significant [boost in performance](#speed).

Last commits of the ported code:
  - [osu!lazer] : `28c846b4d9366484792e27f4729cd1afa2cdeb66` (2025-10-13)
  - [osu!tools] : `ab97b64f60901952926b2121ddffb8976d7f8775` (2025-10-16)

News posts of the latest updates: <https://osu.ppy.sh/home/news/2025-10-29-performance-points-star-rating-updates>

### Usage

```rust
// Decode the map
let map = rosu_pp::Beatmap::from_path("./resources/2785319.osu").unwrap();

// Whereas osu! simply times out on malicious maps, rosu-pp does not. To
// prevent potential performance/memory issues, it is recommended to check
// beforehand whether a map is too suspicious for further calculation.
// Alternatively, using e.g. `checked_calculate` instead of `calculate`
// performs the same suspicion check before calculating.
if let Err(sus) = map.check_suspicion() {
    panic!("{sus:?}");
}

// Calculate difficulty attributes
let diff_attrs = rosu_pp::Difficulty::new()
    .mods(8 + 16) // HDHR
    .calculate(&map); // or `checked_calculate`

let stars = diff_attrs.stars();

// Calculate performance attributes
let perf_attrs = rosu_pp::Performance::new(diff_attrs)
    // To speed up the calculation, we used the previous attributes.
    // **Note** that this should only be done if the map and all difficulty
    // settings stay the same, otherwise the final attributes will be incorrect!
    .mods(24) // HDHR, must be the same as before
    .combo(789)
    .accuracy(99.2)
    .misses(2)
    .calculate();

let pp = perf_attrs.pp();

// Again, we re-use the previous attributes for maximum efficiency.
let max_pp = perf_attrs.performance()
    .mods(24) // Still the same
    .calculate()
    .pp();

println!("Stars: {stars} | PP: {pp}/{max_pp}");
```

### Gradual calculation

Gradually calculating attributes provides an efficient way to process each hitobject
separately and calculate the attributes only up to that point.

For difficulty attributes, there is `GradualDifficulty` which implements `Iterator`
and for performance attributes there is `GradualPerformance` which requires the current
score state.

```rust
use rosu_pp::{Beatmap, GradualPerformance, Difficulty, any::ScoreState};

let map = Beatmap::from_path("./resources/1028484.osu").unwrap();

let mut gradual = Difficulty::new()
    .mods(16 + 64) // HRDT
    .clock_rate(1.2)
    .gradual_performance(&map);

let mut state = ScoreState::new(); // empty state, everything is on 0.

// The first 10 hitresults are 300s
for _ in 0..10 {
    state.n300 += 1;
    state.max_combo += 1;
    let attrs = gradual.next(state.clone()).unwrap();
    println!("PP: {}", attrs.pp());
}

// Fast-forward to the end
state.max_combo = ...
state.n300 = ...
state.n_katu = ...
...
let attrs = gradual.last(state).unwrap();
println!("PP: {}", attrs.pp());
```

### Accuracy

`rosu-pp` was tested against millions of real scores and delivered
values that matched osu!lazer perfectly down to the last decimal place.

However, there is one small caveat: the values are only this precise on debug mode.
On release mode, Rust's compiler performs optimizations that produce the tiniest discrepancies
due to floating point inaccuracies. With this in mind, `rosu-pp` is still as accurate as can
be without targeting the .NET compiler itself.
Realistically, the inaccuracies in release mode are negligibly small.

### Speed

An important factor for `rosu-pp` is the calculation speed. Optimizations and an accurate translation
unfortunately don't always go hand-in-hand. Nonetheless, performance improvements are still
snuck in wherever possible, providing a significantly faster runtime than the native C# code.

Results of a rudimentary [benchmark] of osu!lazer and rosu-pp:
```txt
osu!lazer:
Decoding maps:            Median: 325.18ms | Mean: 325.50ms
Calculating difficulties: Median: 568.63ms | Mean: 575.97ms
Calculating performances: Median: 256.00µs | Mean: 240.40µs

rosu-pp:
Decoding maps:            Median: 46.03ms | Mean: 47.13ms
Calculating difficulties: Median: 82.11ms | Mean: 84.27ms
Calculating performances: Median: 40.57µs | Mean: 43.41µs
```

### Features

| Flag          | Description         | Dependencies
| ------------- | ------------------- | ------------
| `default`     | No features enabled |
| `sync`        | Some gradual calculation types can only be shared across threads if this feature is enabled. This feature adds a small performance penalty. |
| `tracing`     | Any error encountered during beatmap decoding will be logged through `tracing::error`. If this feature is **not** enabled, errors will be ignored. | [`tracing`]

### Bindings

Using `rosu-pp` from other languages than Rust:
- JavaScript: [rosu-pp-js]
- Python: [rosu-pp-py]

[osu!]: https://osu.ppy.sh/home
[osu!lazer]: https://github.com/ppy/osu
[osu!tools]: https://github.com/ppy/osu-tools
[`tracing`]: https://docs.rs/tracing
[rosu-pp-js]: https://github.com/MaxOhn/rosu-pp-js
[rosu-pp-py]: https://github.com/MaxOhn/rosu-pp-py
[benchmark]: https://gist.github.com/MaxOhn/625af10011f6d7e13a171b08ccf959ff

<!-- cargo-rdme end -->
