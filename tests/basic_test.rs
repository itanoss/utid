use utid::{ConstantSegment, Spec};

#[test]
fn spec1_initiation() {
    let _spec = Spec {
        segment: Box::new(ConstantSegment::new(128, 123456)),
    };
}
