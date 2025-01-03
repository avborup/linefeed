use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    day01,
    include_str!("day01.lf"),
    equals(indoc! {r#"
        Part 1: 514579
        Part 2: 241861950
    "#}),
    empty()
);
