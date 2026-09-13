use nutorch_core::{
    Data, Device, Kind, SharedTensor, Tensor,
    nn::{SharedModule, SharedOptimizer},
    operations::{Output, Parameter, Params, execute},
};

fn p<const N: usize>(pairs: [(&str, Parameter); N]) -> Params {
    Params {
        values: pairs.into_iter().map(|(k, v)| (k.into(), v)).collect(),
    }
}
fn data(values: &[f64]) -> Data {
    Data::List(values.iter().copied().map(Data::Float).collect())
}
fn tensor(values: &[f64]) -> SharedTensor {
    SharedTensor::create(&data(values), None, false).unwrap()
}
fn matrix(rows: &[&[f64]]) -> SharedTensor {
    SharedTensor::create(
        &Data::List(rows.iter().map(|r| data(r)).collect()),
        None,
        false,
    )
    .unwrap()
}
fn linear(weight: f64) -> SharedModule {
    SharedModule::create(
        "linear",
        &p([
            ("in_features", Parameter::Int(1)),
            ("out_features", Parameter::Int(1)),
            ("no_bias", Parameter::Bool(true)),
            ("weight", Parameter::Tensor(matrix(&[&[weight]]))),
        ]),
    )
    .unwrap()
}
fn step(model: &SharedModule, optimizer: &SharedOptimizer) {
    model
        .forward(&matrix(&[&[2.0]]))
        .unwrap()
        .sum(None, false)
        .unwrap()
        .backward()
        .unwrap();
    optimizer.step().unwrap();
}

#[test]
fn nondefault_optimizer_options_match_libtorch_optimizers() {
    use tch::nn::OptimizerConfig;
    // Independent LibTorch C++ optimizers are the oracle, not a second copy
    // of our Rust update equations. Every scalar option differs from default.
    for kind in ["sgd", "adam", "adamw", "rmsprop"] {
        let model = linear(1.0);
        let mut params = p([
            ("lr", Parameter::Float(0.1)),
            ("weight_decay", Parameter::Float(0.2)),
        ]);
        let mut reference = match kind {
            "sgd" => {
                params.values.extend([
                    ("momentum".into(), Parameter::Float(0.5)),
                    ("dampening".into(), Parameter::Float(0.25)),
                ]);
                tch::nn::Sgd {
                    momentum: 0.5,
                    dampening: 0.25,
                    wd: 0.2,
                    nesterov: false,
                }
                .build_copt(0.1)
                .unwrap()
            }
            "adam" | "adamw" => {
                params.values.extend([
                    ("beta1".into(), Parameter::Float(0.5)),
                    ("beta2".into(), Parameter::Float(0.25)),
                    ("eps".into(), Parameter::Float(0.5)),
                ]);
                if kind == "adam" {
                    tch::nn::Adam {
                        beta1: 0.5,
                        beta2: 0.25,
                        wd: 0.2,
                        eps: 0.5,
                        amsgrad: false,
                    }
                    .build_copt(0.1)
                    .unwrap()
                } else {
                    tch::nn::AdamW {
                        beta1: 0.5,
                        beta2: 0.25,
                        wd: 0.2,
                        eps: 0.5,
                        amsgrad: false,
                    }
                    .build_copt(0.1)
                    .unwrap()
                }
            }
            _ => {
                params.values.extend([
                    ("alpha".into(), Parameter::Float(0.5)),
                    ("eps".into(), Parameter::Float(0.5)),
                    ("momentum".into(), Parameter::Float(0.25)),
                ]);
                tch::nn::RmsProp {
                    alpha: 0.5,
                    eps: 0.5,
                    wd: 0.2,
                    momentum: 0.25,
                    centered: false,
                }
                .build_copt(0.1)
                .unwrap()
            }
        };
        let reference_weight =
            Tensor::ones([1, 1], (Kind::Float, Device::Mps)).set_requires_grad(true);
        reference.add_parameters(&reference_weight, 0).unwrap();
        let native = SharedOptimizer::create(kind, &model, &params).unwrap();
        let held = native.clone();
        let parameter = model.parameters().unwrap().remove(0);
        for lr in [0.1, 0.2, 0.05] {
            reference.set_learning_rate(lr).unwrap();
            held.set_lr(lr).unwrap();
            reference_weight
                .f_mul_scalar(2.)
                .unwrap()
                .f_sum(Kind::Float)
                .unwrap()
                .f_backward()
                .unwrap();
            reference.step().unwrap();
            step(&model, &native);
            let Data::List(rows) = parameter.data().unwrap() else {
                panic!()
            };
            let Data::List(values) = &rows[0] else {
                panic!()
            };
            let Data::Float(actual) = values[0] else {
                panic!()
            };
            let expected = reference_weight
                .f_to_device(Device::Cpu)
                .unwrap()
                .double_value(&[0, 0]);
            assert!(
                (actual - expected).abs() < 1e-6,
                "{kind}, lr={lr}: {actual} vs LibTorch {expected}"
            );
            reference.zero_grad().unwrap();
            held.zero_grad().unwrap();
        }
    }
}

