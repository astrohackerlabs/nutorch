//! Toolchain proof (issue 0002, Experiment 1): the tch/libtorch stack builds,
//! MPS is available, and an all-ones matmul returns exact values on both
//! devices. All expected values are integer-valued f32 (exact, no tolerance).

use tch::{Device, Kind, Tensor};

fn assert_ones_matmul_exact(device: Device) {
    let a = Tensor::ones([4, 4], (Kind::Float, device));
    let b = Tensor::ones([4, 4], (Kind::Float, device));
    let c = a.matmul(&b).to_device(Device::Cpu);

    let values: Vec<f32> = Vec::<f32>::try_from(c.reshape(16)).unwrap();
    assert_eq!(values.len(), 16);
    for (i, v) in values.iter().enumerate() {
        assert_eq!(*v, 4.0, "element {i} on {device:?} is {v}, expected 4.0");
    }

    let mean = c.mean(Kind::Float);
    let mean_value: f64 = mean.double_value(&[]);
    assert_eq!(mean_value, 4.0, "mean on {device:?}");
}

#[test]
fn cpu_ones_matmul_is_exact() {
    assert_ones_matmul_exact(Device::Cpu);
}

#[test]
fn mps_is_available() {
    assert!(
        tch::utils::has_mps(),
        "MPS not available through this libtorch build"
    );
}

#[test]
fn mps_ones_matmul_is_exact() {
    assert!(tch::utils::has_mps(), "MPS not available");
    assert_ones_matmul_exact(Device::Mps);
}
