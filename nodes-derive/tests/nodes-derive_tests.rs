use nodes::*;
use std::any::{Any, TypeId};

#[derive(InputComponent, FromAnyProto, Debug, PartialEq, Clone)]
struct A {
    lhs: OneOrMany<u32>,
    rhs: One<u32>,
}

#[derive(InputComponent, FromAnyProto, Debug, PartialEq, Clone)]
enum B {
    A(A),
    Other(OneOrMany<u32>),
}

#[derive(InputComponent, FromAnyProto, Debug, PartialEq, Clone)]
struct C {
    numerator: OneOrMany<f32>,
    denominator: OneOrMany<u32>,
}

#[derive(InputComponent, FromAnyProto, Debug, PartialEq, Clone)]
struct D {
    required: OneOrMany<u32>,
    optional: Option<OneOrMany<u32>>,
    optional2: Option<OneOrMany<u32>>,
}

#[test]
fn struct_skip_optional_test() {
    let mut inputs: Vec<Box<dyn Any>> = vec![];
    inputs.push(Box::new(One::new(2u32)));
    inputs.push(Box::new(Option::<()>::None));
    inputs.push(Box::new(One::new(3u32)));
    let d = D::from_any(InputStack::new(&mut inputs, ..)).unwrap();
    assert_eq!(
        d,
        D {
            required: OneOrMany::One(One::new(2)),
            optional: None,
            optional2: Some(OneOrMany::One(One::new(3)))
        }
    );
}

#[test]
fn struct_optional_test() {
    let possible_inputs: PossibleInputs =
        D::possible_inputs(&["required", "optional", "optional2"]);
    let optional_count = possible_inputs.groups.iter().fold(0, |acc, group| {
        acc + group.info.iter().filter(|info| info.optional).count()
    });
    let required_count = possible_inputs.groups.iter().fold(0, |acc, group| {
        acc + group.info.iter().filter(|info| !info.optional).count()
    });
    assert_eq!(optional_count, required_count * 2);

    let mut inputs: Vec<Box<dyn Any>> = vec![];
    inputs.push(Box::new(One::new(2u32)));
    let d = D::from_any(InputStack::new(&mut inputs, ..)).unwrap();
    assert_eq!(
        d,
        D {
            required: OneOrMany::One(One::new(2)),
            optional: None,
            optional2: None
        }
    );

    inputs.push(Box::new(One::new(2u32)));
    inputs.push(Box::new(One::new(3u32)));
    let d = D::from_any(InputStack::new(&mut inputs, ..)).unwrap();
    assert_eq!(
        d,
        D {
            required: OneOrMany::One(One::new(2)),
            optional: Some(OneOrMany::One(One::new(3))),
            optional2: None
        }
    );
}

#[test]
fn input_list_struct_test() {
    let inputs = C::possible_inputs(&["numerator", "denominator"]);
    assert_eq!(inputs.groups.len(), 9);
    assert!(inputs.groups.iter().all(|group| group.info.len() == 2));
}

#[test]
fn input_list_enum_test() {
    let inputs = B::possible_inputs(&["lhs", "rhs"]);
    assert_eq!(inputs.groups.len(), 6);
    assert_eq!(
        inputs
            .groups
            .iter()
            .filter(|group| group.info.len() == 2)
            .count(),
        3
    );
    assert_eq!(
        inputs
            .groups
            .iter()
            .filter(|group| group.info.len() == 1)
            .count(),
        3
    );
}

#[test]
fn from_any_struct_test() {
    let mut inputs: Vec<Box<dyn Any>> = vec![];
    inputs.push(Box::new(One::new(1u32)));
    inputs.push(Box::new(One::new(2u32)));
    let stack = InputStack::new(&mut inputs, ..);
    let result = A::from_any(stack);
    assert_eq!(
        result.ok(),
        Some(A {
            lhs: OneOrMany::One(One::new(1)),
            rhs: One::new(2)
        })
    );
}

#[test]
fn from_any_enum_test() {
    let mut inputs: Vec<Box<dyn Any>> = vec![];
    inputs.push(Box::new(One::new(1u32)));
    let stack = InputStack::new(&mut inputs, ..);
    let result = B::from_any(stack);
    assert_eq!(result.ok(), Some(B::Other(OneOrMany::One(One::new(1)))));
}

#[test]
fn input_component_struct_test() {
    let inner = A {
        lhs: OneOrMany::One(One::new(1)),
        rhs: One::new(2),
    };
    let v = Box::new(inner.clone());
    assert!(A::is(&*v));
    assert_eq!(A::downcast(v).ok(), Some(inner.clone()));
    assert_eq!(A::type_ids(), vec![TypeId::of::<A>()]);
}

#[test]
fn input_component_enum_test() {
    let inner = B::A(A {
        lhs: OneOrMany::One(One::new(1)),
        rhs: One::new(2),
    });
    let v = Box::new(inner.clone());
    assert!(B::is(&*v));
    assert_eq!(B::downcast(v).ok(), Some(inner.clone()));
    let types_ids: Vec<TypeId> = B::type_ids();
    assert_eq!(types_ids.len(), 3);
    assert!(types_ids.contains(&TypeId::of::<B>()));
    assert!(types_ids.contains(&TypeId::of::<A>()));
    assert!(types_ids.contains(&TypeId::of::<OneOrMany<u32>>()));
}
