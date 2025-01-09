use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    regex_basic_can_be_created,
    indoc! {r#"
        reg = /\d+/;
        print(reg);
    "#},
    equals(indoc! {r#"
        /\d+/
    "#}),
    empty()
);

eval_and_assert!(
    regex_long_can_be_created,
    indoc! {r#"
        print(/(\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d\.\d+([+-][0-2]\d:[0-5]\d|Z))|(\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d([+-][0-2]\d:[0-5]\d|Z))|(\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d([+-][0-2]\d:[0-5]\d|Z))/);
    "#},
    equals(indoc! {r#"
        /(\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d\.\d+([+-][0-2]\d:[0-5]\d|Z))|(\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d([+-][0-2]\d:[0-5]\d|Z))|(\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d([+-][0-2]\d:[0-5]\d|Z))/
    "#}),
    empty()
);

eval_and_assert!(
    regex_find_all_integers,
    indoc! {r#"
        print("123 321 423 idk 312,1231.123".find_all(/\d+/));
    "#},
    equals(r#"[[123], [321], [423], [312], [1231], [123]]"#),
    empty()
);

eval_and_assert!(
    regex_find_all_negative_integers,
    indoc! {r#"
        print("123 -321 423 idk -312,1231.123".find_all(/-\d+/));
    "#},
    equals(r#"[[-321], [-312]]"#),
    empty()
);

eval_and_assert!(
    regex_find_all_groups,
    indoc! {r#"
        inp = "1-3 a: abcde\n"
            + "1-3 b: cdefg\n"
            + "2-9 c: ccccccccc\n";

        matches = inp.find_all(/(\d+)-(\d+) (\w): (\w+)/);

        print(matches);
    "#},
    equals(
        r#"[["1-3 a: abcde", 1, 3, "a", "abcde"], ["1-3 b: cdefg", 1, 3, "b", "cdefg"], ["2-9 c: ccccccccc", 2, 9, "c", "ccccccccc"]]"#
    ),
    empty()
);

eval_and_assert!(
    regex_optional_group_is_null_when_not_present,
    indoc! {r#"
        regex = /(\d+)(?:-(\d+))?/;
        print("123".find_all(regex));
        print("123-321".find_all(regex));
    "#},
    equals(indoc! {r#"
        [[123, 123, null]]
        [["123-321", 123, 321]]
    "#}),
    empty()
);
