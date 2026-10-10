//! Compare the frozen sigmoid-gradient case with direct LibTorch backends.
use tch::{Device, Kind, Tensor};

#[test]
fn sigmoid_gradient_cpu_and_mps_stay_within_one_step_of_f64() {
    let mut gradients = Vec::new();
    for device in [Device::Cpu, Device::Mps] {
        let x = Tensor::from_slice(&[1f32, 2., 3., -1.])
            .reshape([2, 2])
            .to_device(device);
        let w = Tensor::from_slice(&[0.5f32, -0.25, 1.5, 2., -1., 0.75])
            .reshape([3, 2])
            .to_device(device)
            .set_requires_grad(true);
        let b = Tensor::from_slice(&[0.1f32, -0.2, 0.3])
            .to_device(device)
            .set_requires_grad(true);
        let y = (x.matmul(&w.transpose(0, 1)) + &b).sigmoid();
        y.sum(Kind::Float).backward();
        println!(
            "{device:?} output {:?}",
            Vec::<f32>::try_from(y.to_device(Device::Cpu).reshape([-1])).unwrap()
        );
        println!(
            "{device:?} weight gradient {:?}",
            Vec::<f32>::try_from(w.grad().to_device(Device::Cpu).reshape([-1])).unwrap()
        );
        println!(
            "{device:?} bias gradient {:?}",
            Vec::<f32>::try_from(b.grad().to_device(Device::Cpu)).unwrap()
        );
        gradients
            .push(Vec::<f32>::try_from(w.grad().to_device(Device::Cpu).reshape([-1])).unwrap());
    }
    let x = [[1f64, 2.], [3., -1.]];
    let w = [[0.5f64, -0.25], [1.5, 2.], [-1., 0.75]];
    let b = [0.1f32 as f64, -0.2f32 as f64, 0.3f32 as f64];
    for row in 0..3 {
        let mut gradient = [0f64; 2];
        for input in x {
            let y = 1. / (1. + (-(input[0] * w[row][0] + input[1] * w[row][1] + b[row])).exp());
            for column in 0..2 {
                gradient[column] += input[column] * y * (1. - y);
            }
        }
        println!("f64 weight gradient row {row}: {gradient:?}");
        if row == 2 {
            // This is the only element that differs from the frozen gradient.
            let cases: serde_json::Value =
                serde_json::from_str(include_str!("golden.json")).unwrap();
            let fixture = cases
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["name"] == "nn_linear_sigmoid")
                .unwrap();
            let frozen = fixture["expect_weight_grad"][row][0].as_f64().unwrap() as f32;
            let step = f64::from(f32::from_bits(frozen.to_bits() + 1)) - f64::from(frozen);
            assert!((f64::from(frozen) - gradient[0]).abs() <= step);
            for values in &gradients {
                let actual = values[4];
                assert!(
                    (f64::from(actual) - gradient[0]).abs() <= step,
                    "{actual} vs {}",
                    gradient[0]
                );
                assert!(actual.to_bits().abs_diff(frozen.to_bits()) <= 1);
            }
        }
    }
}
