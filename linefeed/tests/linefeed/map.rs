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
    map_get_all_with_existing_keys,
    indoc! {r#"
        map = {
            "a": 1,
            "b": 2,
            "c": 3,
        };

        keys = ["a", "b", "c"];
        values = map.get_all(keys);
        print(values);
    "#},
    equals("[1, 2, 3]"),
    empty()
);

eval_and_assert!(
    map_get_all_with_missing_keys,
    indoc! {r#"
        map = {
            "a": 1,
            "b": 2,
        };

        keys = ["a", "c", "b"];
        values = map.get_all(keys);
        print(values);
    "#},
    equals("[1, null, 2]"),
    empty()
);

eval_and_assert!(
    map_get_all_with_empty_iterable,
    indoc! {r#"
        map = {
            "a": 1,
            "b": 2,
        };

        keys = [];
        values = map.get_all(keys);
        print(values);
    "#},
    equals("[]"),
    empty()
);

eval_and_assert!(
    map_get_all_with_range,
    indoc! {r#"
        map = {
            0: "zero",
            1: "one",
            2: "two",
            3: "three",
        };

        values = map.get_all(0..3);
        print(values);
    "#},
    equals(r#"["zero", "one", "two"]"#),
    empty()
);

eval_and_assert!(
    map_get_all_with_tuple,
    indoc! {r#"
        map = {
            "x": 10,
            "y": 20,
            "z": 30,
        };

        values = map.get_all(("x", "z"));
        print(values);
    "#},
    equals("[10, 30]"),
    empty()
);

eval_and_assert!(
    map_values_basic,
    indoc! {r#"
        map = {
            "a": 1,
            "b": 2,
            "c": 3,
        };

        values = map.values();
        sorted_values = values.sort();
        print(sorted_values);
    "#},
    equals("[1, 2, 3]"),
    empty()
);

eval_and_assert!(
    map_values_empty,
    indoc! {r#"
        map = {};
        values = map.values();
        print(values);
    "#},
    equals("[]"),
    empty()
);

eval_and_assert!(
    map_values_with_duplicates,
    indoc! {r#"
        map = {
            "x": 10,
            "y": 10,
            "z": 20,
        };

        values = map.values();
        sorted_values = values.sort();
        print(sorted_values);
    "#},
    equals("[10, 10, 20]"),
    empty()
);

eval_and_assert!(
    map_keys_basic,
    indoc! {r#"
        map = {
            "a": 1,
            "b": 2,
            "c": 3,
        };

        keys = map.keys();
        sorted_keys = keys.sort();
        print(sorted_keys);
    "#},
    equals("[\"a\", \"b\", \"c\"]"),
    empty()
);

eval_and_assert!(
    map_keys_empty,
    indoc! {r#"
        map = {};
        keys = map.keys();
        print(keys);
    "#},
    equals("[]"),
    empty()
);

eval_and_assert!(
    map_keys_numeric,
    indoc! {r#"
        map = {
            3: "three",
            1: "one",
            2: "two",
        };

        keys = map.keys();
        sorted_keys = keys.sort();
        print(sorted_keys);
    "#},
    equals("[1, 2, 3]"),
    empty()
);

eval_and_assert!(
    map_keys_mixed_types,
    indoc! {r#"
        map = {
            "name": "Alice",
            42: "answer",
            (1, 2): "tuple",
        };

        keys = map.keys();
        print(keys.len());
    "#},
    equals("3"),
    empty()
);
