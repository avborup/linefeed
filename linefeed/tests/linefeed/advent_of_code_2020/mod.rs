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
