//! Predefined spinner animations extracted from the Go `charm-spinner` package.
//!
//! This file is **machine generated** by `scripts/generate_spinners.rs` from
//! `reference/go/charm-spinner/spinner.go`. Do not edit it by hand.
//!
//! Every frame is stored as a `\u{...}` escape, so this source file is pure
//! ASCII and can never be damaged by an encoding round-trip (mojibake). The
//! compiled strings are byte-identical to the Go source; the parity tests in
//! `tests/parity.rs` prove it against the original implementation.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::Spinner;

/// The `Dots` spinner: 10 frames at 80 ms per frame.
///
/// Frames (Unicode code points): 280B / 2819 / 2839 / 2838 / 283C / 2834 / 2826 / 2827 / 2807 / 280F
pub fn dots() -> Spinner {
    Spinner::new(
        vec![
            "\u{280B}".to_owned(),
            "\u{2819}".to_owned(),
            "\u{2839}".to_owned(),
            "\u{2838}".to_owned(),
            "\u{283C}".to_owned(),
            "\u{2834}".to_owned(),
            "\u{2826}".to_owned(),
            "\u{2827}".to_owned(),
            "\u{2807}".to_owned(),
            "\u{280F}".to_owned(),
        ],
        Duration::from_millis(80),
    )
}

/// The `Line` spinner: 4 frames at 130 ms per frame.
///
/// Frames (Unicode code points): 002D / 005C / 007C / 002F
pub fn line() -> Spinner {
    Spinner::new(
        vec![
            "\u{002D}".to_owned(),
            "\u{005C}".to_owned(),
            "\u{007C}".to_owned(),
            "\u{002F}".to_owned(),
        ],
        Duration::from_millis(130),
    )
}

/// The `Star` spinner: 6 frames at 70 ms per frame.
///
/// Frames (Unicode code points): 2736 / 2738 / 2739 / 273A / 2739 / 2737
pub fn star() -> Spinner {
    Spinner::new(
        vec![
            "\u{2736}".to_owned(),
            "\u{2738}".to_owned(),
            "\u{2739}".to_owned(),
            "\u{273A}".to_owned(),
            "\u{2739}".to_owned(),
            "\u{2737}".to_owned(),
        ],
        Duration::from_millis(70),
    )
}

/// The `Moon` spinner: 8 frames at 80 ms per frame.
///
/// Frames (Unicode code points): 1F311 / 1F312 / 1F313 / 1F314 / 1F315 / 1F316 / 1F317 / 1F318
pub fn moon() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F311}".to_owned(),
            "\u{1F312}".to_owned(),
            "\u{1F313}".to_owned(),
            "\u{1F314}".to_owned(),
            "\u{1F315}".to_owned(),
            "\u{1F316}".to_owned(),
            "\u{1F317}".to_owned(),
            "\u{1F318}".to_owned(),
        ],
        Duration::from_millis(80),
    )
}

/// The `Earth` spinner: 3 frames at 180 ms per frame.
///
/// Frames (Unicode code points): 1F30D / 1F30E / 1F30F
pub fn earth() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F30D}".to_owned(),
            "\u{1F30E}".to_owned(),
            "\u{1F30F}".to_owned(),
        ],
        Duration::from_millis(180),
    )
}

/// The `Clock` spinner: 12 frames at 100 ms per frame.
///
/// Frames (Unicode code points): 1F550 / 1F551 / 1F552 / 1F553 / 1F554 / 1F555 / 1F556 / 1F557 / 1F558 / 1F559 / 1F55A / 1F55B
pub fn clock() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F550}".to_owned(),
            "\u{1F551}".to_owned(),
            "\u{1F552}".to_owned(),
            "\u{1F553}".to_owned(),
            "\u{1F554}".to_owned(),
            "\u{1F555}".to_owned(),
            "\u{1F556}".to_owned(),
            "\u{1F557}".to_owned(),
            "\u{1F558}".to_owned(),
            "\u{1F559}".to_owned(),
            "\u{1F55A}".to_owned(),
            "\u{1F55B}".to_owned(),
        ],
        Duration::from_millis(100),
    )
}

