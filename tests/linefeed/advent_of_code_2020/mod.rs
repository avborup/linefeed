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
