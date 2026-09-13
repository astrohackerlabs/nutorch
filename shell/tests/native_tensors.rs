use std::{
    fs,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

struct Shell {
    dir: tempfile::TempDir,
}

#[test]
fn explicit_imports_control_scope_and_preserve_native_values() {
    let shell = Shell::new();
    // `use`/`hide` are parse-time operations. Inspect distinct source units so
    // a later hide cannot make a pre-import visibility assertion pass by accident.
    for script in [
        "scope commands | where name =~ '^(torch|nutorch)( |$)' | length",
        "let escaped = (do { use torch; torch tensor [2 3] }); scope commands | where name =~ '^(torch|nutorch)( |$)' | length",
        "use torch; hide torch; scope commands | where name == 'torch tensor' | length",
    ] {
        assert_eq!(shell.run_raw(script, true).trim(), "0");
    }
    assert_eq!(
        shell
            .run_raw("scope modules | where name == torch | length", true)
            .trim(),
        "1"
    );
    assert_eq!(
        shell
            .run_raw(
                "use torch; scope commands | where name == 'torch tensor' | get type | first",
                true
            )
            .trim(),
        "built-in"
    );
    assert_eq!(shell.run_raw("use torch; use torch tensor; let names = (scope commands | where name in ['torch tensor' tensor]); ($names | get decl_id | uniq | length) == 1 and ($names | length) == 2", true).trim(), "true");
    let output = shell.run_raw(
        r#"
        let escaped = (do { use torch; torch tensor [2 3] })
        use torch
        alias make_tensor = torch tensor
        if (make_tensor [1] | describe) != tensor { error make {msg: 'alias lost native value'} }
        module regular { export def sum [] { 5 } }
        def sum [] { -1 }
        let regular_shadow = (do { use regular *; sum })
        let native_shadow = (do { use torch *; tensor [2 3] | sum | value })
        if $regular_shadow != $native_shadow { error make {msg: 'module shadowing differs'} }
        let qualified = ($escaped | torch mul $escaped | torch value)
        use torch
        let repeated = ($escaped | torch value)
        let selective = (do { use torch [tensor value]; tensor [4 5] | value })
        let wildcard = (do { use torch *; tensor [6 7] | add (tensor [1 1]) | value })
        let compat = (do { use torch nutorch; nutorch tensor [8 9] | nutorch value })
        module exported { export use torch }
        let reexported = (do { use exported; exported torch tensor [10] | exported torch value })
        {qualified: $qualified, repeated: $repeated, selective: $selective,
         wildcard: $wildcard, compat: $compat, reexported: $reexported}
        | to json --raw
    "#,
        true,
    );
    let actual: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        actual,
        serde_json::json!({
        "qualified":[4.,9.],"repeated":[2.,3.],"selective":[4.,5.],
        "wildcard":[7.,8.],"compat":[8.,9.],"reexported":[10.]})
    );
}

#[test]
fn imports_preserve_rng_and_optimizer_state_and_support_scripts_and_config() {
    let shell = Shell::new();
    let script = r#"
        use torch
        torch manual_seed 42
        let first = torch randn [3]
        let second = torch randn [3]
        torch manual_seed 42
        let replay_first = torch randn [3]
        use torch
        let replay_second = torch randn [3]
        let held = (do {
            use torch
            let model = torch nn linear 1 1 --no-bias --weight (torch tensor [[0.0]])
            {model: $model, parameter: (torch nn parameters $model).0,
             optimizer: (torch nn sgd $model --lr 0.1 --momentum 0.5)}
        })
        let opt_alias = $held.optimizer
        let input = torch tensor [[1.0]]
        let target = torch tensor [[2.0]]
        for i in 0..1 {
            use torch
            $input | torch forward $held.model | torch mse_loss $target | torch backward
            torch step $opt_alias
            torch nn zero_grad $held.optimizer
        }
        {rng: (($first | torch equal $replay_first) and ($second | torch equal $replay_second)),
         parameter: ($held.parameter | torch value),
         prediction: ($input | torch forward $held.model | torch value)} | to json --raw
    "#;
    let path = shell.dir.path().join("import.nu");
    fs::write(&path, script).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&shell.invoke(
        &["--no-config-file", "--no-std-lib", path.to_str().unwrap()],
        true,
    ))
    .unwrap();
    assert_eq!(actual["rng"], true);
    let weight = actual["parameter"][0][0].as_f64().unwrap();
    assert!(
        (weight - 0.92).abs() < 1e-6,
        "SGD momentum must survive imports: {actual}"
    );
    assert_eq!(actual["parameter"], actual["prediction"]);

    let config = shell.dir.path().join("config.nu");
    fs::write(&config, "use torch\n").unwrap();
    let out = shell.invoke(
        &[
            "--config",
            config.to_str().unwrap(),
            "-i",
            "-c",
            "torch tensor [2 3] | torch value | to json --raw",
        ],
        true,
    );
    assert_eq!(out.trim(), "[2.0,3.0]");
}