/// The `Bounce` spinner: 4 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 2801 / 2802 / 2804 / 2802
pub fn bounce() -> Spinner {
    Spinner::new(
        vec![
            "\u{2801}".to_owned(),
            "\u{2802}".to_owned(),
            "\u{2804}".to_owned(),
            "\u{2802}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Bar` spinner: 15 frames at 80 ms per frame.
///
/// Frames (Unicode code points): 258F / 258E / 258D / 258C / 258B / 258A / 2589 / 2588 / 2589 / 258A / 258B / 258C / 258D / 258E / 258F
pub fn bar() -> Spinner {
    Spinner::new(
        vec![
            "\u{258F}".to_owned(),
            "\u{258E}".to_owned(),
            "\u{258D}".to_owned(),
            "\u{258C}".to_owned(),
            "\u{258B}".to_owned(),
            "\u{258A}".to_owned(),
            "\u{2589}".to_owned(),
            "\u{2588}".to_owned(),
            "\u{2589}".to_owned(),
            "\u{258A}".to_owned(),
            "\u{258B}".to_owned(),
            "\u{258C}".to_owned(),
            "\u{258D}".to_owned(),
            "\u{258E}".to_owned(),
            "\u{258F}".to_owned(),
        ],
        Duration::from_millis(80),
    )
}

/// The `Circle` spinner: 4 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 25D0 / 25D3 / 25D1 / 25D2
pub fn circle() -> Spinner {
    Spinner::new(
        vec![
            "\u{25D0}".to_owned(),
            "\u{25D3}".to_owned(),
            "\u{25D1}".to_owned(),
            "\u{25D2}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Square` spinner: 4 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 2596 / 2598 / 259D / 2597
pub fn square() -> Spinner {
    Spinner::new(
        vec![
            "\u{2596}".to_owned(),
            "\u{2598}".to_owned(),
            "\u{259D}".to_owned(),
            "\u{2597}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Triangle` spinner: 4 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 25E2 / 25E3 / 25E4 / 25E5
pub fn triangle() -> Spinner {
    Spinner::new(
        vec![
            "\u{25E2}".to_owned(),
            "\u{25E3}".to_owned(),
            "\u{25E4}".to_owned(),
            "\u{25E5}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Arrow` spinner: 8 frames at 100 ms per frame.
///
/// Frames (Unicode code points): 2190 / 2196 / 2191 / 2197 / 2192 / 2198 / 2193 / 2199
pub fn arrow() -> Spinner {
    Spinner::new(
        vec![
            "\u{2190}".to_owned(),
            "\u{2196}".to_owned(),
            "\u{2191}".to_owned(),
            "\u{2197}".to_owned(),
            "\u{2192}".to_owned(),
            "\u{2198}".to_owned(),
            "\u{2193}".to_owned(),
            "\u{2199}".to_owned(),
        ],
        Duration::from_millis(100),
    )
}

/// The `Pulse` spinner: 6 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 2588 / 2593 / 2592 / 2591 / 2592 / 2593
pub fn pulse() -> Spinner {
    Spinner::new(
        vec![
            "\u{2588}".to_owned(),
            "\u{2593}".to_owned(),
            "\u{2592}".to_owned(),
            "\u{2591}".to_owned(),
            "\u{2592}".to_owned(),
            "\u{2593}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Flip` spinner: 12 frames at 70 ms per frame.
///
/// Frames (Unicode code points): 005F / 005F / 005F / 002D / 0060 / 0060 / 0027 / 00B4 / 002D / 005F / 005F / 005F
pub fn flip() -> Spinner {
    Spinner::new(
        vec![
            "\u{005F}".to_owned(),
            "\u{005F}".to_owned(),
            "\u{005F}".to_owned(),
            "\u{002D}".to_owned(),
            "\u{0060}".to_owned(),
            "\u{0060}".to_owned(),
            "\u{0027}".to_owned(),
            "\u{00B4}".to_owned(),
            "\u{002D}".to_owned(),
            "\u{005F}".to_owned(),
            "\u{005F}".to_owned(),
            "\u{005F}".to_owned(),
        ],
        Duration::from_millis(70),
    )
}

/// The `Grow` spinner: 15 frames at 80 ms per frame.
///
/// Frames (Unicode code points): 2581 / 2582 / 2583 / 2584 / 2585 / 2586 / 2587 / 2588 / 2587 / 2586 / 2585 / 2584 / 2583 / 2582 / 2581
pub fn grow() -> Spinner {
    Spinner::new(
        vec![
            "\u{2581}".to_owned(),
            "\u{2582}".to_owned(),
            "\u{2583}".to_owned(),
            "\u{2584}".to_owned(),
            "\u{2585}".to_owned(),
            "\u{2586}".to_owned(),
            "\u{2587}".to_owned(),
            "\u{2588}".to_owned(),
            "\u{2587}".to_owned(),
            "\u{2586}".to_owned(),
            "\u{2585}".to_owned(),
            "\u{2584}".to_owned(),
            "\u{2583}".to_owned(),
            "\u{2582}".to_owned(),
            "\u{2581}".to_owned(),
        ],
        Duration::from_millis(80),
    )
}

/// The `Layer` spinner: 4 frames at 150 ms per frame.
///
/// Frames (Unicode code points): 2500 / 2501 / 2550 / 2501
pub fn layer() -> Spinner {
    Spinner::new(
        vec![
            "\u{2500}".to_owned(),
            "\u{2501}".to_owned(),
            "\u{2550}".to_owned(),
            "\u{2501}".to_owned(),
        ],
        Duration::from_millis(150),
    )
}

/// The `Toggle` spinner: 2 frames at 250 ms per frame.
///
/// Frames (Unicode code points): 22B6 / 22B7
pub fn toggle() -> Spinner {
    Spinner::new(
        vec!["\u{22B6}".to_owned(), "\u{22B7}".to_owned()],
        Duration::from_millis(250),
    )
}

/// The `Balloon` spinner: 7 frames at 140 ms per frame.
///
/// Frames (Unicode code points): 002E / 006F / 004F / 00B0 / 004F / 006F / 002E
pub fn balloon() -> Spinner {
    Spinner::new(
        vec![
            "\u{002E}".to_owned(),
            "\u{006F}".to_owned(),
            "\u{004F}".to_owned(),
            "\u{00B0}".to_owned(),
            "\u{004F}".to_owned(),
            "\u{006F}".to_owned(),
            "\u{002E}".to_owned(),
        ],
        Duration::from_millis(140),
    )
}

/// The `Noise` spinner: 4 frames at 100 ms per frame.
///
/// Frames (Unicode code points): 2593 / 2592 / 2591 / 2592
pub fn noise() -> Spinner {
    Spinner::new(
        vec![
            "\u{2593}".to_owned(),
            "\u{2592}".to_owned(),
            "\u{2591}".to_owned(),
            "\u{2592}".to_owned(),
        ],
        Duration::from_millis(100),
    )
}

/// The `Fading` spinner: 8 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 2591 / 2592 / 2593 / 2588 / 2593 / 2592 / 2591 / 0020
pub fn fading() -> Spinner {
    Spinner::new(
        vec![
            "\u{2591}".to_owned(),
            "\u{2592}".to_owned(),
            "\u{2593}".to_owned(),
            "\u{2588}".to_owned(),
            "\u{2593}".to_owned(),
            "\u{2592}".to_owned(),
            "\u{2591}".to_owned(),
            "\u{0020}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Christmas` spinner: 6 frames at 200 ms per frame.
///
/// Frames (Unicode code points): 1F384 / 1F385 / 26C4 / 1F381 / 1F514 / 2B50
pub fn christmas() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F384}".to_owned(),
            "\u{1F385}".to_owned(),
            "\u{26C4}".to_owned(),
            "\u{1F381}".to_owned(),
            "\u{1F514}".to_owned(),
            "\u{2B50}".to_owned(),
        ],
        Duration::from_millis(200),
    )
}

/// The `Monkey` spinner: 3 frames at 210 ms per frame.
///
/// Frames (Unicode code points): 1F648 / 1F649 / 1F64A
pub fn monkey() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F648}".to_owned(),
            "\u{1F649}".to_owned(),
            "\u{1F64A}".to_owned(),
        ],
        Duration::from_millis(210),
    )
}

/// The `Hearts` spinner: 6 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 1F49B / 1F499 / 1F49C / 1F49A / 2764 FE0F / 1F9E1
pub fn hearts() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F49B}".to_owned(),
            "\u{1F499}".to_owned(),
            "\u{1F49C}".to_owned(),
            "\u{1F49A}".to_owned(),
            "\u{2764}\u{FE0F}".to_owned(),
            "\u{1F9E1}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Weather` spinner: 13 frames at 150 ms per frame.
///
/// Frames (Unicode code points): 2600 FE0F / 1F324 FE0F / 26C5 / 1F325 FE0F / 2601 FE0F / 1F327 FE0F / 26C8 FE0F / 1F329 FE0F / 1F327 FE0F / 2601 FE0F / 1F325 FE0F / 26C5 / 1F324 FE0F
pub fn weather() -> Spinner {
    Spinner::new(
        vec![
            "\u{2600}\u{FE0F}".to_owned(),
            "\u{1F324}\u{FE0F}".to_owned(),
            "\u{26C5}".to_owned(),
            "\u{1F325}\u{FE0F}".to_owned(),
            "\u{2601}\u{FE0F}".to_owned(),
            "\u{1F327}\u{FE0F}".to_owned(),
            "\u{26C8}\u{FE0F}".to_owned(),
            "\u{1F329}\u{FE0F}".to_owned(),
            "\u{1F327}\u{FE0F}".to_owned(),
            "\u{2601}\u{FE0F}".to_owned(),
            "\u{1F325}\u{FE0F}".to_owned(),
            "\u{26C5}".to_owned(),
            "\u{1F324}\u{FE0F}".to_owned(),
        ],
        Duration::from_millis(150),
    )
}

/// The `Soccer` spinner: 4 frames at 200 ms per frame.
///
/// Frames (Unicode code points): 26BD / 1F945 / 1F45F / 1F4A8
pub fn soccer() -> Spinner {
    Spinner::new(
        vec![
            "\u{26BD}".to_owned(),
            "\u{1F945}".to_owned(),
            "\u{1F45F}".to_owned(),
            "\u{1F4A8}".to_owned(),
        ],
        Duration::from_millis(200),
    )
}

/// The `Matrix` spinner: 10 frames at 70 ms per frame.
///
/// Frames (Unicode code points): 16A0 / 16A1 / 16A2 / 16A3 / 16A4 / 16A5 / 16A6 / 16A7 / 16A8 / 16A9
pub fn matrix() -> Spinner {
    Spinner::new(
        vec![
            "\u{16A0}".to_owned(),
            "\u{16A1}".to_owned(),
            "\u{16A2}".to_owned(),
            "\u{16A3}".to_owned(),
            "\u{16A4}".to_owned(),
            "\u{16A5}".to_owned(),
            "\u{16A6}".to_owned(),
            "\u{16A7}".to_owned(),
            "\u{16A8}".to_owned(),
            "\u{16A9}".to_owned(),
        ],
        Duration::from_millis(70),
    )
}

/// The `DNA` spinner: 6 frames at 160 ms per frame.
///
/// Frames (Unicode code points): 2554 2550 2557 / 255A 2550 255D / 2554 2550 2557 / 2550 2550 2550 / 255A 2550 255D / 2550 2550 2550
pub fn dna() -> Spinner {
    Spinner::new(
        vec![
            "\u{2554}\u{2550}\u{2557}".to_owned(),
            "\u{255A}\u{2550}\u{255D}".to_owned(),
            "\u{2554}\u{2550}\u{2557}".to_owned(),
            "\u{2550}\u{2550}\u{2550}".to_owned(),
            "\u{255A}\u{2550}\u{255D}".to_owned(),
            "\u{2550}\u{2550}\u{2550}".to_owned(),
        ],
        Duration::from_millis(160),
    )
}

/// The `Orbit` spinner: 4 frames at 100 ms per frame.
///
/// Frames (Unicode code points): 25DC 0020 / 0020 25DD / 0020 25DE / 25DF 0020
pub fn orbit() -> Spinner {
    Spinner::new(
        vec![
            "\u{25DC}\u{0020}".to_owned(),
            "\u{0020}\u{25DD}".to_owned(),
            "\u{0020}\u{25DE}".to_owned(),
            "\u{25DF}\u{0020}".to_owned(),
        ],
        Duration::from_millis(100),
    )
}

/// The `Rocket` spinner: 6 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 1F680 / 1F4A8 / 2728 / 1F31F / 2728 / 1F4A8
pub fn rocket() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F680}".to_owned(),
            "\u{1F4A8}".to_owned(),
            "\u{2728}".to_owned(),
            "\u{1F31F}".to_owned(),
            "\u{2728}".to_owned(),
            "\u{1F4A8}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Fire` spinner: 6 frames at 100 ms per frame.
///
/// Frames (Unicode code points): 1F525 / 1F525 / 1F4A5 / 2728 / 1F525 / 1F4A5
pub fn fire() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F525}".to_owned(),
            "\u{1F525}".to_owned(),
            "\u{1F4A5}".to_owned(),
            "\u{2728}".to_owned(),
            "\u{1F525}".to_owned(),
            "\u{1F4A5}".to_owned(),
        ],
        Duration::from_millis(100),
    )
}

/// The `Ellipsis` spinner: 3 frames at 120 ms per frame.
///
/// Frames (Unicode code points): 002E / 002E 002E / 002E 002E 002E
pub fn ellipsis() -> Spinner {
    Spinner::new(
        vec![
            "\u{002E}".to_owned(),
            "\u{002E}\u{002E}".to_owned(),
            "\u{002E}\u{002E}\u{002E}".to_owned(),
        ],
        Duration::from_millis(120),
    )
}

/// The `Rectangle` spinner: 4 frames at 110 ms per frame.
///
/// Frames (Unicode code points): 259B / 259C / 2599 / 259F
pub fn rectangle() -> Spinner {
    Spinner::new(
        vec![
            "\u{259B}".to_owned(),
            "\u{259C}".to_owned(),
            "\u{2599}".to_owned(),
            "\u{259F}".to_owned(),
        ],
        Duration::from_millis(110),
    )
}

/// The `Diamond` spinner: 2 frames at 180 ms per frame.
///
/// Frames (Unicode code points): 25C6 / 25C7
pub fn diamond() -> Spinner {
    Spinner::new(
        vec!["\u{25C6}".to_owned(), "\u{25C7}".to_owned()],
        Duration::from_millis(180),
    )
}

/// The `Hexagon` spinner: 2 frames at 180 ms per frame.
///
/// Frames (Unicode code points): 2B22 / 2B21
pub fn hexagon() -> Spinner {
    Spinner::new(
        vec!["\u{2B22}".to_owned(), "\u{2B21}".to_owned()],
        Duration::from_millis(180),
    )
}

/// The `BouncingBar` spinner: 7 frames at 100 ms per frame.
///
/// Frames (Unicode code points): 005B 0020 0020 0020 0020 005D / 005B 003D 0020 0020 0020 005D / 005B 003D 003D 0020 0020 005D / 005B 003D 003D 003D 0020 005D / 005B 0020 003D 003D 003D 005D / 005B 0020 0020 003D 003D 005D / 005B 0020 0020 0020 003D 005D
pub fn bouncingbar() -> Spinner {
    Spinner::new(
        vec![
            "\u{005B}\u{0020}\u{0020}\u{0020}\u{0020}\u{005D}".to_owned(),
            "\u{005B}\u{003D}\u{0020}\u{0020}\u{0020}\u{005D}".to_owned(),
            "\u{005B}\u{003D}\u{003D}\u{0020}\u{0020}\u{005D}".to_owned(),
            "\u{005B}\u{003D}\u{003D}\u{003D}\u{0020}\u{005D}".to_owned(),
            "\u{005B}\u{0020}\u{003D}\u{003D}\u{003D}\u{005D}".to_owned(),
            "\u{005B}\u{0020}\u{0020}\u{003D}\u{003D}\u{005D}".to_owned(),
            "\u{005B}\u{0020}\u{0020}\u{0020}\u{003D}\u{005D}".to_owned(),
        ],
        Duration::from_millis(100),
    )
}

/// The `Digits` spinner: 10 frames at 90 ms per frame.
///
/// Frames (Unicode code points): 0030 / 0031 / 0032 / 0033 / 0034 / 0035 / 0036 / 0037 / 0038 / 0039
pub fn digits() -> Spinner {
    Spinner::new(
        vec![
            "\u{0030}".to_owned(),
            "\u{0031}".to_owned(),
            "\u{0032}".to_owned(),
            "\u{0033}".to_owned(),
            "\u{0034}".to_owned(),
            "\u{0035}".to_owned(),
            "\u{0036}".to_owned(),
            "\u{0037}".to_owned(),
            "\u{0038}".to_owned(),
            "\u{0039}".to_owned(),
        ],
        Duration::from_millis(90),
    )
}

/// The `Quotes` spinner: 4 frames at 150 ms per frame.
///
/// Frames (Unicode code points): 0027 / 0022 / 275E / 275F
pub fn quotes() -> Spinner {
    Spinner::new(
        vec![
            "\u{0027}".to_owned(),
            "\u{0022}".to_owned(),
            "\u{275E}".to_owned(),
            "\u{275F}".to_owned(),
        ],
        Duration::from_millis(150),
    )
}

/// The `Cat` spinner: 6 frames at 130 ms per frame.
///
/// Frames (Unicode code points): 1F63A / 1F638 / 1F639 / 1F63B / 1F63C / 1F63D
pub fn cat() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F63A}".to_owned(),
            "\u{1F638}".to_owned(),
            "\u{1F639}".to_owned(),
            "\u{1F63B}".to_owned(),
            "\u{1F63C}".to_owned(),
            "\u{1F63D}".to_owned(),
        ],
        Duration::from_millis(130),
    )
}

