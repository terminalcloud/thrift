use test::*;
use mock::*;

use protocol::Type;

macro_rules! test_prim_encode {
    ($({ $testname:ident, $iter:expr, $variant:ident, $T:ty }),*) => {$(
        #[test]
        fn $testname() {
            for val in $iter {
                let mut protocol = encode(&val);
                assert_eq!(protocol.log(), &[Prim($variant(val.clone()))]);
                assert_eq!(decode::<$T>(&mut protocol), val);
            }
        }
    )*}
}

test_prim_encode! {
    { test_i8_encode,  (0..100), Byte, i8 },
    { test_i16_encode, (0..100), I16, i16 },
    { test_i32_encode, (0..100), I32, i32 },
    { test_i64_encode, (0..100), I64, i64 },
    { test_string_encode, vec![
        String::from("hello"),
        String::from("goodbye"),
        String::from("garbage"),
        String::from("unicode \u{2600}\u{2601}")
    ], PString, String }
}

#[test]
fn test_list() {
    let list = vec![5, 6, 2, 3, 45, 56, 6];
    let mut protocol = encode(&list);

    assert_eq!(protocol.log(), &[
        List(Begin((Type::I32, list.len()))),
        Prim(I32(5)),
        Prim(I32(6)),
        Prim(I32(2)),
        Prim(I32(3)),
        Prim(I32(45)),
        Prim(I32(56)),
        Prim(I32(6)),
        List(End)
    ]);

    assert_eq!(list, decode::<Vec<i32>>(&mut protocol));
}

