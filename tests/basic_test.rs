use utid::{ConstantSegment, Spec, Spec2, Spec3, Spec4};

#[test]
fn spec1() {
    let spec = Spec {
        segment: Box::new(ConstantSegment::new(128, 123456)),
    };
    let generated = spec.generate().unwrap();
    assert_eq!(123456, generated);
    let decomposed = spec.decompose(generated).unwrap();
    assert_eq!(123456, decomposed);
}

#[test]
fn spec2() {
    let spec = Spec2 {
        segments: (
            Box::new(ConstantSegment::new(40, 1111)),
            Box::new(ConstantSegment::new(88, 22222)),
        ),
    };
    let generated = spec.generate().unwrap();
    let (first, second) = spec.decompose(generated).unwrap();
    assert_eq!(1111, first);
    assert_eq!(22222, second);
}

#[test]
fn spec3() {
    let spec = Spec3 {
        segments: (
            Box::new(ConstantSegment::new(16, 111)),
            Box::new(ConstantSegment::new(32, 2222)),
            Box::new(ConstantSegment::new(80, 33333)),
        ),
    };
    let generated = spec.generate().unwrap();
    let (first, second, third) = spec.decompose(generated).unwrap();
    assert_eq!(111, first);
    assert_eq!(2222, second);
    assert_eq!(33333, third);
}

#[test]
fn spec4() {
    let spec = Spec4 {
        segments: (
            Box::new(ConstantSegment::new(8, 11)),
            Box::new(ConstantSegment::new(16, 222)),
            Box::new(ConstantSegment::new(32, 3333)),
            Box::new(ConstantSegment::new(72, 44444)),
        ),
    };
    let generated = spec.generate().unwrap();
    let (first, second, third, fourth) = spec.decompose(generated).unwrap();
    assert_eq!(11, first);
    assert_eq!(222, second);
    assert_eq!(3333, third);
    assert_eq!(44444, fourth);
}