#[test]
fn baseline_nonfinite_predicates_shapes_and_module_parameters() {
    let output = Shell::new().run(r#"
        let bad = torch tensor [0.0 1.0 -1.0 1.0] | torch div (torch tensor [0.0 0.0 0.0 1.0])
        let x = torch tensor [-1.5 0.0 2.0]
        let initial = torch tensor [[1.0 2.0] [3.0 4.0] [5.0 6.0]]
        let explicit = torch nn linear 2 3 --weight $initial --no-bias
        let model = torch nn linear 2 3
        let params = torch nn parameters $model
        let gradient_model = torch nn linear 2 1
        torch tensor [[1.0 2.0]] | torch forward $gradient_model | torch sum | torch backward
        {isnan: ($bad | torch isnan | torch value), isinf: ($bad | torch isinf | torch value),
         isfinite: ($bad | torch isfinite | torch value), isposinf: ($bad | torch isposinf | torch value),
         isneginf: ($bad | torch isneginf | torch value),
         replaced: ($bad | torch nan_to_num --nan 0.5 --posinf 100 --neginf -100 | torch value),
         shape: (torch full [2 3] 1 | torch shape), scalar_shape: (torch tensor 3.0 | torch shape),
         mean: (torch tensor [1 2 3 4] --dtype int64 | torch mean | torch value --meta),
         source_untracked: ($initial | to json | str contains 'requires_grad=false'),
         copied_tracked: ((torch nn parameters $explicit).0 | to json | str contains 'requires_grad=true'),
         parameter_shapes: ($params | each {torch shape $in}),
         parameters_on_mps: ($params | all {|p| $p | to json | str contains 'device=mps'}),
         parameters_tracked: ($params | all {|p| $p | to json | str contains 'requires_grad=true'}),
         gradient: ((torch nn parameters $gradient_model).0 | torch grad | torch value),
         relu: ($x | torch forward (torch nn relu) | torch equal ($x | torch relu)),
         sigmoid: ($x | torch forward (torch nn sigmoid) | torch equal ($x | torch sigmoid)),
         tanh: ($x | torch forward (torch nn tanh) | torch equal ($x | torch tanh))} | to json --raw
    "#, true);
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = serde_json::json!({
        "isnan": [true,false,false,false], "isinf": [false,true,true,false], "isfinite": [false,false,false,true],
        "isposinf": [false,true,false,false], "isneginf": [false,false,true,false], "replaced": [0.5,100.,-100.,1.],
        "shape": [2,3], "scalar_shape": [], "mean": {"data":2.5,"shape":[],"dtype":"float32"},
        "source_untracked":true,"copied_tracked":true,"parameter_shapes":[[3,2],[3]],
        "parameters_on_mps":true,"parameters_tracked":true,"gradient":[[1.,2.]],"relu":true,"sigmoid":true,"tanh":true
    });
    assert_eq!(v, expected);
}

#[test]
fn every_loss_supports_native_reduction_modes_and_log_targets() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../../core/tests/golden.json")).unwrap();
    let shell = Shell::new();
    let mut seen = std::collections::HashSet::new();
    for case in cases.as_array().unwrap() {
        let Some(name) = case["op"].as_str() else {
            continue;
        };
        if nutorch_ops::find(name).unwrap().category != "loss" || !seen.insert(name) {
            continue;
        }
        // Preserve each baseline loss input and its independently generated
        // PyTorch mean oracle. Exercise sum/none through the actual Nu parser.
        let mut script = String::new();
        for (index, tensor) in case["tensors"].as_array().unwrap().iter().enumerate() {
            script.push_str(&format!(
                "let t{index} = ('{}' | from json | torch tensor --dtype {}); ",
                tensor["data"],
                tensor["dtype"].as_str().unwrap()
            ));
        }
        let mut flags = String::new();
        for (key, value) in case["params"].as_object().unwrap() {
            if key != "reduction" {
                flags.push_str(&format!(" --{key} {value}"));
            }
        }
        for mode in ["none", "sum", "mean"] {
            script.push_str(&format!(
                "let {mode} = (torch {name} $t0 $t1{flags} --reduction {mode} | torch value); "
            ));
        }
        if name == "kl_div" {
            script.push_str("let logged = (torch kl_div $t0 ($t1 | torch log) --log_target --reduction mean | torch value); ");
        } else {
            script.push_str("let logged = null; ");
        }
        script.push_str("{none: $none, sum: $sum, mean: $mean, logged: $logged} | to json --raw");
        let actual: serde_json::Value = serde_json::from_str(&shell.run(&script, true)).unwrap();
        let expected_mean = case["expect"]["values"][0].as_f64().unwrap();
        assert_eq!(
            actual["mean"].as_f64().unwrap(),
            expected_mean,
            "{name}: original mean oracle"
        );
        fn flatten(value: &serde_json::Value, out: &mut Vec<f64>) {
            if let Some(values) = value.as_array() {
                for v in values {
                    flatten(v, out);
                }
            } else {
                out.push(value.as_f64().unwrap());
            }
        }
        let mut elements = vec![];
        flatten(&actual["none"], &mut elements);
        assert!(!elements.is_empty());
        let count = if matches!(name, "cross_entropy" | "nll_loss") {
            case["tensors"][1]["data"].as_array().unwrap().len()
        } else {
            let mut target = vec![];
            flatten(&case["tensors"][1]["data"], &mut target);
            target.len()
        };
        assert_eq!(elements.len(), count, "{name}: unreduced element count");
        let sum = actual["sum"].as_f64().unwrap();
        let tolerance = 1e-6 * (1.0 + sum.abs());
        assert!(
            (sum - expected_mean * count as f64).abs() < tolerance,
            "{name}: sum vs frozen mean"
        );
        assert!(
            (elements.iter().sum::<f64>() - sum).abs() < tolerance,
            "{name}: none vs sum"
        );
        if name == "kl_div" {
            assert!((actual["logged"].as_f64().unwrap() - expected_mean).abs() < 1e-6);
        }
    }
    assert_eq!(seen.len(), 9);
}