/// The `Money` spinner: 4 frames at 200 ms per frame.
///
/// Frames (Unicode code points): 1F4B0 / 1F4B5 / 1F4B4 / 1F4B6
pub fn money() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F4B0}".to_owned(),
            "\u{1F4B5}".to_owned(),
            "\u{1F4B4}".to_owned(),
            "\u{1F4B6}".to_owned(),
        ],
        Duration::from_millis(200),
    )
}

/// The `Beach` spinner: 4 frames at 220 ms per frame.
///
/// Frames (Unicode code points): 1F3D6 FE0F / 1F334 / 1F30A / 2600 FE0F
pub fn beach() -> Spinner {
    Spinner::new(
        vec![
            "\u{1F3D6}\u{FE0F}".to_owned(),
            "\u{1F334}".to_owned(),
            "\u{1F30A}".to_owned(),
            "\u{2600}\u{FE0F}".to_owned(),
        ],
        Duration::from_millis(220),
    )
}

/// The `Music` spinner: 3 frames at 140 ms per frame.
///
/// Frames (Unicode code points): 266A / 266B / 266C
pub fn music() -> Spinner {
    Spinner::new(
        vec![
            "\u{266A}".to_owned(),
            "\u{266B}".to_owned(),
            "\u{266C}".to_owned(),
        ],
        Duration::from_millis(140),
    )
}

