use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    tuple_order_comparison_works,
    indoc! {r#"
        foo = (1, 2, 3);
        bar = (1, 2, 4);
        print(foo < bar);
        print(foo > bar);
        print(foo < foo);
        print(foo <= foo);
    "#},
    equals(indoc! {r#"
        true
        false
        false
        true
    "#}),
    empty()
);

eval_and_assert!(
    tuple_element_wise_addition_basic,
    indoc! {r#"
        a = (1, 2, 3);
        b = (4, 5, 6);
        print(a + b);
    "#},
    equals("(5, 7, 9)\n"),
    empty()
);

eval_and_assert!(
    tuple_element_wise_addition_floats,
    indoc! {r#"
        a = (1.5, 2.5);
        b = (3.0, 4.0);
        print(a + b);
    "#},
    equals("(4.5, 6.5)\n"),
    empty()
);

eval_and_assert!(
    tuple_element_wise_addition_strings,
    indoc! {r#"
        a = ("hello", "world");
        b = (" ", "!");
        print(a + b);
    "#},
    equals("(\"hello \", \"world!\")\n"),
    empty()
);

eval_and_assert!(
    tuple_element_wise_addition_nested,
    indoc! {r#"
        a = ((1, 2), (3, 4));
        b = ((5, 6), (7, 8));
        print(a + b);
    "#},
    equals("((6, 8), (10, 12))\n"),
    empty()
);

eval_and_assert!(
    tuple_mismatched_length_error,
    indoc! {r#"
        a = (1, 2);
        b = (3, 4, 5);
        print(a + b);
    "#},
    empty(),
    contains("Cannot add tuples of different lengths")
);

eval_and_assert!(
    tuple_element_wise_subtraction_basic,
    indoc! {r#"
        a = (10, 20, 30);
        b = (4, 5, 6);
        print(a - b);
    "#},
    equals("(6, 15, 24)\n"),
    empty()
);

eval_and_assert!(
    tuple_element_wise_subtraction_floats,
    indoc! {r#"
        a = (5.5, 10.0);
        b = (2.5, 3.0);
        print(a - b);
    "#},
    equals("(3, 7)\n"),
    empty()
);

eval_and_assert!(
    tuple_element_wise_subtraction_nested,
    indoc! {r#"
        a = ((10, 20), (30, 40));
        b = ((5, 6), (7, 8));
        print(a - b);
    "#},
    equals("((5, 14), (23, 32))\n"),
    empty()
);

eval_and_assert!(
    tuple_subtraction_mismatched_length_error,
    indoc! {r#"
        a = (10, 20);
        b = (3, 4, 5);
        print(a - b);
    "#},
    empty(),
    contains("Cannot subtract tuples of different lengths")
);

eval_and_assert!(
    tuple_scalar_multiplication_basic,
    indoc! {r#"
        a = (1, 2, 3);
        print(a * 5);
    "#},
    equals("(5, 10, 15)\n"),
    empty()
);

eval_and_assert!(
    tuple_scalar_multiplication_commutative,
    indoc! {r#"
        a = (2, 4, 6);
        print(3 * a);
    "#},
    equals("(6, 12, 18)\n"),
    empty()
);

eval_and_assert!(
    tuple_scalar_multiplication_float,
    indoc! {r#"
        a = (1.0, 2.0, 3.0);
        print(a * 2.5);
    "#},
    equals("(2.5, 5, 7.5)\n"),
    empty()
);

eval_and_assert!(
    tuple_scalar_multiplication_nested,
    indoc! {r#"
        a = ((1, 2), (3, 4));
        print(a * 10);
    "#},
    equals("((10, 20), (30, 40))\n"),
    empty()
);
