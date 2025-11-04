use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    create_and_print_vector,
    indoc! {r#"
        vec = v(1, 2);
        print(vec);
    "#},
    equals("(1, 2)"),
    empty()
);

eval_and_assert!(
    create_vector_with_floats,
    indoc! {r#"
        vec = v(1.5, 2.7);
        print(vec);
    "#},
    equals("(1.5, 2.7)"),
    empty()
);

eval_and_assert!(
    create_vector_with_mixed_types,
    indoc! {r#"
        vec = v(1, 2.5);
        print(vec);
    "#},
    equals("(1, 2.5)"),
    empty()
);

eval_and_assert!(
    vector_addition,
    indoc! {r#"
        v1 = v(1, 2);
        v2 = v(3, 4);
        v3 = v1 + v2;
        print(v3);
    "#},
    equals("(4, 6)"),
    empty()
);

eval_and_assert!(
    vector_addition_with_floats,
    indoc! {r#"
        v1 = v(1.5, 2.5);
        v2 = v(0.5, 0.5);
        v3 = v1 + v2;
        print(v3);
    "#},
    equals("(2, 3)"),
    empty()
);

eval_and_assert!(
    vector_index_x,
    indoc! {r#"
        vec = v(10, 20);
        print(vec[0]);
    "#},
    equals("10"),
    empty()
);

eval_and_assert!(
    vector_index_y,
    indoc! {r#"
        vec = v(10, 20);
        print(vec[1]);
    "#},
    equals("20"),
    empty()
);

eval_and_assert!(
    vector_index_out_of_bounds,
    indoc! {r#"
        vec = v(10, 20);
        print(vec[2]);
    "#},
    empty(),
    contains("Index 2 out of bounds, length is 2")
);

eval_and_assert!(
    vector_wrong_arg_count,
    indoc! {r#"
        vec = v(1);
    "#},
    empty(),
    contains("Function v expects 2 arguments, but got 1")
);

eval_and_assert!(
    vector_wrong_type,
    indoc! {r#"
        vec = v("hello", 2);
    "#},
    empty(),
    contains("Cannot create vector with x coordinate of type 'str'")
);