#[test]
fn remaining_reduction_shape_and_comparison_options_have_explicit_oracles() {
    let output = Shell::new().run(r#"
        let x = torch tensor [[1 2] [3 4]]
        let boolean = torch tensor [[true false] [true true]]
        let symmetric = torch tensor [[1 3] [3 5]]
        let noisy = torch tensor [[1 NaN] [3 4]]
        let near = torch tensor [[1.1 2.1] [3.1 4.1]]
        let max_pair = $x | torch max --dim 1 --keepdim
        let min_pair = $x | torch min --dim 1 --keepdim
        let median_pair = $x | torch median --dim 1 --keepdim
        let top = $x | torch topk 1 --dim 0
        {mean: ($x | torch mean --dim 1 --keepdim | torch value),
         prod: ($x | torch prod --dim 1 --keepdim | torch value),
         amax: ($x | torch amax --dim 1 --keepdim | torch value),
         amin: ($x | torch amin --dim 1 --keepdim | torch value),
         max: ($max_pair | each {torch value $in}), min: ($min_pair | each {torch value $in}),
         median: ($median_pair | each {torch value $in}),
         argmax: ($x | torch argmax --dim 1 --keepdim | torch value),
         argmin: ($x | torch argmin --dim 1 --keepdim | torch value),
         all: ($boolean | torch all --dim 1 --keepdim | torch value),
         any: ($boolean | torch any --dim 1 --keepdim | torch value),
         std: ($symmetric | torch std --dim 1 --correction 0 --keepdim | torch value),
         var: ($symmetric | torch var --dim 1 --correction 0 --keepdim | torch value),
         nansum: ($noisy | torch nansum --dim 1 --keepdim | torch value),
         logsumexp: ((torch tensor [[0 0] [0 0]]) | torch logsumexp --dim 1 --keepdim | torch value),
         nonzero: ($boolean | torch count_nonzero --dim 1 | torch value),
         norm: ($x | torch norm --p 1 --dim 1 --keepdim | torch value),
         allclose: ($x | torch allclose $near --rtol 0.0 --atol 0.2),
         isclose: ($x | torch isclose $near --rtol 0.0 --atol 0.2 | torch value),
         topk: ($top | each {torch value $in}),
         argsort: ($x | torch argsort --dim 0 --descending | torch value),
         tril: ($x | torch tril --diagonal -1 | torch value),
         diag: ($x | torch diag --diagonal 1 | torch value),
         squeeze: ($x | torch unsqueeze 0 | torch squeeze --dim 0 | torch value),
         flatten: ($x | torch unsqueeze 0 | torch flatten --start_dim 1 --end_dim 2 | torch value),
         split: ($x | torch split 1 --dim 1 | each {torch value $in}),
         chunk: ($x | torch chunk 2 --dim 1 | each {torch value $in}),
         roll: ($x | torch roll [1] --dims [1] | torch value),
         repeat: ($x | torch repeat_interleave 2 --dim 1 | torch value),
         zeros: (torch zeros [2] --dtype int64 | torch value --meta),
         full: (torch full [2] 7 --dtype int64 | torch value --meta),
         addcdiv: ($x | torch addcdiv $x $x --value 2 | torch value),
         cross: (torch tensor [[1 0 0] [0 1 0]] | torch cross (torch tensor [[0 1 0] [0 0 1]]) --dim 1 | torch value),
         diff: ($x | torch diff --dim 0 | torch value)} | to json --raw
    "#, true);
    let actual: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = serde_json::json!({
        "mean": [[1.5],[3.5]], "prod": [[2.],[12.]], "amax": [[2.],[4.]], "amin": [[1.],[3.]],
        "max": [[[2.],[4.]],[[1],[1]]], "min": [[[1.],[3.]],[[0],[0]]], "median": [[[1.],[3.]],[[0],[0]]],
        "argmax": [[1],[1]], "argmin": [[0],[0]], "all": [[false],[true]], "any": [[true],[true]],
        "std": [[1.],[1.]], "var": [[1.],[1.]], "nansum": [[1.],[7.]], "nonzero": [1,2], "norm": [[3.],[7.]],
        "allclose": true, "isclose": [[true,true],[true,true]], "topk": [[[3.,4.]],[[1,1]]],
        "argsort": [[1,1],[0,0]], "tril": [[0.,0.],[3.,0.]], "diag": [2.], "squeeze": [[1.,2.],[3.,4.]],
        "flatten": [[1.,2.,3.,4.]], "split": [[[1.],[3.]],[[2.],[4.]]], "chunk": [[[1.],[3.]],[[2.],[4.]]],
        "roll": [[2.,1.],[4.,3.]], "repeat": [[1.,1.,2.,2.],[3.,3.,4.,4.]],
        "zeros": {"data":[0,0],"shape":[2],"dtype":"int64"}, "full": {"data":[7,7],"shape":[2],"dtype":"int64"},
        "addcdiv": [[3.,4.],[5.,6.]], "cross": [[0.,0.,1.],[1.,0.,0.]], "diff": [[2.,2.]]
    });
    for (name, value) in expected.as_object().unwrap() {
        assert_eq!(&actual[name], value, "{name}");
    }
    assert_eq!(actual["logsumexp"].as_array().unwrap().len(), 2);
    for row in actual["logsumexp"].as_array().unwrap() {
        assert_eq!(row.as_array().unwrap().len(), 1);
        assert!((row[0].as_f64().unwrap() - std::f64::consts::LN_2).abs() < 1e-6);
    }
}

#[test]
fn native_module_lists_and_retired_controls_preserve_live_values() {
    let shell = Shell::new();
    let output = shell.run(
        r#"
        let child = torch nn linear 1 1 --no-bias --weight (torch tensor [[2.0]])
        let list = [$child (torch nn relu)]
        let explicit = torch nn sequential $list
        let piped = $list | nutorch nn sequential
        let x = torch tensor [[3.0]]
        {explicit: ($x | torch forward $explicit | torch value),
         piped: ($x | nutorch forward $piped | torch value),
         child: ($x | torch forward $child | torch value),
         duplicate: (try { torch nn sequential [$child $child]; false } catch { true }),
         free: (try { torch free $x; false } catch { true }),
         still_live: ($x | torch value)} | to json --raw
    "#,
        true,
    );
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    for name in ["explicit", "piped", "child"] {
        assert_eq!(v[name], serde_json::json!([[6.0]]), "{name}");
    }
    assert_eq!(v["duplicate"], true);
    assert_eq!(v["free"], true);
    assert_eq!(v["still_live"], serde_json::json!([[3.0]]));
    for command in [
        "torch free",
        "torch tensors",
        "torch daemon status",
        "torch nu-module",
        "nutorch free",
        "nutorch tensors",
        "nutorch daemon stop",
        "nutorch nu-module",
    ] {
        let error = shell.run(command, false);
        assert!(
            error.contains("controls were removed"),
            "{command}: {error}"
        );
        assert!(
            error.contains("shared reference lifetime"),
            "{command}: {error}"
        );
    }
    assert_eq!(
        shell.run("torch --version", true).trim(),
        format!("nutorch {}", env!("CARGO_PKG_VERSION"))
    );
    let help = shell.run("torch pow --help", true);
    assert!(help.contains("usage: torch pow <t1> <exponent>"), "{help}");
}

#[test]
fn native_training_shared_modules_and_state_files() {
    let output = Shell::new().run(r#"
        cd $env.HOME
        let initial = torch tensor [[0.0]]
        let layer = torch nn linear 1 1 --no-bias --weight $initial
        let net = torch nn sequential $layer (torch nn flatten --start-dim 0)
        let optimizer = torch nn sgd $net --lr 0.05
        let held = {optimizer: $optimizer, weights: (torch nn parameters $layer)}
        let x = torch tensor [[1.0] [2.0] [3.0]]
        let target = torch tensor [2.0 4.0 6.0]
        for _ in 0..40 {
            $x | torch forward $net | torch mse_loss $target | torch backward
            torch step $held.optimizer
            torch nn zero_grad $held.optimizer
        }
        let prediction = ($x | torch forward $net | torch value)
        let saved = torch nn save $net 'trained.safetensors'
        let restored = torch nn sequential (torch nn linear 1 1 --no_bias --weight $initial) (torch nn flatten --start_dim 0)
        let restored_alias = (torch nn parameters $restored).0
        let resumed = torch nn adam $restored --lr 0.001
        let loaded = torch nn load $restored 'trained.safetensors'
        let after_load = ($x | torch forward $restored | torch value)
        $x | torch forward $restored | torch mse_loss $target | torch backward
        torch step $resumed
        torch nn zero_grad $resumed
        torch nn set_lr $resumed 0.0
        let dropout = torch nn dropout --p 0.5
        let dropout_net = torch nn sequential $dropout
        torch nn eval $dropout_net
        {prediction: $prediction, restored: $after_load, source: ($initial | torch value),
         receipts: (($saved.saved == ($env.HOME | path join 'trained.safetensors')) and ($loaded.loaded == $saved.saved)),
         weights: ($held.weights.0 | torch value), restored_type: ($restored_alias | describe),
         layer: (torch nn info $layer).kind, mode: (torch nn info $dropout).training,
         optimizer_type: ($held.optimizer | describe), lr: (torch nn info $resumed).lr} | to json --raw
    "#, true);
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    for (actual, expected) in v["prediction"].as_array().unwrap().iter().zip([2., 4., 6.]) {
        assert!((actual.as_f64().unwrap() - expected).abs() < 0.001);
    }
    assert_eq!(v["prediction"], v["restored"]);
    assert_eq!(v["receipts"], true);
    assert_eq!(v["source"], serde_json::json!([[0.0]]));
    assert_eq!(v["restored_type"], "tensor");
    assert_eq!(v["optimizer_type"], "optimizer");
    assert_eq!(v["layer"], "linear");
    assert_eq!(v["mode"], "false");
    assert_eq!(v["lr"], "0");
}

#[test]
fn tensor_metadata_roundtrips_integer_boolean_and_nonfinite_data() {
    let output = Shell::new().run(r#"
        let int = torch tensor [9007199254740993 -7] --dtype int64
        let boolean = torch tensor [true false]
        let nonfinite = torch tensor [NaN inf -inf]
        let round = ($int | torch value --meta | torch tensor)
        {integers: ($round | torch value), booleans: ($boolean | torch value --meta | torch tensor | torch value),
         shape: ($round | torch shape), dtype: ($round | torch value --meta).dtype,
         finite: ($nonfinite | torch value --meta | torch tensor | torch isfinite | torch value),
         large_literal_kind: (18446744073709551615 | describe),
         large_literal: (torch tensor [18446744073709551615] | torch value),
         ops: (torch ops | length)} | to json --raw
    "#, true);
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(v["integers"], serde_json::json!([9007199254740993i64, -7]));
    assert_eq!(v["booleans"], serde_json::json!([true, false]));
    assert_eq!(v["finite"], serde_json::json!([false, false, false]));
    assert_eq!(v["dtype"], "int64");
    assert_eq!(v["shape"], serde_json::json!([2]));
    assert_eq!(v["large_literal_kind"], "float");
    assert_eq!(
        v["large_literal"],
        serde_json::json!([u64::MAX as f32 as f64])
    );
    assert_eq!(v["ops"], 185);
}

#[test]
fn native_variadic_multi_output_and_compatibility_aliases() {
    let output = Shell::new().run(
        r#"
        let a = nutorch tensor [3.0 1.0]
        let b = torch tensor [2.0 4.0]
        let sorted = ($a | nutorch sort)
        let one_chunk = ($a | torch chunk 1)
        let lo = torch tensor [2.0 2.0]
        {cat: ([$a $b] | torch cat | torch value),
         stacked: (torch stack [$a $b] | torch value),
         added: ([$a $b] | torch add | torch value),
         sorted: ($sorted | each {torch value $in}),
         one_chunk_type: ($one_chunk | describe), one_chunk: ($one_chunk.0 | torch value),
         clamped: ($a | torch clamp --min $lo | torch value),
         powers: ($a | torch pow $b | torch value),
         max: ($a | torch max | each {torch value $in})} | to json --raw
    "#,
        true,
    );
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(v["cat"], serde_json::json!([3., 1., 2., 4.]));
    assert_eq!(v["stacked"], serde_json::json!([[3., 1.], [2., 4.]]));
    assert_eq!(v["added"], serde_json::json!([5., 5.]));
    assert_eq!(v["sorted"], serde_json::json!([[1., 3.], [1, 0]]));
    assert!(v["one_chunk_type"].as_str().unwrap().starts_with("list"));
    assert_eq!(v["one_chunk"], serde_json::json!([3., 1.]));
    assert_eq!(v["clamped"], serde_json::json!([3., 2.]));
    assert_eq!(v["powers"], serde_json::json!([9., 1.]));
    assert_eq!(v["max"], serde_json::json!([3.]));
}

#[test]
fn native_command_adapter_matches_every_operation_fixture() {
    use nutorch_ops::{ParamKind, ResultKind};
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../core/tests/golden.json")).unwrap();
    let literal = |v: &serde_json::Value| {
        format!(
            "({} | from json)",
            serde_json::to_string(&v.to_string()).unwrap()
        )
    };
    let shell = Shell::new();
    let mut count = 0;
    for case in fixtures.iter().filter(|c| !c["op"].is_null()) {
        count += 1;
        let spec = nutorch_ops::find(case["op"].as_str().unwrap()).unwrap();
        let mut script = String::new();
        if let Some(seed) = case["seed"].as_i64() {
            script.push_str(&format!("torch manual_seed {seed}; "));
        }
        for (index, input) in case["tensors"].as_array().unwrap().iter().enumerate() {
            script.push_str(&format!(
                "let t{index} = torch tensor {} --dtype {}; ",
                literal(&input["data"]),
                literal(&input["dtype"])
            ));
        }
        let params = case["params"].as_object().cloned().unwrap_or_default();
        let mut refs = vec![];
        let mut param_values = std::collections::BTreeMap::new();
        for (name, val) in &params {
            let expression = if let Some(index) = val
                .as_str()
                .and_then(|s| s.strip_prefix('T'))
                .and_then(|s| s.parse::<usize>().ok())
            {
                refs.push(index);
                format!("$t{index}")
            } else {
                literal(val)
            };
            param_values.insert(name.as_str(), expression);
        }
        let mut invocation = format!("torch {}", spec.name);
        for index in 0..case["tensors"].as_array().unwrap().len() {
            if !refs.contains(&index) {
                invocation.push_str(&format!(" $t{index}"));
            }
        }
        for param in spec.params {
            if let Some(expression) = param_values.get(param.name) {
                if param.positional {
                    invocation.push_str(&format!(" {expression}"));
                } else if param.kind == ParamKind::Bool {
                    if params[param.name].as_bool().unwrap() {
                        invocation.push_str(&format!(" --{}", param.name));
                    }
                } else {
                    invocation.push_str(&format!(" --{} {expression}", param.name));
                }
            }
        }
        script.push_str(&format!("let result = ({invocation}); "));
        let expect = &case["expect"];
        if !expect["error"].is_null() {
            shell.run(&script, false);
            continue;
        }
        script.push_str(match spec.results {
            ResultKind::Tensors(1) => "[($result | torch value)] | to json --raw",
            ResultKind::Tensors(_) | ResultKind::VariableTensors => {
                "$result | each { torch value $in } | to json --raw"
            }
            ResultKind::Value => "$result | to json --raw",
            ResultKind::None => "[] | to json --raw",
        });
        let actual: serde_json::Value = serde_json::from_str(&shell.run(&script, true)).unwrap();
        let expected = if spec.results == ResultKind::Value {
            &expect["value"]
        } else {
            &expect["values"]
        };
        assert_eq!(&actual, expected, "{}: {script}", case["name"]);
    }
    assert_eq!(count, 218);
}

#[test]
fn module_randomness_and_validation_do_not_poison_native_execution() {
    let output = Shell::new().run(r#"
        torch manual_seed 11
        let first = (torch nn linear 3 2 | torch nn parameters | each {torch value $in})
        torch manual_seed 11
        let second = (torch nn linear 3 2 | torch nn parameters | each {torch value $in})
        let d = torch nn dropout --p 0.25
        let x = torch ones [1000]
        torch manual_seed 5
        let mask1 = ($x | torch forward $d | torch value)
        torch manual_seed 5
        let mask2 = ($x | torch forward $d | torch value)
        let wrong_weight = try { torch nn linear 1 1 --weight (torch tensor [[1]] --dtype int64); false } catch {true}
        let wrong_shape = try {torch allclose (torch ones [2]) (torch ones [3]); false} catch {true}
        let bad_envelope = try {torch tensor {dtype: float32, shape: [99], data: [1 2]}; false} catch {true}
        let conflicting_dtype = try {torch tensor {dtype: float32, data: [1 2]} --dtype int64; false} catch {true}
        {seeded: ($first == $second), masks: ($mask1 == $mask2), wrong_weight: $wrong_weight,
         wrong_shape: $wrong_shape, bad_envelope: $bad_envelope, conflicting_dtype: $conflicting_dtype,
         recovered: (torch tensor [1 2] | torch mul (torch tensor [2 3]) | torch value)} | to json --raw
    "#,true);
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    for name in [
        "seeded",
        "masks",
        "wrong_weight",
        "wrong_shape",
        "bad_envelope",
        "conflicting_dtype",
    ] {
        assert_eq!(v[name], true, "{name}");
    }
    assert_eq!(v["recovered"], serde_json::json!([2., 6.]));
}
impl Shell {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for name in ["torch", "nutorchd"] {
                let path = dir.path().join(name);
                fs::write(&path,format!("#!{} --no-config-file\n'called' | save --force $env.NUTORCH_TEST_SENTINEL\nexit 98\n",env!("CARGO_BIN_EXE_nutorch"))).unwrap();
                fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        Self { dir }
    }
    fn run(&self, script: &str, success: bool) -> String {
        self.run_raw(&format!("use torch; use torch nutorch; {script}"), success)
    }
    fn run_raw(&self, script: &str, success: bool) -> String {
        self.invoke(&["--no-config-file", "--no-std-lib", "-c", script], success)
    }
    fn invoke(&self, args: &[&str], success: bool) -> String {
        let out = self.dir.path().join("stdout");
        let err = self.dir.path().join("stderr");
        let mut child = Command::new(env!("CARGO_BIN_EXE_nutorch"))
            .arg("--no-history")
            .args(args)
            .current_dir(self.dir.path())
            .env("HOME", self.dir.path())
            .env("XDG_CONFIG_HOME", self.dir.path())
            .env("PATH", self.dir.path())
            .env("NUTORCH_TEST_SENTINEL", self.dir.path().join("called"))
            .stdin(Stdio::null())
            .stdout(fs::File::create(&out).unwrap())
            .stderr(fs::File::create(&err).unwrap())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() > deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("native shell command timed out: {args:?}");
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let stdout = fs::read_to_string(out).unwrap();
        let stderr = fs::read_to_string(err).unwrap();
        assert_eq!(
            status.success(),
            success,
            "{args:?}\nstdout: {stdout}\nstderr: {stderr}"
        );
        assert!(
            !self.dir.path().join("called").exists(),
            "native path executed a legacy program"
        );
        if success { stdout } else { stderr }
    }
}

#[test]
fn native_values_survive_shell_containers_functions_and_autograd() {
    let output = Shell::new().run(r#"
        def identity [v] { $v }
        let x = torch tensor [1 2 3] --requires_grad
        let alias = (identity {nested: [$x]}).nested.0
        let loss = (do { let middle = ($x | torch mul $alias); $middle | torch sum })
        $loss | torch backward
        let snapshot = ($alias | torch grad)
        $x | torch mul $x | torch sum | torch backward
        let accumulated = ($x | torch grad | torch tolist)
        torch zero_grad $alias
        {kind: ($alias | describe), shape: (torch shape $x), data: ($x | torch tolist), grad: $accumulated,
         cleared: (torch grad $x | torch tolist), snapshot: ($snapshot | torch tolist),
         added: (torch add $x $alias --alpha 2 | torch tolist)} | to json --raw
    "#,true);
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(v["kind"], "tensor");
    assert_eq!(v["shape"], serde_json::json!([3]));
    assert_eq!(v["data"], serde_json::json!([1., 2., 3.]));
    assert_eq!(v["grad"], serde_json::json!([4., 8., 12.]));
    assert_eq!(v["cleared"], serde_json::json!([0., 0., 0.]));
    assert_eq!(v["snapshot"], serde_json::json!([2., 4., 6.]));
    assert_eq!(v["added"], serde_json::json!([3., 6., 9.]));
}

#[test]
fn invalid_inputs_and_unsupported_commands_do_not_fall_back() {
    let shell = Shell::new();
    for script in [
        "torch shape 'tensor://old'",
        "torch tensor [1 2] --dtype invalid",
        "torch tensor [1 2] --dtype int64 --requires_grad",
        "torch add (torch tensor [1 2]) (torch tensor [1 2 3])",
        "torch tensor [1 2] | torch sum (torch tensor [1])",
        "torch tensor [1 2] | torch backward",
        "torch tensor [1 2] | torch grad",
        "torch tensor [[1 2] [3]]",
        "torch tensor []",
        "torch tensor hello",
        "torch tensor [1 two]",
        "torch tensor [NaN] --dtype int64",
        "torch tensor [true 2]",
        "torch tensor [1 2] --device cpu",
        "torch randn [2] --dtype int64",
        "torch tensor [1 2] | torch clamp",
        "torch nn linear 1 1 | torch forward (torch tensor [1])",
        "torch nn relu | torch value",
        "torch nn relu | update kind linear",
        "torch nn adam (torch nn linear 1 1) | update lr 0.5",
        "torch nn linear 2 3 --no-bias --bias-tensor (torch tensor [1 2 3])",
        "torch tensor [1 2] | torch mse_loss (torch tensor [1 2]) --reduction invalid",
        "torch nn linear",
        "torch mul (torch tensor [1]) (torch tensor [1]) (torch tensor [1])",
        "torch tensor [1 2] | update 0 9",
    ] {
        shell.run(script, false);
    }
    assert_eq!(shell.run("torch tensor [1 2] | torch mul (torch tensor [2 3]) | torch tolist | to json --raw",true).trim(),"[2.0,6.0]");
}

#[test]
fn metadata_is_bounded_and_generic_json_is_only_a_summary() {
    let shell = Shell::new();
    let out = shell.run("let x = (torch tensor (1..100000 | each {|i| $i})); print $x; {nested: [$x]} | to json --raw",true);
    assert!(out.len() < 500, "unexpected tensor data materialization");
    assert!(out.contains("shape=[100000]"));
    assert!(out.contains("device=mps"));
    assert!(!out.contains("tensor://"));
    shell.run(
        "torch tensor [1 2] | to json | from json | torch shape",
        false,
    );
}

#[test]
fn parallel_nushell_tasks_share_native_tensors() {
    let out = Shell::new().run(r#"
        let x = torch tensor [1 2 3]
        1..8 | par-each {|i| $x | torch mul $x | torch sum | torch tolist } | math sum | to json --raw
    "#,true);
    assert_eq!(out.trim(), "112.0");
}

#[test]
fn dtype_reduction_and_detach_contracts() {
    let out = Shell::new().run(
        r#"
        let x = torch tensor [[1 2] [3 4]] --requires-grad
        {reduced: (torch sum $x --dim 1 --keepdim | torch tolist),
         detached: (torch detach $x | to json), tracked: ($x | to json),
         ints: (torch tensor [9007199254740993] --dtype int64 | torch tolist),
         bools: (torch tensor [true false] | torch tolist)} | to json --raw
    "#,
        true,
    );
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["reduced"], serde_json::json!([[3.], [7.]]));
    assert!(
        v["detached"]
            .as_str()
            .unwrap()
            .contains("requires_grad=false")
    );
    assert!(
        v["tracked"]
            .as_str()
            .unwrap()
            .contains("requires_grad=true")
    );
    assert_eq!(v["ints"], serde_json::json!([9007199254740993i64]));
    assert_eq!(v["bools"], serde_json::json!([true, false]));
}
