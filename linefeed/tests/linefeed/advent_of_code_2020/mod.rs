use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    day01,
    include_str!("day01.lf"),
    include_str!("inputs/day01.txt"),
    equals(indoc! {r#"
        Part 1: 514579
        Part 2: 241861950
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day01_secret,
    include_str!("day01.lf"),
    include_str!("inputs/day01-secret.txt"),
    equals(indoc! {r#"
        Part 1: 1018944
        Part 2: 8446464
    "#}),
    empty()
);

eval_and_assert!(
    day02,
    include_str!("day02.lf"),
    include_str!("inputs/day02.txt"),
    equals(indoc! {r#"
        Part 1: 2
        Part 2: 1
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day02_secret,
    include_str!("day02.lf"),
    include_str!("inputs/day02-secret.txt"),
    equals(indoc! {r#"
        Part 1: 620
        Part 2: 727
    "#}),
    empty()
);

eval_and_assert!(
    day03,
    include_str!("day03.lf"),
    include_str!("inputs/day03.txt"),
    equals(indoc! {r#"
        Part 1: 7
        Part 2: 336
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day03_secret,
    include_str!("day03.lf"),
    include_str!("inputs/day03-secret.txt"),
    equals(indoc! {r#"
        Part 1: 244
        Part 2: 9406609920
    "#}),
    empty()
);

eval_and_assert!(
    day04,
    include_str!("day04.lf"),
    include_str!("inputs/day04.txt"),
    equals(indoc! {r#"
        Part 1: 2
        Part 2: 2
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day04_secret,
    include_str!("day04.lf"),
    include_str!("inputs/day04-secret.txt"),
    equals(indoc! {r#"
        Part 1: 182
        Part 2: 109
    "#}),
    empty()
);

eval_and_assert!(
    day05,
    include_str!("day05.lf"),
    include_str!("inputs/day05.txt"),
    equals(indoc! {r#"
        Part 1: 357
        Part 2: ?
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day05_secret,
    include_str!("day05.lf"),
    include_str!("inputs/day05-secret.txt"),
    equals(indoc! {r#"
        Part 1: 880
        Part 2: 731
    "#}),
    empty()
);

eval_and_assert!(
    day06,
    include_str!("day06.lf"),
    include_str!("inputs/day06.txt"),
    equals(indoc! {r#"
        Part 1: 11
        Part 2: 6
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day06_secret,
    include_str!("day06.lf"),
    include_str!("inputs/day06-secret.txt"),
    equals(indoc! {r#"
        Part 1: 6742
        Part 2: 3447
    "#}),
    empty()
);

eval_and_assert!(
    day07,
    include_str!("day07.lf"),
    include_str!("inputs/day07.txt"),
    equals(indoc! {r#"
        Part 1: 4
        Part 2: 32
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day07_secret,
    include_str!("day07.lf"),
    include_str!("inputs/day07-secret.txt"),
    equals(indoc! {r#"
        Part 1: 115
        Part 2: 1250
    "#}),
    empty()
);

eval_and_assert!(
    day08,
    include_str!("day08.lf"),
    include_str!("inputs/day08.txt"),
    equals(indoc! {r#"
        Part 1: 5
        Part 2: 8
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day08_secret,
    include_str!("day08.lf"),
    include_str!("inputs/day08-secret.txt"),
    equals(indoc! {r#"
        Part 1: 2034
        Part 2: 672
    "#}),
    empty()
);

eval_and_assert!(
    day09,
    include_str!("day09.lf"),
    include_str!("inputs/day09.txt"),
    equals(indoc! {r#"
        Part 1: 127
        Part 2: 62
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day09_secret,
    include_str!("day09.lf"),
    include_str!("inputs/day09-secret.txt"),
    equals(indoc! {r#"
        Part 1: 1930745883
        Part 2: 268878261
    "#}),
    empty()
);

eval_and_assert!(
    day10,
    include_str!("day10.lf"),
    include_str!("inputs/day10.txt"),
    equals(indoc! {r#"
        Part 1: 35
        Part 2: 8
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day10_secret,
    include_str!("day10.lf"),
    include_str!("inputs/day10-secret.txt"),
    equals(indoc! {r#"
        Part 1: 2112
        Part 2: 3022415986688
    "#}),
    empty()
);

eval_and_assert!(
    day11,
    include_str!("day11.lf"),
    include_str!("inputs/day11.txt"),
    equals(indoc! {r#"
        Part 1: 37
        Part 2: 26
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day11_secret,
    include_str!("day11.lf"),
    include_str!("inputs/day11-secret.txt"),
    equals(indoc! {r#"
        Part 1: 2494
        Part 2: 2306
    "#}),
    empty()
);

eval_and_assert!(
    day12,
    include_str!("day12.lf"),
    include_str!("inputs/day12.txt"),
    equals(indoc! {r#"
        Part 1: 25
        Part 2: 286
    "#}),
    empty()
);

#[cfg(feature = "aoc-secret")]
eval_and_assert!(
    day12_secret,
    include_str!("day12.lf"),
    include_str!("inputs/day12-secret.txt"),
    equals(indoc! {r#"
        Part 1: 582
        Part 2: 52069
    "#}),
    empty()
);
