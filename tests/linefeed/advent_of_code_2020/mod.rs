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
        Part 2: ...
    "#}),
    empty()
);
