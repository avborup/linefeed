use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
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

eval_and_assert!(
    string_join_works,
    indoc! {r#"
        print(["a", "b", "c"].join(", "));
        print(["a", "b", "c"].join());
        print([1, 2, 3].join());
    "#},
    equals(indoc! {r#"
        a, b, c
        abc
        123
    "#}),
    empty()
);

eval_and_assert!(
    string_join_too_many_args_yields_error,
    indoc! {r#"
        print([1,2,3].join(" ", "extra"));
    "#},
    empty(),
    contains("Method join expects 0-1 arguments, but got 2")
);

eval_and_assert!(
    string_index_with_range_works,
    indoc! {r#"
        foo = "hello world";
        n = foo.len();
        print(foo[0..5]);
        print(foo[0..n]);
        print(foo[..n]);
        print(foo[0..]);
        print(foo[..]);
        print(foo[0..-1]);
        print(foo[0..-5]);
        print(foo[-5..-1]);
        print(foo[-5..]);
    "#},
    equals(indoc! {r#"
        hello
        hello world
        hello world
        hello world
        hello world
        hello worl
        hello
        worl
        world
    "#}),
    empty()
);

eval_and_assert!(
    starts_with_returns_true,
    indoc! {r#"
        text = "hello world";
        print(text.starts_with("hello"));
        print(text.starts_with("h"));
        print(text.starts_with(""));
    "#},
    equals(indoc! {r#"
        true
        true
        true
    "#}),
    empty()
);

eval_and_assert!(
    starts_with_returns_false,
    indoc! {r#"
        text = "hello world";
        print(text.starts_with("world"));
        print(text.starts_with("Hello"));
        print(text.starts_with("goodbye"));
    "#},
    equals(indoc! {r#"
        false
        false
        false
    "#}),
    empty()
);
