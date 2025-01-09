use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    string_upper_and_lower,
    indoc! {r#"
        my_str = "Hi Mom!";
        print(my_str.upper());
        print(my_str.lower());
        print(my_str);
    "#},
    equals(indoc! {r#"
        HI MOM!
        hi mom!
        Hi Mom!
    "#}),
    empty()
);

eval_and_assert!(
    chained_string_upper_and_lower,
    indoc! {r#"
        print("Hi Mom!".upper().lower());
    "#},
    equals("hi mom!"),
    empty()
);

eval_and_assert!(
    split_works_for_newline,
    indoc! {r#"
        res = "line 1\nline 2\nline 3\n".split("\n");
        print(res);
    "#},
    equals(r#"["line 1", "line 2", "line 3", ""]"#),
    empty()
);

eval_and_assert!(
    split_works_for_empty_delimiter,
    indoc! {r#"
        print("abcde".split(""));
    "#},
    equals(r#"["", "a", "b", "c", "d", "e", ""]"#),
    empty()
);

eval_and_assert!(
    string_lines_works,
    indoc! {r#"
        # Trailing newline
        res1 = "line 1\nline 2\nline 3\n".lines();
        print(res1);

        # Same as non-trailing newline
        res2 = "line 1\nline 2\nline 3".lines();
        print(res2);
    "#},
    equals(indoc! {r#"
        ["line 1", "line 2", "line 3"]
        ["line 1", "line 2", "line 3"]
    "#}),
    empty()
);

eval_and_assert!(
    string_index_works,
    indoc! {r#"
        foo = "hello world";

        print(foo[0]);
        print(foo[1]);
        print(foo[10]);

        print(foo[-1]);
        print(foo[-11]);
    "#},
    equals(indoc! {r#"
        h
        e
        d
        d
        h
    "#}),
    empty()
);

eval_and_assert!(
    raw_string_works,
    indoc! {r#"
        foo = r"hello\nworld";
        print(foo);
        regex = r"\d+";
        print(regex);
    "#},
    equals(indoc! {r#"
        hello\nworld
        \d+
    "#}),
    empty()
);
