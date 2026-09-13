//! Frozen PyTorch/MPS fixtures, evaluated through native ownership and typed
//! parameters. JSON exists only at this test's fixture boundary.
use nutorch_core::nn::{SharedModule, SharedOptimizer};
use nutorch_core::{
    Data, SharedTensor,
    operations::{Output, Parameter, Params, execute},
};
use serde_json::Value;

fn data(v: &Value) -> Data {
    match v {
        Value::Bool(b) => Data::Bool(*b),
        Value::Number(n) => n
            .as_i64()
            .map(Data::Int)
            .unwrap_or_else(|| Data::Float(n.as_f64().unwrap())),
        Value::Array(v) => Data::List(v.iter().map(data).collect()),
        _ => panic!("invalid fixture data: {v}"),
    }
}
fn json(v: Data) -> Value {
    match v {
        Data::Bool(v) => v.into(),
        Data::Int(v) => v.into(),
        Data::Float(v) => serde_json::json!(v),
        Data::List(v) => Value::Array(v.into_iter().map(json).collect()),
    }
}
fn input(v: &Value, tracked: bool) -> SharedTensor {
    SharedTensor::create(&data(&v["data"]), v["dtype"].as_str(), tracked).unwrap()
}
fn params(v: &Value, inputs: &[SharedTensor]) -> (Params, Vec<usize>) {
    let mut params = Params::default();
    let mut referenced = vec![];
    for (name, v) in v.as_object().into_iter().flatten() {
        let p = match v {
            Value::Bool(v) => Parameter::Bool(*v),
            Value::Number(v) => v
                .as_i64()
                .map(Parameter::Int)
                .unwrap_or_else(|| Parameter::Float(v.as_f64().unwrap())),
            Value::Array(v) => Parameter::IntList(v.iter().map(|v| v.as_i64().unwrap()).collect()),
            Value::String(v) => {
                if let Some(i) = v.strip_prefix('T').and_then(|v| v.parse::<usize>().ok()) {
                    referenced.push(i);
                    Parameter::Tensor(inputs[i].clone())
                } else {
                    Parameter::Str(v.clone())
                }
            }
            _ => panic!("invalid fixture parameter: {v}"),
        };
        params.values.insert(name.clone(), p);
    }
    (params, referenced)
}
fn run(op: &str, inputs: &[SharedTensor], params: &Params) -> nutorch_core::Result<Output> {
    execute(nutorch_ops::find(op).unwrap(), inputs, params)
}
fn one(op: &str, inputs: &[SharedTensor], params: &Params) -> SharedTensor {
    match run(op, inputs, params).unwrap() {
        Output::Tensors(mut ts) if ts.len() == 1 => ts.remove(0),
        other => panic!("expected one tensor, got {other:?}"),
    }
}
fn table_case(c: &Value) {
    if let Some(seed) = c["seed"].as_i64() {
        run(
            "manual_seed",
            &[],
            &Params {
                values: [("seed".into(), Parameter::Int(seed))].into(),
            },
        )
        .unwrap();
    }
    let tensors = c["tensors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| input(v, false))
        .collect::<Vec<_>>();
    let (p, refs) = params(&c["params"], &tensors);
    let operands = tensors
        .iter()
        .enumerate()
        .filter(|(i, _)| !refs.contains(i))
        .map(|(_, t)| t.clone())
        .collect::<Vec<_>>();
    let result = run(c["op"].as_str().unwrap(), &operands, &p);
    let expected = &c["expect"];
    if let Some(code) = expected["error"].as_str() {
        assert_eq!(result.unwrap_err().0, code);
        return;
    }
    match result.unwrap() {
        Output::Value(v) => assert_eq!(json(v), expected["value"]),
        Output::Tensors(ts) => {
            let actual = ts
                .iter()
                .map(|t| json(t.data().unwrap()))
                .collect::<Vec<_>>();
            assert_eq!(Value::Array(actual), expected["values"]);
        }
        Output::Nothing => assert_eq!(expected["values"], serde_json::json!([])),
    }
}
fn grad_case(c: &Value) {
    let x = input(&c["input"], true);
    let operands = if c["with_self"].as_bool().unwrap_or(false) {
        vec![x.clone(), x.clone()]
    } else if !c["target"].is_null() {
        vec![x.clone(), input(&c["target"], false)]
    } else {
        vec![x.clone()]
    };
    let (p, _) = params(&c["params"], &operands);
    let y = one(c["grad_op"].as_str().unwrap(), &operands, &p);
    let loss = if c["skip_sum"].as_bool().unwrap_or(false) {
        y
    } else if c["square_loss"].as_bool().unwrap_or(false) {
        y.mul(&y).unwrap().sum(None, false).unwrap()
    } else {
        y.sum(None, false).unwrap()
    };
    loss.backward().unwrap();
    assert_eq!(json(x.grad().unwrap().data().unwrap()), c["expect_grad"]);
}

fn float_tensor(v: &Value) -> SharedTensor {
    SharedTensor::create(&data(v), Some("float32"), false).unwrap()
}
fn linear(weights: &Value, bias: &Value) -> SharedModule {
    let mut p = Params::default();
    p.values.insert(
        "in_features".into(),
        Parameter::Int(weights[0].as_array().unwrap().len() as i64),
    );
    p.values.insert(
        "out_features".into(),
        Parameter::Int(weights.as_array().unwrap().len() as i64),
    );
    p.values
        .insert("weight".into(), Parameter::Tensor(float_tensor(weights)));
    if bias.is_null() {
        p.values.insert("no_bias".into(), Parameter::Bool(true));
    } else {
        p.values
            .insert("bias_tensor".into(), Parameter::Tensor(float_tensor(bias)));
    }
    SharedModule::create("linear", &p).unwrap()
}
fn linear_case(c: &Value) {
    let layer = linear(&c["weight"], &c["bias"]);
    let model = if let Some(chain) = c["chain"].as_array().filter(|v| !v.is_empty()) {
        let mut children = vec![layer.clone()];
        children.extend(
            chain
                .iter()
                .map(|v| SharedModule::create(v.as_str().unwrap(), &Params::default()).unwrap()),
        );
        SharedModule::sequential(children).unwrap()
    } else {
        layer.clone()
    };
    let output = model.forward(&float_tensor(&c["input"])).unwrap();
    assert_eq!(json(output.data().unwrap()), c["expect_output"]);
    output.sum(None, false).unwrap().backward().unwrap();
    let parameters = model.parameters().unwrap();
    assert_eq!(
        json(parameters[0].grad().unwrap().data().unwrap()),
        c["expect_weight_grad"]
    );
    if !c["expect_bias_grad"].is_null() {
        assert_eq!(
            json(parameters[1].grad().unwrap().data().unwrap()),
            c["expect_bias_grad"]
        );
    }
    // Sequential retains the live child, so its gradients remain observable.
    assert_eq!(
        json(
            layer.parameters().unwrap()[0]
                .grad()
                .unwrap()
                .data()
                .unwrap()
        ),
        c["expect_weight_grad"]
    );
}
fn optimizer_case(c: &Value) {
    let layer = linear(&c["weight0"], &Value::Null);
    let optimizer = SharedOptimizer::create(
        c["optim_step"].as_str().unwrap(),
        &layer,
        &params(&c["hyper"], &[]).0,
    )
    .unwrap();
    let x = float_tensor(&c["input"]);
    let target = float_tensor(&c["target"]);
    let weight_alias = layer.parameters().unwrap().remove(0);
    let optimizer_alias = optimizer.clone();
    drop(optimizer);
    for expected in c["expect_steps"].as_array().unwrap() {
        optimizer_alias.zero_grad().unwrap();
        let prediction = layer.forward(&x).unwrap();
        one(
            "mse_loss",
            &[prediction, target.clone()],
            &Params::default(),
        )
        .backward()
        .unwrap();
        optimizer_alias.step().unwrap();
        assert_eq!(json(weight_alias.data().unwrap()), *expected);
    }
}
fn module_case(c: &Value) {
    let mut p = params(&c["cargs"], &[]).0;
    for (field, arg) in [("weight", "weight"), ("bias", "bias_tensor")] {
        if !c[field].is_null() {
            p.values
                .insert(arg.into(), Parameter::Tensor(float_tensor(&c[field])));
        }
    }
    let module = SharedModule::create(c["nn_module_forward"].as_str().unwrap(), &p).unwrap();
    if c["eval_mode"].as_bool().unwrap_or(false) {
        module.set_training(false).unwrap();
    }
    let output = module.forward(&input(&c["input"], false)).unwrap();
    assert_eq!(json(output.data().unwrap()), c["expect_output"]);
}

#[test]
fn all_native_cases_match_frozen_pytorch_cases() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("golden.json")).unwrap();
    assert_eq!(cases.len(), 255);
    let mut covered = 0;
    let mut failures = vec![];
    let names = cases
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(names.len(), 255);
    for case in &cases {
        covered += 1;
        if std::panic::catch_unwind(|| {
            if !case["op"].is_null() {
                table_case(case)
            } else if !case["grad_op"].is_null() {
                grad_case(case)
            } else if !case["nn_linear_forward"].is_null() {
                linear_case(case)
            } else if !case["optim_step"].is_null() {
                optimizer_case(case)
            } else if !case["nn_module_forward"].is_null() {
                module_case(case)
            } else {
                panic!("unmapped fixture: {case}")
            }
        })
        .is_err()
        {
            failures.push(case["name"].as_str().unwrap());
        }
    }
    assert_eq!(covered, 255);
    assert!(failures.is_empty(), "native fixture failures: {failures:?}");
}
