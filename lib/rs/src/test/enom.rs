use test::*;
use mock::*;
use test::generated::*;

#[test]
fn test_enum() {
    for op in vec![Operation::Add, Operation::Sub, Operation::Clear] {
        let mut protocol = encode(&op);

        assert_eq!(protocol.log(), &[
            Prim(I32(op as i32))
        ]);

        let other = decode::<Operation>(&mut protocol);
        assert_eq!(other, op);
    }

    assert_eq!(Operation::default(), Operation::Sub);
}

