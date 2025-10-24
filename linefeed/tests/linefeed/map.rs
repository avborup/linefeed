use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    map_can_be_created,
    indoc! {r#"
        map = {};
        print(map);

        map = {1: 2};
        print(map);
    "#},
    equals(indoc! {r#"
        {}
        {1: 2}
    "#}),
    empty()
);

eval_and_assert!(
    map_can_be_accessed,
    indoc! {r#"
        map = {
            1: 2,
            "wow": "cool",
            (1, 2): -1,
        };

        print(map["wow"]);
        print(map[1]);
        print(map[(1, 2)]);
        print(map[3]);
    "#},
    equals(indoc! {r#"
        cool
        2
        -1
        null
    "#}),
    empty()
);

eval_and_assert!(
    map_can_be_updated,
    indoc! {r#"
        map = {
            1: 2,
            "wow": "cool",
            (1, 2): -1,
        };

        map["wow"] = "not cool";
        map[(1, 2)] = 0;

        print(map[1]);
        print(map["wow"]);
        print(map[(1, 2)]);
        print(map[3]);
    "#},
    equals(indoc! {r#"
        2
        not cool
        0
        null
    "#}),
    empty()
);

eval_and_assert!(
    nested_map_can_be_updated,
    indoc! {r#"
        map = {
            "foo": {
                1: 5,
                2: 6,
            },
            "bar": {
                3: 7,
                4: 8,
            },
        };

        map["foo"][1] = 10;
        map["bar"][3] = 11;

        print(map["foo"][1]);
        print(map["foo"][2]);
        print(map["bar"][3]);
        print(map["bar"][4]);
    "#},
    equals(indoc! {r#"
        10
        6
        11
        8
    "#}),
    empty()
);

eval_and_assert!(
    map_can_be_iterated,
    indoc! {r#"
        map = { 1: 2 };

        for kv in map {
            print(kv);
        };
    "#},
    equals("(1, 2)"),
    empty()
);

eval_and_assert!(
    map_default_generator_works,
    indoc! {r#"
        m = map(fn() 42);
        m["b"] = 100;
        print(m["a"], m["b"]);
    "#},
    equals("42 100"),
    empty()
);