/// The `Target` spinner: 4 frames at 130 ms per frame.
///
/// Frames (Unicode code points): 25CE / 25C9 / 2299 / 25C9
pub fn target() -> Spinner {
    Spinner::new(
        vec![
            "\u{25CE}".to_owned(),
            "\u{25C9}".to_owned(),
            "\u{2299}".to_owned(),
            "\u{25C9}".to_owned(),
        ],
        Duration::from_millis(130),
    )
}

/// Returns every predefined spinner, keyed by its display name.
///
/// The keys are exactly the ones used by the Go implementation
/// (`"Dots"`, `"Line"`, `"Star"`, ...), and the iteration order is
/// alphabetical, matching Go's `sort.Strings` ordering.
///
/// # Examples
///
/// ```
/// let spinners = charm_spinner::all();
/// assert_eq!(spinners.len(), 42);
/// assert!(spinners.contains_key("Dots"));
/// ```
pub fn all() -> BTreeMap<&'static str, Spinner> {
    let mut spinners = BTreeMap::new();
    spinners.insert("Dots", dots());
    spinners.insert("Line", line());
    spinners.insert("Star", star());
    spinners.insert("Moon", moon());
    spinners.insert("Earth", earth());
    spinners.insert("Clock", clock());
    spinners.insert("Bounce", bounce());
    spinners.insert("Bar", bar());
    spinners.insert("Circle", circle());
    spinners.insert("Square", square());
    spinners.insert("Triangle", triangle());
    spinners.insert("Arrow", arrow());
    spinners.insert("Pulse", pulse());
    spinners.insert("Flip", flip());
    spinners.insert("Grow", grow());
    spinners.insert("Layer", layer());
    spinners.insert("Toggle", toggle());
    spinners.insert("Balloon", balloon());
    spinners.insert("Noise", noise());
    spinners.insert("Fading", fading());
    spinners.insert("Christmas", christmas());
    spinners.insert("Monkey", monkey());
    spinners.insert("Hearts", hearts());
    spinners.insert("Weather", weather());
    spinners.insert("Soccer", soccer());
    spinners.insert("Matrix", matrix());
    spinners.insert("DNA", dna());
    spinners.insert("Orbit", orbit());
    spinners.insert("Rocket", rocket());
    spinners.insert("Fire", fire());
    spinners.insert("Ellipsis", ellipsis());
    spinners.insert("Rectangle", rectangle());
    spinners.insert("Diamond", diamond());
    spinners.insert("Hexagon", hexagon());
    spinners.insert("BouncingBar", bouncingbar());
    spinners.insert("Digits", digits());
    spinners.insert("Quotes", quotes());
    spinners.insert("Cat", cat());
    spinners.insert("Money", money());
    spinners.insert("Beach", beach());
    spinners.insert("Music", music());
    spinners.insert("Target", target());
    spinners
}