#[test]
fn module_optimizer_aliases_survive_scope_and_keep_parameter_identity() {
    for kind in ["sgd", "adam", "adamw", "rmsprop"] {
        let model = linear(1.0);
        let child = model.clone();
        let seq = SharedModule::sequential(vec![model.clone()]).unwrap();
        assert!(SharedModule::sequential(vec![model.clone(), model.clone()]).is_err());
        assert!(SharedModule::sequential(vec![seq.clone(), child.clone()]).is_err());
        assert!(SharedModule::sequential(vec![]).is_err());
        let parameter = model.parameters().unwrap().remove(0);
        let optimizer =
            SharedOptimizer::create(kind, &seq, &p([("lr", Parameter::Float(0.1))])).unwrap();
        let alias = optimizer.clone();
        // Undefined grads skip both weight decay and state initialization.
        optimizer.step().unwrap();
        assert_eq!(parameter.data().unwrap(), Data::List(vec![data(&[1.0])]));
        assert!(
            optimizer
                .describe()
                .unwrap()
                .contains(&"state_bytes: 0".into())
        );
        step(&child, &optimizer);
        assert_ne!(parameter.data().unwrap(), Data::List(vec![data(&[1.0])]));
        assert_eq!(
            parameter.grad().unwrap().data().unwrap(),
            Data::List(vec![data(&[2.0])])
        );
        child.zero_grad().unwrap();
        assert_eq!(
            parameter.grad().unwrap().data().unwrap(),
            Data::List(vec![data(&[0.0])])
        );
        parameter
            .mul(&parameter)
            .unwrap()
            .sum(None, false)
            .unwrap()
            .backward()
            .unwrap();
        drop(model);
        drop(child);
        drop(seq);
        drop(optimizer);
        let before = parameter.data().unwrap();
        alias.step().unwrap();
        assert_ne!(
            before,
            parameter.data().unwrap(),
            "{kind} lost its parameters"
        );
        alias.zero_grad().unwrap();
        assert_eq!(
            parameter.grad().unwrap().data().unwrap(),
            Data::List(vec![data(&[0.0])])
        );
        alias.set_lr(0.0).unwrap();
        assert!(alias.describe().unwrap().contains(&"lr: 0".into()));
    }
}

#[test]
fn sgd_hand_checked_momentum_and_shared_learning_rate() {
    let model = linear(1.0);
    let opt = SharedOptimizer::create(
        "sgd",
        &model,
        &p([
            ("lr", Parameter::Float(0.1)),
            ("momentum", Parameter::Float(0.5)),
        ]),
    )
    .unwrap();
    let weight = model.parameters().unwrap().remove(0);
    step(&model, &opt); // g = 2, buffer = 2, w = .8
    let expected = (1.0f32 - 0.2f32) as f64;
    assert_eq!(weight.data().unwrap(), Data::List(vec![data(&[expected])]));
    opt.zero_grad().unwrap();
    let alias = opt.clone();
    alias.set_lr(0.2).unwrap();
    step(&model, &opt); // buffer = .5*2 + 2 = 3; w = .8 - .6
    let expected = ((expected as f32) - 0.6f32) as f64;
    assert_eq!(weight.data().unwrap(), Data::List(vec![data(&[expected])]));
    assert!(SharedOptimizer::create("sgd", &model, &Params::default()).is_err());
    assert!(
        SharedOptimizer::create(
            "sgd",
            &model,
            &p([
                ("lr", Parameter::Float(0.1)),
                ("nesterov", Parameter::Bool(true))
            ])
        )
        .is_err()
    );
    assert!(
        SharedOptimizer::create(
            "adam",
            &SharedModule::create("relu", &Params::default()).unwrap(),
            &Params::default()
        )
        .is_err()
    );
}

