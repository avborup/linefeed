use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    big_ints_via_pow,
    indoc! {r#"
        foo = 2 ** 100;
        print(foo);
    "#},
    equals("1267650600228229401496703205376"),
    empty()
);

eval_and_assert!(
    big_ints_negative,
    indoc! {r#"
        foo = -(2 ** 100);
        print(foo);
    "#},
    equals("-1267650600228229401496703205376"),
    empty()
);

eval_and_assert!(
    big_ints_via_many_mults,
    indoc! {r#"
        result = 1;
        for _ in 0..100 {
            result *= 2;
            result += 3 ** 10;
            result *= -1;
        };
        print(result);
    "#},
    equals("24952434414892467539061105894601501"),
    empty()
);
