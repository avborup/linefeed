use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    binary_no_padding,
    indoc::indoc! {r#"
        print(42.binary());
        print(255.binary());
        print(0.binary());
        print(1.binary());
    "#},
    equals(indoc::indoc! {r#"
        101010
        11111111
        0
        1
    "#}),
    empty()
);

eval_and_assert!(
    binary_with_padding_basic,
    indoc::indoc! {r#"
        print(42.binary(8));
        print(42.binary(16));
    "#},
    equals(indoc::indoc! {r#"
        00101010
        0000000000101010
    "#}),
    empty()
);

eval_and_assert!(
    binary_padding_smaller_than_number,
    indoc::indoc! {r#"
        print(42.binary(4));
        print(255.binary(4));
    "#},
    equals(indoc::indoc! {r#"
        101010
        11111111
    "#}),
    empty()
);

eval_and_assert!(
    binary_with_zero_padding,
    indoc::indoc! {r#"
        print(0.binary(8));
        print(1.binary(8));
    "#},
    equals(indoc::indoc! {r#"
        00000000
        00000001
    "#}),
    empty()
);

eval_and_assert!(
    binary_various_padding_sizes,
    indoc::indoc! {r#"
        print(255.binary(8));
        print(255.binary(12));
        print(16.binary(8));
        print(128.binary(8));
    "#},
    equals(indoc::indoc! {r#"
        11111111
        000011111111
        00010000
        10000000
    "#}),
    empty()
);

eval_and_assert!(
    binary_powers_of_two,
    indoc::indoc! {r#"
        print(256.binary(16));
        print(7.binary(3));
        print(7.binary(5));
        print(15.binary(4));
    "#},
    equals(indoc::indoc! {r#"
        0000000100000000
        111
        00111
        1111
    "#}),
    empty()
);

eval_and_assert!(
    binary_zero_width_padding,
    "print(42.binary(0));",
    equals("101010"),
    empty()
);

eval_and_assert!(
    binary_negative_numbers,
    indoc::indoc! {r#"
        print((-1).binary());
        print((-5).binary(8));
    "#},
    equals(indoc::indoc! {r#"
        1111111111111111111111111111111111111111111111111111111111111111
        1111111111111111111111111111111111111111111111111111111111111011
    "#}),
    empty()
);

eval_and_assert!(
    bitwise_and,
    indoc::indoc! {r#"
        print(12 & 10);
        print(15 & 7);
    "#},
    equals(indoc::indoc! {r#"
        8
        7
    "#}),
    empty()
);

eval_and_assert!(
    bitwise_or,
    indoc::indoc! {r#"
        print(12 | 10);
        print(8 | 4);
    "#},
    equals(indoc::indoc! {r#"
        14
        12
    "#}),
    empty()
);

eval_and_assert!(
    bitwise_xor,
    indoc::indoc! {r#"
        print(12 ^ 10);
        print(15 ^ 15);
    "#},
    equals(indoc::indoc! {r#"
        6
        0
    "#}),
    empty()
);

eval_and_assert!(
    bitwise_not,
    indoc::indoc! {r#"
        print(~5);
        print(~(-1));
    "#},
    equals(indoc::indoc! {r#"
        -6
        0
    "#}),
    empty()
);

eval_and_assert!(
    bitwise_left_shift,
    indoc::indoc! {r#"
        print(5 << 2);
        print(1 << 4);
    "#},
    equals(indoc::indoc! {r#"
        20
        16
    "#}),
    empty()
);

eval_and_assert!(
    bitwise_right_shift,
    indoc::indoc! {r#"
        print(20 >> 2);
        print(16 >> 4);
    "#},
    equals(indoc::indoc! {r#"
        5
        1
    "#}),
    empty()
);
