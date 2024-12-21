use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    function_oneliners,
    include_str!("../functions.lf"),
    equals(indoc! {r#"
        2
        2
        yes
    "#}),
    empty()
);