#[test]
fn dropout_distribution_gradient_and_shared_recursive_mode() {
    let x = SharedTensor::create(&data(&vec![1.; 1000]), None, true).unwrap();
    let dropout = SharedModule::create("dropout", &p([("p", Parameter::Float(0.25))])).unwrap();
    let seq = SharedModule::sequential(vec![dropout.clone()]).unwrap();
    let output = seq.forward(&x).unwrap();
    let Data::List(values) = output.data().unwrap() else {
        panic!()
    };
    let zeros = values.iter().filter(|v| **v == Data::Float(0.0)).count();
    assert!(((zeros as f64 / values.len() as f64) - 0.25).abs() < 0.07);
    for v in values {
        let Data::Float(v) = v else { panic!() };
        assert!(v == 0. || (v - 1.0 / 0.75).abs() < 1e-5);
    }
    output.sum(None, false).unwrap().backward().unwrap();
    assert_eq!(x.grad().unwrap().data().unwrap(), output.data().unwrap());
    seq.set_training(false).unwrap();
    assert!(!dropout.is_training().unwrap());
    assert_eq!(seq.forward(&x).unwrap().data().unwrap(), x.data().unwrap());
    dropout.set_training(true).unwrap();
    assert!(seq.is_training().unwrap());
    for (probability, expected) in [(0., 1.), (1., 0.)] {
        let m =
            SharedModule::create("dropout", &p([("p", Parameter::Float(probability))])).unwrap();
        assert_eq!(
            m.forward(&x).unwrap().data().unwrap(),
            data(&vec![expected; 1000])
        );
    }
    assert!(SharedModule::create("dropout", &p([("p", Parameter::Float(1.5))])).is_err());
}

#[test]
fn batch_norm_buffers_state_names_load_rejection_and_parameter_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let model = || {
        SharedModule::sequential(vec![
            SharedModule::create(
                "linear",
                &p([
                    ("in_features", Parameter::Int(2)),
                    ("out_features", Parameter::Int(2)),
                    ("weight", Parameter::Tensor(matrix(&[&[1., 0.], &[0., 1.]]))),
                    ("bias_tensor", Parameter::Tensor(tensor(&[0., 0.]))),
                ]),
            )
            .unwrap(),
            SharedModule::create("batch_norm", &p([("num_features", Parameter::Int(2))])).unwrap(),
        ])
        .unwrap()
    };
    let original = model();
    let x = matrix(&[&[10., -10.], &[12., -8.]]);
    original.set_training(false).unwrap();
    let before = original.forward(&x).unwrap().data().unwrap();
    original.set_training(true).unwrap();
    original.forward(&x).unwrap();
    original.set_training(false).unwrap();
    let expected = original.forward(&x).unwrap().data().unwrap();
    assert_ne!(before, expected);
    let path = dir.path().join("state.safetensors");
    original.save(&path).unwrap();
    let state = Tensor::read_safetensors(&path).unwrap();
    let mut names = state.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        [
            "0.bias",
            "0.weight",
            "1.bias",
            "1.num_batches_tracked",
            "1.running_mean",
            "1.running_var",
            "1.weight"
        ]
    );
    assert_eq!(
        state
            .iter()
            .find(|(k, _)| k == "1.num_batches_tracked")
            .unwrap()
            .1
            .int64_value(&[]),
        1
    );
    let fresh = model();
    fresh.set_training(false).unwrap();
    let alias = fresh.parameters().unwrap().remove(0);
    let opt = SharedOptimizer::create("sgd", &fresh, &p([("lr", Parameter::Float(0.1))])).unwrap();
    fresh.load(&path).unwrap();
    assert_eq!(fresh.forward(&x).unwrap().data().unwrap(), expected);
    let malformed = dir.path().join("malformed.safetensors");
    for scenario in [
        "missing",
        "unexpected",
        "shape",
        "dtype",
        "corrupt",
        "absent",
    ] {
        let mut values = state
            .iter()
            .map(|(k, t)| (k.clone(), t.shallow_clone()))
            .collect::<Vec<_>>();
        match scenario {
            "missing" => {
                values.pop();
            }
            "unexpected" => values.push((
                "extra".into(),
                Tensor::zeros([1], (Kind::Float, Device::Cpu)),
            )),
            "shape" => values[0].1 = Tensor::zeros([99], (Kind::Float, Device::Cpu)),
            "dtype" => {
                // Float64 cannot transfer to MPS under the baseline load
                // contract. Other entries deliberately differ, proving none
                // of those earlier prepared values are applied on failure.
                for (_, value) in &mut values {
                    *value = value.f_add_scalar(1).unwrap();
                }
                let last = values.last_mut().unwrap();
                last.1 = last.1.f_to_kind(Kind::Double).unwrap();
            }
            _ => (),
        }
        Tensor::write_safetensors(&values, &malformed).unwrap();
        if scenario == "corrupt" {
            std::fs::write(&malformed, b"not a tensor archive").unwrap();
        }
        if scenario == "absent" {
            std::fs::remove_file(&malformed).unwrap();
        }
        assert!(fresh.load(&malformed).is_err(), "{scenario}");
        assert_eq!(
            fresh.forward(&x).unwrap().data().unwrap(),
            expected,
            "{scenario} changed state"
        );
    }
    let weights_before = alias.data().unwrap();
    fresh
        .forward(&x)
        .unwrap()
        .sum(None, false)
        .unwrap()
        .backward()
        .unwrap();
    opt.step().unwrap();
    assert_ne!(weights_before, alias.data().unwrap());
    assert_ne!(fresh.forward(&x).unwrap().data().unwrap(), expected);
}

