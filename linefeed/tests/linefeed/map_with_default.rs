use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    map_with_default_works,
    indoc! {r#"
        m = defaultmap(42);
        m["b"] = 100;
        print(m["a"], m["b"]);
    "#},
    equals("42 100"),
    empty()
);
