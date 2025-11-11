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
    op_modulo,
    indoc::indoc! {r#"
        print(10 % 3);
        print(10 % 2);
        print((-10) % 3);
    "#},
    equals(indoc::indoc! {r#"
        1
        0
        -1
    "#}),
    empty()
);

eval_and_assert!(
    negation_with_function_calls,
    indoc::indoc! {r#"
        fn returns_five() {
            return 5;
        };
        fn returns_negative() {
            return -3;
        };

        print(-returns_five());
        print(-returns_negative());
        print(-returns_five() + 10);
        print(-returns_five() * 2);
    "#},
    equals(indoc::indoc! {r#"
        -5
        3
        5
        -10
    "#}),
    empty()
);

eval_and_assert!(
    manhattan_distance_single_tuple,
    indoc::indoc! {r#"
        print(manhattan((3, 4)));
        print(manhattan((-3, -4)));
        print(manhattan((1, 2, 3)));
    "#},
    equals(indoc::indoc! {r#"
        7
        7
        6
    "#}),
    empty()
);

eval_and_assert!(
    manhattan_distance_two_tuples,
    indoc::indoc! {r#"
        print(manhattan((1, 2), (4, 6)));
        print(manhattan((0, 0), (3, 4)));
        print(manhattan((5, 5, 5), (2, 3, 1)));
    "#},
    equals(indoc::indoc! {r#"
        7
        7
        9
    "#}),
    empty()
);

eval_and_assert!(
    manhattan_distance_error_non_tuple,
    "print(manhattan(5))",
    empty(),
    crate::helpers::output::contains("cannot calculate manhattan distance")
);
