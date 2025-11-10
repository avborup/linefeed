use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    rot_90_degrees_clockwise,
    indoc! {r#"
        print((1, 0).rot(1));
    "#},
    equals("(0, -1)"),
    empty()
);

eval_and_assert!(
    rot_180_degrees,
    indoc! {r#"
        print((1, 0).rot(2));
    "#},
    equals("(-1, 0)"),
    empty()
);

eval_and_assert!(
    rot_270_degrees,
    indoc! {r#"
        print((1, 0).rot(3));
    "#},
    equals("(0, 1)"),
    empty()
);

eval_and_assert!(
    rot_360_degrees_full_circle,
    indoc! {r#"
        print((1, 0).rot(4));
    "#},
    equals("(1, 0)"),
    empty()
);

eval_and_assert!(
    rot_negative_rotation,
    indoc! {r#"
        print((1, 0).rot(-1));
    "#},
    equals("(0, 1)"),
    empty()
);

eval_and_assert!(
    rot_zero_rotation,
    indoc! {r#"
        print((1, 0).rot(0));
    "#},
    equals("(1, 0)"),
    empty()
);

eval_and_assert!(
    rot_with_floats,
    indoc! {r#"
        print((3.0, 4.0).rot(1));
    "#},
    equals("(4, -3)"),
    empty()
);

eval_and_assert!(
    rot_complex_example,
    indoc! {r#"
        vec = (5, 10);
        print(vec.rot(1));
        print(vec.rot(2));
    "#},
    equals(indoc! {r#"
        (10, -5)
        (-5, -10)
    "#}),
    empty()
);

eval_and_assert!(
    rot_error_non_2d_tuple,
    indoc! {r#"
        print((1, 2, 3).rot(1));
    "#},
    empty(),
    contains("rotation only works on 2D numeric tuples, but got tuple with 3 elements")
);

// Note: Single element tuple syntax (1,) is not supported by the parser,
// so we test with a dynamically created 1-element tuple instead
eval_and_assert!(
    rot_error_single_element_tuple,
    indoc! {r#"
        items = [1];
        single = tuple(items);
        print(single.rot(1));
    "#},
    empty(),
    contains("rotation only works on 2D numeric tuples, but got tuple with 1 element")
);

eval_and_assert!(
    rot_error_non_numeric_elements,
    indoc! {r#"
        print(("a", "b").rot(1));
    "#},
    empty(),
    contains("rotation requires tuple elements to be numbers")
);

eval_and_assert!(
    rot_error_mixed_numeric_and_string,
    indoc! {r#"
        print((1, "b").rot(1));
    "#},
    empty(),
    contains("rotation requires tuple elements to be numbers")
);

eval_and_assert!(
    rot_error_wrong_type,
    indoc! {r#"
        print([1, 2].rot(1));
    "#},
    empty(),
    contains("Cannot call method 'rot' on type 'list'")
);

eval_and_assert!(
    rot_error_non_numeric_argument,
    indoc! {r#"
        print((1, 2).rot("hello"));
    "#},
    empty(),
    contains("rotation requires a numeric argument")
);