#[test]
fn typed_validation_and_creation_autograd_regressions() {
    for name in ["randn", "rand", "ones", "zeros", "full"] {
        let mut params = p([
            ("shape", Parameter::IntList(vec![2])),
            ("requires_grad", Parameter::Bool(true)),
        ]);
        if name == "full" {
            params.values.insert("value".into(), Parameter::Float(2.));
        }
        let Output::Tensors(ts) = execute(nutorch_ops::find(name).unwrap(), &[], &params).unwrap()
        else {
            panic!()
        };
        let x = &ts[0];
        assert!(x.metadata().unwrap().requires_grad);
        assert_eq!(x.metadata().unwrap().device, "mps");
        x.mul(x)
            .unwrap()
            .sum(None, false)
            .unwrap()
            .backward()
            .unwrap();
        assert_eq!(
            x.grad().unwrap().data().unwrap(),
            x.add(x, None).unwrap().data().unwrap()
        );
    }
    for (kind, args) in [
        (
            "conv2d",
            p([
                ("in_channels", Parameter::Int(4)),
                ("out_channels", Parameter::Int(4)),
                ("kernel_size", Parameter::Int(3)),
                ("groups", Parameter::Int(0)),
            ]),
        ),
        (
            "conv2d",
            p([
                ("in_channels", Parameter::Int(4)),
                ("out_channels", Parameter::Int(4)),
                ("kernel_size", Parameter::Int(3)),
                ("groups", Parameter::Int(3)),
            ]),
        ),
        (
            "conv_transpose2d",
            p([
                ("in_channels", Parameter::Int(4)),
                ("out_channels", Parameter::Int(3)),
                ("kernel_size", Parameter::Int(2)),
                ("groups", Parameter::Int(2)),
            ]),
        ),
    ] {
        assert!(SharedModule::create(kind, &args).is_err());
    }
    assert!(
        SharedModule::create(
            "linear",
            &p([
                ("in_features", Parameter::Int(1)),
                ("out_features", Parameter::Int(2)),
                ("weight", Parameter::Tensor(matrix(&[&[1.]])))
            ])
        )
        .is_err()
    );
    assert!(SharedModule::create("unknown", &Params::default()).is_err());
    let conv_error = SharedModule::create(
        "conv2d",
        &p([
            ("in_channels", Parameter::Int(1)),
            ("out_channels", Parameter::Int(2)),
            ("kernel_size", Parameter::Int(2)),
            ("weight", Parameter::Tensor(matrix(&[&[1., 2.]]))),
        ]),
    )
    .unwrap_err();
    assert_eq!(conv_error.0, "shape_mismatch");
    assert!(SharedModule::create("relu", &p([("unknown", Parameter::Int(1))])).is_err());
    assert!(
        SharedModule::create(
            "linear",
            &p([
                ("in_features", Parameter::Float(1.)),
                ("out_features", Parameter::Int(2))
            ])
        )
        .is_err()
    );
    let x = tensor(&[1.]);
    let a = matrix(&[&[1., 2., 3.], &[4., 5., 6.]]);
    let b = tensor(&[1., 2., 3., 4.]);
    let mismatch = execute(
        nutorch_ops::find("add").unwrap(),
        &[a.clone(), b],
        &Params::default(),
    )
    .unwrap_err();
    assert_eq!(mismatch.0, "shape_mismatch");
    assert!(mismatch.1.contains("[2, 3]") && mismatch.1.contains("[4]"));
    let mismatch = execute(
        nutorch_ops::find("mm").unwrap(),
        &[a.clone(), a],
        &Params::default(),
    )
    .unwrap_err();
    assert_eq!(mismatch.0, "shape_mismatch");
    assert!(mismatch.1.contains("[2, 3]"));
    let reduction = execute(
        nutorch_ops::find("mse_loss").unwrap(),
        &[x.clone(), x.clone()],
        &p([("reduction", Parameter::Str("median".into()))]),
    )
    .unwrap_err();
    assert_eq!(reduction.0, "bad_argument");
    assert!(reduction.1.contains("mean, sum, or none"));
    assert!(
        execute(
            nutorch_ops::find("add").unwrap(),
            &[x.clone()],
            &Params::default()
        )
        .is_err()
    );
    assert!(
        execute(
            nutorch_ops::find("sum").unwrap(),
            &[x.clone()],
            &p([("unknown", Parameter::Int(1))])
        )
        .is_err()
    );
    assert!(
        execute(
            nutorch_ops::find("sum").unwrap(),
            &[x.clone()],
            &p([("dim", Parameter::Float(1.))])
        )
        .is_err()
    );
    assert!(execute(nutorch_ops::find("pow").unwrap(), &[x], &Params::default()).is_err());
}

