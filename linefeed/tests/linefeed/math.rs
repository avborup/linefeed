use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
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
    contains("cannot calculate manhattan distance")
);

eval_and_assert!(
    mod_inv_basic,
    indoc::indoc! {r#"
        print(mod_inv(3, 7));
        print(mod_inv(5, 7));
        print(mod_inv(2, 7));
    "#},
    equals(indoc::indoc! {r#"
        5
        3
        4
    "#}),
    empty()
);

eval_and_assert!(
    mod_inv_verify,
    indoc::indoc! {r#"
        a = 3;
        m = 7;
        inv = mod_inv(a, m);
        print((a * inv) % m);
    "#},
    equals("1"),
    empty()
);

eval_and_assert!(
    mod_inv_larger_numbers,
    indoc::indoc! {r#"
        print(mod_inv(17, 43));
        print(mod_inv(42, 2017));
    "#},
    equals(indoc::indoc! {r#"
        38
        1969
    "#}),
    empty()
);

eval_and_assert!(
    mod_inv_negative_numbers,
    indoc::indoc! {r#"
        inv = mod_inv(-3, 7);
        print(inv);
        # Verify: ((-3 % 7) * inv) % 7 should be 1
        print((((-3) % 7 + 7) % 7 * inv) % 7);
    "#},
    equals(indoc::indoc! {r#"
        2
        1
    "#}),
    empty()
);

eval_and_assert!(
    mod_inv_no_inverse,
    "print(mod_inv(4, 6))",
    empty(),
    contains("Modular inverse does not exist")
);

eval_and_assert!(
    mod_inv_type_error,
    "print(mod_inv(\"hello\", 7))",
    empty(),
    contains("mod_inv first argument must be a number")
);

eval_and_assert!(
    mod_inv_with_big_integers,
    indoc::indoc! {r#"
        # Test with numbers that would overflow isize
        a = 123456789012347;
        m = 987654321098761;
        inv = mod_inv(a, m);
        print((a * inv) % m);
    "#},
    equals("1"),
    empty()
);
