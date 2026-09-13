//! The baseline conversion oracles, expressed as typed data rather than a
//! JSON request. CPU is intentional here, as in the original conversion tests;
//! MPS execution is covered by golden.rs, mps_smoke.rs and shell integration.
use nutorch_core::{Data, Device, Kind, from_data, parse_kind, resolve_kind, to_data};

fn ints(xs: &[i64]) -> Data {
    Data::List(xs.iter().copied().map(Data::Int).collect())
}
fn floats(xs: &[f64]) -> Data {
    Data::List(xs.iter().copied().map(Data::Float).collect())
}

#[test]
fn large_floating_scalar_and_mixed_list_preserve_casts() {
    // Nu's integer type is signed i64. Explicit floating input still keeps
    // the old CPU conversion behavior beyond that integer range.
    let large = u64::MAX as f64;
    let t = from_data(&Data::Float(large), Kind::Double, Device::Cpu).unwrap();
    assert_eq!(to_data(&t).unwrap(), Data::Float(large));
    let values = floats(&[large, 0.5]);
    let t = from_data(&values, Kind::Double, Device::Cpu).unwrap();
    assert_eq!(to_data(&t).unwrap(), values);
}

#[test]
fn flat_int_list_round_trips_as_float32_by_default() {
    let t = from_data(&ints(&[1, 2, 3]), parse_kind(None).unwrap(), Device::Cpu).unwrap();
    assert_eq!(t.kind(), Kind::Float);
    assert_eq!(to_data(&t).unwrap(), floats(&[1., 2., 3.]));
}

#[test]
fn nested_list_round_trips() {
    let t = from_data(
        &Data::List(vec![ints(&[1, 2]), ints(&[3, 4])]),
        Kind::Float,
        Device::Cpu,
    )
    .unwrap();
    assert_eq!(t.size(), [2, 2]);
    assert_eq!(
        to_data(&t).unwrap(),
        Data::List(vec![floats(&[1., 2.]), floats(&[3., 4.])])
    );
}

#[test]
fn scalar_round_trips() {
    let t = from_data(&Data::Int(7), Kind::Float, Device::Cpu).unwrap();
    assert!(t.size().is_empty());
    assert_eq!(to_data(&t).unwrap(), Data::Float(7.));
}

#[test]
fn int64_dtype_round_trips_as_integers() {
    let data = ints(&[1, 2, 3, 9007199254740993, i64::MAX, i64::MIN]);
    let t = from_data(&data, parse_kind(Some("int64")).unwrap(), Device::Cpu).unwrap();
    assert_eq!(t.kind(), Kind::Int64);
    assert_eq!(to_data(&t).unwrap(), data);
}

#[test]
fn ragged_nested_list_is_an_error_not_a_panic() {
    assert!(
        from_data(
            &Data::List(vec![ints(&[1, 2]), ints(&[3])]),
            Kind::Float,
            Device::Cpu
        )
        .is_err()
    );
}

#[test]
fn empty_list_is_an_error() {
    assert!(from_data(&ints(&[]), Kind::Float, Device::Cpu).is_err());
}

#[test]
fn invalid_dtype_is_an_error() {
    assert!(parse_kind(Some("float16")).is_err());
}

#[test]
fn bool_inference_mixed_casts_and_nonfinite_validation() {
    let boolean = Data::List(vec![Data::Bool(true), Data::Bool(false)]);
    assert_eq!(resolve_kind(&boolean, None).unwrap(), Kind::Bool);
    let mixed = Data::List(vec![Data::Bool(true), Data::Int(2)]);
    assert!(resolve_kind(&mixed, None).is_err());
    let kind = resolve_kind(&mixed, Some(Kind::Float)).unwrap();
    assert_eq!(
        to_data(&from_data(&mixed, kind, Device::Cpu).unwrap()).unwrap(),
        floats(&[1., 2.])
    );
    let kind = resolve_kind(&ints(&[0, 2]), Some(Kind::Bool)).unwrap();
    assert_eq!(
        to_data(&from_data(&ints(&[0, 2]), kind, Device::Cpu).unwrap()).unwrap(),
        Data::List(vec![Data::Bool(false), Data::Bool(true)])
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(resolve_kind(&Data::Float(value), Some(Kind::Int64)).is_err());
        let t = from_data(&Data::Float(value), Kind::Double, Device::Cpu).unwrap();
        let Data::Float(actual) = to_data(&t).unwrap() else {
            panic!("expected float")
        };
        if value.is_nan() {
            assert!(actual.is_nan());
        } else {
            assert_eq!(actual.to_bits(), value.to_bits());
        }
    }
}