#[test]
fn reads_state_saved_by_the_baseline_daemon() {
    let model = SharedModule::sequential(vec![
        SharedModule::create(
            "linear",
            &p([
                ("in_features", Parameter::Int(2)),
                ("out_features", Parameter::Int(2)),
            ]),
        )
        .unwrap(),
        SharedModule::create("batch_norm", &p([("num_features", Parameter::Int(2))])).unwrap(),
    ])
    .unwrap();
    let weights = model.parameters().unwrap();
    let optimizer =
        SharedOptimizer::create("sgd", &model, &p([("lr", Parameter::Float(0.1))])).unwrap();
    model
        .load(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/legacy-linear-batchnorm.safetensors"
        )))
        .unwrap();
    model.set_training(false).unwrap();
    let x = matrix(&[&[1., 2.], &[3., 4.]]);
    assert_eq!(
        model.forward(&x).unwrap().data().unwrap(),
        Data::List(vec![
            data(&[2.829894542694092, 2.6749520301818848]),
            data(&[6.48137092590332, 6.9548749923706055])
        ])
    );
    let before = weights[0].data().unwrap();
    model
        .forward(&x)
        .unwrap()
        .sum(None, false)
        .unwrap()
        .backward()
        .unwrap();
    optimizer.step().unwrap();
    assert_ne!(weights[0].data().unwrap(), before);
}

#[test]
fn group_norm_preserves_the_baseline_direct_kernel_oracle() {
    let values = Data::List(vec![Data::List(vec![
        data(&[1., 2.]),
        data(&[3., 4.]),
        data(&[5., 6.]),
        data(&[7., 8.]),
    ])]);
    let x = SharedTensor::create(&values, None, false).unwrap();
    let module = SharedModule::create(
        "group_norm",
        &p([
            ("num_groups", Parameter::Int(2)),
            ("num_channels", Parameter::Int(4)),
        ]),
    )
    .unwrap();
    let actual = module.forward(&x).unwrap().data().unwrap();
    let raw = nutorch_core::from_data(&values, Kind::Float, Device::Mps).unwrap();
    let expected = raw
        .f_group_norm(2, None::<&Tensor>, None::<&Tensor>, 1e-5, true)
        .unwrap()
        .f_to_device(Device::Cpu)
        .unwrap();
    assert_eq!(actual, nutorch_core::to_data(&expected).unwrap());
    assert!(
        SharedModule::create(
            "group_norm",
            &p([
                ("num_groups", Parameter::Int(3)),
                ("num_channels", Parameter::Int(4))
            ])
        )
        .is_err()
    );
}
