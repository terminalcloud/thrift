use test::*;
use mock::*;
use test::generated::*;

use protocol::Type;

#[test]
fn test_simple_struct() {
    let instance = Simple { key: Some(String::from("Hello World!")) };
    let mut protocol = encode(&instance);

    assert_eq!(protocol.log(), &[
        Struct(Begin(String::from("Simple"))),
        Field(Begin((String::from("key"), Type::String, 16))),
        Prim(PString(String::from("Hello World!"))),
        Field(End),
        field_end(),
        Struct(End)
    ]);

    let second = decode::<Simple>(&mut protocol);
    assert_eq!(instance.key, second.key);
}

#[test]
fn test_empty_struct() {
    let mut protocol = encode(&Empty);

    assert_eq!(protocol.log(), &[
        Struct(Begin(String::from("Empty"))),
        field_end(),
        Struct(End)
    ]);

    decode::<Empty>(&mut protocol);
}

#[test]
fn test_recursive_struct() {
    let instance = Recursive {
        recurse: Some(vec![
            Recursive { recurse: Some(vec![]) },
            Recursive { recurse: Some(vec![
                Recursive { recurse: Some(vec![]) }
            ]) }
        ])
    };

    let mut protocol = encode(&instance);

    let type_name = String::from("Recursive");
    let field_name = String::from("recurse");

    assert_eq!(protocol.log(), &[
        Struct(Begin(type_name.clone())),
            Field(Begin((field_name.clone(), Type::List, 0))),
                List(Begin((Type::Struct, 2))),
                    Struct(Begin(type_name.clone())),
                        Field(Begin((field_name.clone(), Type::List, 0))),
                            List(Begin((Type::Struct, 0))),
                            List(End),
                        Field(End),
                        field_end(),
                    Struct(End),

                    Struct(Begin(type_name.clone())),
                        Field(Begin((field_name.clone(), Type::List, 0))),
                            List(Begin((Type::Struct, 1))),
                                Struct(Begin(type_name.clone())),
                                    Field(Begin((field_name.clone(), Type::List, 0))),
                                        List(Begin((Type::Struct, 0))),
                                        List(End),
                                    Field(End),
                                    field_end(),
                                Struct(End),
                            List(End),
                        Field(End),
                        field_end(),
                    Struct(End),
                List(End),
            Field(End),
            field_end(),
        Struct(End)
    ]);

    assert_recursive_equal(instance, decode(&mut protocol));

    fn assert_recursive_equal(first: Recursive, second: Recursive) {
        if first.recurse.as_ref().unwrap().len() != second.recurse.as_ref().unwrap().len() {
             panic!("different recurse lengths")
        }

        for (one, two) in first.recurse.unwrap().into_iter().zip(second.recurse.unwrap().into_iter()) {
            assert_recursive_equal(one, two);
        }
    }
}

#[test]
fn test_nested_list_in_struct() {
    let instance = Nested {
        nested: Some(vec![vec![vec![Simple { key: Some(String::from("Hello World!")) }]]])
    };
    let mut protocol = encode(&instance);

    assert_eq!(protocol.log(), &[
        Struct(Begin(String::from("Nested"))),
            Field(Begin((String::from("nested"), Type::List, 32))),
                List(Begin((Type::List, 1))),
                    List(Begin((Type::List, 1))),
                        List(Begin((Type::Struct, 1))),
                            Struct(Begin(String::from("Simple"))),
                                Field(Begin((String::from("key"), Type::String, 16))),
                                    Prim(PString(String::from("Hello World!"))),
                                Field(End),
                                field_end(),
                            Struct(End),
                        List(End),
                    List(End),
                List(End),
            Field(End),
            field_end(),
        Struct(End)
    ]);

    let second = decode::<Nested>(&mut protocol);
    assert_eq!(instance.nested.unwrap()[0][0][0].key, second.nested.unwrap()[0][0][0].key);
}

#[test]
fn test_struct_with_many_fields() {
    let instance = Many {
        one: Some(17),
        two: Some(String::from("Some String")),
        three: Some(vec![Simple { key: Some(String::from("A String")) }])
    };
    let mut protocol = encode(&instance);

    assert_eq!(protocol.log(), &[
        Struct(Begin(String::from("Many"))),
            Field(Begin((String::from("one"), Type::I32, 3))),
                Prim(I32(17)),
            Field(End),
            Field(Begin((String::from("two"), Type::String, 4))),
                Prim(PString(String::from("Some String"))),
            Field(End),
            Field(Begin((String::from("three"), Type::List, 9))),
                List(Begin((Type::Struct, 1))),
                    Struct(Begin(String::from("Simple"))),
                        Field(Begin((String::from("key"), Type::String, 16))),
                            Prim(PString(String::from("A String"))),
                        Field(End),
                        field_end(),
                    Struct(End),
                List(End),
            Field(End),
            field_end(),
        Struct(End)
    ]);

    let second = decode::<Many>(&mut protocol);
    assert_eq!(instance.one, second.one);
    assert_eq!(instance.two, second.two);
    assert_eq!(instance.three.unwrap()[0].key, second.three.unwrap()[0].key);
}

// #[test]
// fn test_struct_with_optional_field_as_some() {
//     let instance = Optional { this: Some(7489) };
//     let mut protocol = encode(&instance);
//
//     assert_eq!(protocol.log(), &[
//         Struct(Begin(String::from("Optional"))),
//         Field(Begin((String::from("this"), Type::I64, 2))),
//         Prim(I64(7489)),
//         Field(End),
//         field_end(),
//         Struct(End)
//     ]);
//
//     let second = decode::<Optional>(&mut protocol);
//     assert_eq!(instance.this, second.this);
// }
//
// #[test]
// fn test_struct_with_optional_field_as_none() {
//     let instance = Optional { this: None };
//     let mut protocol = encode(&instance);
//
//     assert_eq!(protocol.log(), &[
//         Struct(Begin(String::from("Optional"))),
//         field_end(),
//         Struct(End)
//     ]);
//
//     let second = decode::<Optional>(&mut protocol);
//     assert_eq!(instance.this, second.this);
// }

