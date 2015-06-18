use test::*;
use mock::*;

macro_rules! test_prim_encode {
    ($({ $testname:ident, $iter:expr, $variant:ident }),*) => {$(
        #[test]
        fn $testname() {
            for val in $iter {
                let protocol = encode(val);
                assert_eq!(protocol.log(), &[Write(Prim($variant(val)))]);
            }
        }
    )*}
}

test_prim_encode! {
    { test_i8_encode,  (0..100), Byte },
    { test_i16_encode, (0..100), I16 },
    { test_i32_encode, (0..100), I32 },
    { test_i64_encode, (0..100), I64 }
}

