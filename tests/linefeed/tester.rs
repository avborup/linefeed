use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    factorial,
    include_str!("../factorial.lf"),
    equals("3628800"),
    empty()
);

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

eval_and_assert!(op_addition, "print(1 + 2)", equals("3"), empty());
eval_and_assert!(op_subtraction, "print(1 - 2)", equals("-1"), empty());
eval_and_assert!(op_negation, "print(-1 + 3)", equals("2"), empty());
eval_and_assert!(
    op_multiplication,
    "print(3 * (1 + 3))",
    equals("12"),
    empty()
);
eval_and_assert!(op_division, "print((2 * 10) / 5)", equals("4"), empty());

eval_and_assert!(
    op_equals,
    "print(1 == 1);print(-1 == 1)",
    equals("true \n false"),
    empty()
);
eval_and_assert!(
    op_not_equals,
    "print(1 != 1);print(-1 != 1)",
    equals("false \n true"),
    empty()
);
eval_and_assert!(
    op_not,
    "print(!true);print(!false)",
    equals("false \n true"),
    empty()
);
