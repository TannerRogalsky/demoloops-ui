#[test]
fn from_any_test() {
    // #[derive(nodes_derive::InputComponent, Debug, PartialEq)]
    // struct A {
    //     lhs: nodes::One<u32>,
    //     rhs: nodes::One<u32>,
    // }
    // let inner = nodes::One::new(3u32);
    // let v = Box::new(inner.clone());
    // assert!(A::is(&*v));
    // assert_eq!(A::downcast(v).ok(), Some(FromAnyTest::U32(inner)));
    // assert_eq!(A::type_ids(), vec![]);

    // #[derive(nodes_derive::FromAny, Debug, PartialEq)]
    // struct FromAnyTest2 {
    //     f32: FromAnyTest1,
    //     u32: nodes::OneOrMany<u32>
    // }

    // let inner = nodes::OneOrMany::One(nodes::One::new(3u32));
    // let v = Box::new(inner.clone());
    // assert!(FromAnyTest::is(&*v));
    // assert_eq!(FromAnyTest::downcast(v).ok(), Some(FromAnyTest::U32(inner)));
    // assert_eq!(FromAnyTest::type_ids().len(), 6);
}
