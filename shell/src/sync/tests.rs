use super::*;

fn fixture(source: &str) -> (EngineState, Stack) {
    let mut engine =
        nu_command::add_shell_command_context(nu_cmd_lang::add_default_context(EngineState::new()));
    let mut stack = Stack::new();
    let mut working = StateWorkingSet::new(&engine);
    let block = nu_parser::parse(
        &mut working,
        Some("codec-fixture.nu"),
        source.as_bytes(),
        false,
    );
    assert!(
        working.parse_errors.is_empty(),
        "{:?}",
        working.parse_errors
    );
    engine.merge_delta(working.render()).unwrap();
    nu_engine::eval_block::<nu_protocol::debugger::WithoutDebug>(
        &engine,
        &mut stack,
        &block,
        PipelineData::empty(),
    )
    .unwrap();
    (engine, stack)
}

fn update(
    engine: &EngineState,
    stack: &mut Stack,
    entries: &[(&str, &str)],
) -> transport::Result<()> {
    apply(
        engine,
        stack,
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

#[test]
fn codecs_roundtrip_capture_order_and_upstream_parity() {
    let (engine, mut stack) = fixture(
        r#"
        let suffix = 'captured'
        $env.LIST = [old]
        $env.REC = {old: true}
        $env.ENV_CONVERSIONS = {
            LIST: {to_string: {|v| $v | to json --raw}, from_string: {|v| $v | from json}},
            REC: {to_string: {|v| $v | to json --raw}, from_string: {|v| $v | from json}},
            FIRST: {from_string: {|v| $v + ':' + $env.SECOND + ':' + $suffix}},
            SECOND: {from_string: {|v| $env.FIRST + ':' + $v}}
        }
    "#,
    );
    let entries = [
        ("LIST", r#"["☃","","line\n=literal"]"#),
        ("REC", r#"{"x":[1,true]}"#),
        ("SECOND", "two"),
        ("FIRST", "one"),
    ];
    let mut oracle = stack.clone();
    for (name, value) in entries {
        oracle.add_env_var(name.into(), Value::string(value, Span::unknown()));
    }
    let config = stack.get_env_var(&engine, "ENV_CONVERSIONS").unwrap();
    convert_env_vars(&mut oracle, &engine, &config).unwrap();
    update(&engine, &mut stack, &entries).unwrap();
    for (name, _) in entries {
        assert_eq!(
            stack.get_env_var(&engine, name),
            oracle.get_env_var(&engine, name)
        );
    }
    assert_eq!(
        stack
            .get_env_var(&engine, "LIST")
            .unwrap()
            .as_list()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        stack
            .get_env_var(&engine, "REC")
            .unwrap()
            .as_record()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        stack
            .get_env_var(&engine, "FIRST")
            .unwrap()
            .as_str()
            .unwrap(),
        "one:two:captured"
    );
    assert_eq!(
        stack
            .get_env_var(&engine, "SECOND")
            .unwrap()
            .as_str()
            .unwrap(),
        "one:two:captured:two"
    );
    assert!(
        exported(&engine, &stack)
            .unwrap()
            .contains(&("LIST".into(), r#"["☃","","line\n=literal"]"#.into()))
    );
}

#[test]
fn unchanged_values_skip_decoders_and_unrelated_malformed_codecs() {
    let (engine, mut stack) = fixture(
        r#"
        $env.LIST = [a b]; $env.REC = {a: 1}; $env.TEXT = 'same'; $env.PATH = ['/bin']
        $env.ENV_CONVERSIONS = {
            LIST: {to_string: {|v| $v | to json --raw}, from_string: {|v| error make {msg: 'decoder ran'}}},
            REC: {to_string: {|v| $v | to json --raw}, from_string: {|v| error make {msg: 'decoder ran'}}},
            TEXT: {from_string: {|v| if $v == 'same' { $v } else { error make {msg: 'decoder ran'} }}},
            PATH: {to_string: {|v| $v | str join ':'}, from_string: {|v| error make {msg: 'decoder ran'}}}
        }
    "#,
    );
    // Set a throwing TEXT decoder after initialization without invoking assignment conversion.
    let mut config = stack
        .get_env_var(&engine, "ENV_CONVERSIONS")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();
    let throwing = config
        .get("LIST")
        .unwrap()
        .as_record()
        .unwrap()
        .get("from_string")
        .unwrap()
        .clone();
    let mut text_codec = Record::new();
    text_codec.push("from_string", throwing);
    config.insert("TEXT", Value::record(text_codec, Span::unknown()));
    // Neither alias may run for an unchanged typed value; untouched aliases
    // must likewise stay outside the filtered conversion record.
    config.push("List", config.get("LIST").unwrap().clone());
    config.push("Rec", config.get("REC").unwrap().clone());
    config.push("UNRELATED", Value::int(17, Span::unknown()));
    config.push("Unrelated", Value::int(17, Span::unknown()));
    stack.add_env_var(
        "ENV_CONVERSIONS".into(),
        Value::record(config, Span::unknown()),
    );
    let before = stack.get_env_vars(&engine);
    update(
        &engine,
        &mut stack,
        &[
            ("LIST", r#"["a","b"]"#),
            ("REC", r#"{"a":1}"#),
            ("PATH", "/bin"),
            ("TEXT", "same"),
            ("NEW", "value"),
        ],
    )
    .unwrap();
    for name in ["LIST", "REC", "PATH", "TEXT"] {
        assert_eq!(stack.get_env_var(&engine, name).unwrap(), &before[name]);
    }
    assert_eq!(
        stack.get_env_var(&engine, "NEW").unwrap().as_str().unwrap(),
        "value"
    );
}

#[test]
fn cancellation_rejects_even_plain_updates_and_recovers_after_reset() {
    let mut engine = EngineState::new();
    engine.set_signals(nu_protocol::Signals::new(Arc::new(AtomicBool::new(true))));
    let mut stack = Stack::new();
    stack.add_env_var("SAFE".into(), Value::string("old", Span::unknown()));
    assert!(update(&engine, &mut stack, &[("SAFE", "new"), ("NEW", "x")]).is_err());
    assert_eq!(
        stack
            .get_env_var(&engine, "SAFE")
            .unwrap()
            .as_str()
            .unwrap(),
        "old"
    );
    assert!(stack.get_env_var(&engine, "NEW").is_none());
    engine.reset_signals();
    update(&engine, &mut stack, &[("SAFE", "recovered")]).unwrap();
}

#[test]
fn error_valued_decoder_output_is_not_committed() {
    let (engine, mut stack) = fixture(
        "$env.SAFE = 'old'; $env.ENV_CONVERSIONS = {BAD: {from_string: {|v| $env.RETURN_ERROR}}}",
    );
    stack.add_env_var(
        "RETURN_ERROR".into(),
        Value::error(
            ShellError::Generic(nu_protocol::shell_error::generic::GenericError::new(
                "secret",
                "secret payload",
                Span::unknown(),
            )),
            Span::unknown(),
        ),
    );
    let result = update(&engine, &mut stack, &[("SAFE", "new"), ("BAD", "payload")]);
    assert!(result.is_err());
    assert!(!result.unwrap_err().contains("secret"));
    assert_eq!(
        stack
            .get_env_var(&engine, "SAFE")
            .unwrap()
            .as_str()
            .unwrap(),
        "old"
    );
    assert!(stack.get_env_var(&engine, "BAD").is_none());
}

#[test]
fn unchanged_native_value_keeps_shared_identity_without_decoding() {
    let (engine, mut stack) = fixture(
        "$env.ENV_CONVERSIONS = {NATIVE: {to_string: {|v| 'native'}, from_string: {|v| error make {msg: 'must not decode'}}}}",
    );
    let tensor =
        nutorch_core::SharedTensor::create(&nutorch_core::Data::Int(1), None, false).unwrap();
    stack.add_env_var(
        "NATIVE".into(),
        Value::custom(
            Box::new(crate::tensor::TensorValue(tensor)),
            Span::unknown(),
        ),
    );
    // The box address is an identity oracle: touching this binding would clone it.
    let address = stack
        .get_env_var(&engine, "NATIVE")
        .unwrap()
        .as_custom_value()
        .unwrap()
        .as_any() as *const dyn std::any::Any as *const () as usize;
    update(&engine, &mut stack, &[("NATIVE", "native")]).unwrap();
    let retained = stack
        .get_env_var(&engine, "NATIVE")
        .unwrap()
        .as_custom_value()
        .unwrap()
        .as_any() as *const dyn std::any::Any as *const () as usize;
    assert_eq!(address, retained);
}

#[test]
fn aliases_preserve_upstream_order_and_skip_non_strings() {
    for (first, second) in [("CASE", "case"), ("case", "CASE")] {
        for result in ["[$v]", "{value: $v}"] {
            for later in ["17", "{from_string: {|v| error make {msg: 'must skip'}}}"] {
                let (engine, mut stack) = fixture(&format!(
                    "$env.ENV_CONVERSIONS = {{{first}: {{from_string: {{|v| {result}}}}}, {second}: {later}}}"
                ));
                let mut oracle = stack.clone();
                oracle.add_env_var("Case".into(), Value::string("input", Span::unknown()));
                convert_env_vars(
                    &mut oracle,
                    &engine,
                    stack.get_env_var(&engine, "ENV_CONVERSIONS").unwrap(),
                )
                .unwrap();
                update(&engine, &mut stack, &[("Case", "input")]).unwrap();
                let actual = stack.get_env_var(&engine, "Case").unwrap();
                assert_eq!(Some(actual), oracle.get_env_var(&engine, "Case"));
                let inner = if result == "[$v]" {
                    &actual.as_list().unwrap()[0]
                } else {
                    actual.as_record().unwrap().get("value").unwrap()
                };
                assert_eq!(inner.as_str().unwrap(), "input");
            }
        }
    }
}

#[test]
fn aliases_chain_strings_in_record_order_with_upstream_parity() {
    for (keys, expected) in [(["CASE", "case"], "inputAB"), (["case", "CASE"], "inputBA")] {
        let entries = keys.map(|key| {
            format!(
                "{key}: {{from_string: {{|v| $v + '{}'}}}}",
                if key == "CASE" { "A" } else { "B" }
            )
        });
        let (engine, mut stack) = fixture(&format!(
            "$env.ENV_CONVERSIONS = {{{}}}",
            entries.join(", ")
        ));
        stack.add_env_var("Case".into(), Value::string("old", Span::unknown()));
        let mut oracle = stack.clone();
        oracle.add_env_var("Case".into(), Value::string("input", Span::unknown()));
        convert_env_vars(
            &mut oracle,
            &engine,
            stack.get_env_var(&engine, "ENV_CONVERSIONS").unwrap(),
        )
        .unwrap();
        update(&engine, &mut stack, &[("case", "input")]).unwrap();
        assert_eq!(
            stack.get_env_var(&engine, "Case"),
            oracle.get_env_var(&engine, "Case")
        );
        assert_eq!(
            stack
                .get_env_var(&engine, "Case")
                .unwrap()
                .as_str()
                .unwrap(),
            expected
        );
        assert!(stack.get_env_vars(&engine).contains_key("Case"));
    }
    let (engine, mut stack) = fixture(
        "$env.ENV_CONVERSIONS = {Path: {from_string: {|v| $v + ':/second'}}, PATH: {from_string: {|v| $v | split row ':'}}}",
    );
    let mut oracle = stack.clone();
    oracle.add_env_var("PATH".into(), Value::string("/first", Span::unknown()));
    convert_env_vars(
        &mut oracle,
        &engine,
        stack.get_env_var(&engine, "ENV_CONVERSIONS").unwrap(),
    )
    .unwrap();
    update(&engine, &mut stack, &[("PATH", "/first")]).unwrap();
    assert_eq!(
        stack.get_env_var(&engine, "PATH"),
        oracle.get_env_var(&engine, "PATH")
    );
    let paths = stack
        .get_env_var(&engine, "PATH")
        .unwrap()
        .as_list()
        .unwrap();
    assert_eq!(
        paths
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        ["/first", "/second"]
    );
}

#[test]
fn reached_alias_failure_is_atomic_and_recovers() {
    for later in [
        "17",
        "{from_string: {|v| error make {msg: 'secret-value'}}}",
    ] {
        let (engine, mut stack) = fixture(&format!(
            "$env.SAFE = 'old'; $env.ENV_CONVERSIONS = {{CASE: {{from_string: {{|v| $v + 'first'}}}}, case: {later}}}"
        ));
        let error = update(
            &engine,
            &mut stack,
            &[("SAFE", "new"), ("Case", "secret-value")],
        )
        .unwrap_err();
        assert!(!error.contains("secret-value"));
        assert_eq!(
            stack
                .get_env_var(&engine, "SAFE")
                .unwrap()
                .as_str()
                .unwrap(),
            "old"
        );
        assert!(stack.get_env_var(&engine, "Case").is_none());
        update(&engine, &mut stack, &[("SAFE", "recovered")]).unwrap();
    }
}

#[test]
fn malformed_and_throwing_codecs_reject_whole_batch() {
    for decoder in [
        "17",
        "{}",
        "{from_string: 17}",
        "{from_string: {|v| error make {msg: 'secret-value'}}}",
    ] {
        let (engine, mut stack) = fixture(&format!(
            "$env.SAFE = 'old'; $env.ENV_CONVERSIONS = {{BAD: {decoder}}}"
        ));
        let result = update(
            &engine,
            &mut stack,
            &[("SAFE", "new"), ("BAD", "secret-value")],
        );
        assert!(result.is_err());
        assert!(!result.unwrap_err().contains("secret-value"));
        assert_eq!(
            stack
                .get_env_var(&engine, "SAFE")
                .unwrap()
                .as_str()
                .unwrap(),
            "old"
        );
        assert!(stack.get_env_var(&engine, "BAD").is_none());
        update(&engine, &mut stack, &[("SAFE", "recovered")]).unwrap();
    }
    let (engine, mut stack) = fixture(
        "$env.ENV_CONVERSIONS = {CASE: {from_string: {|v| $v}}, case: {from_string: {|v| $v}}}",
    );
    update(&engine, &mut stack, &[("Case", "x")]).unwrap();
    stack.add_env_var("ENV_CONVERSIONS".into(), Value::int(1, Span::unknown()));
    assert!(update(&engine, &mut stack, &[("NEW", "x")]).is_err());
}

#[test]
fn path_codecs_follow_upstream_validation_and_default_fallback() {
    for decoder in [
        "{|v| ['/custom' '']}",
        "{|v| '/custom::'}",
        "{|v| [1]}",
        "{|v| 17}",
    ] {
        let (engine, mut stack) = fixture(&format!(
            "$env.PATH = ['/old']; $env.SAFE = 'old'; $env.ENV_CONVERSIONS = {{PATH: {{to_string: {{|v| $v | str join ':'}}, from_string: {decoder}}}}}"
        ));
        let mut oracle = stack.clone();
        oracle.add_env_var("PATH".into(), Value::string("/incoming", Span::unknown()));
        let expected = convert_env_vars(
            &mut oracle,
            &engine,
            &stack.get_env_var(&engine, "ENV_CONVERSIONS").unwrap(),
        );
        let actual = update(
            &engine,
            &mut stack,
            &[("PATH", "/incoming"), ("SAFE", "new")],
        );
        assert_eq!(actual.is_ok(), expected.is_ok());
        if actual.is_ok() {
            assert_eq!(
                stack.get_env_var(&engine, "PATH"),
                oracle.get_env_var(&engine, "PATH")
            );
        } else {
            assert_eq!(
                stack
                    .get_env_var(&engine, "SAFE")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                "old"
            );
        }
    }
    let engine = EngineState::new();
    let mut stack = Stack::new();
    stack.add_env_var("PATH".into(), Value::int(1, Span::unknown()));
    assert!(update(&engine, &mut stack, &[("NEW", "x")]).is_err());
    assert!(stack.get_env_var(&engine, "NEW").is_none());
}

#[test]
fn codec_case_alias_and_local_side_effects_do_not_replace_live_state() {
    let (engine, mut stack) = fixture(
        r#"
        $env.UNRELATED = 'original'
        $env.ENV_CONVERSIONS = {case: {from_string: {|v| $env.UNRELATED = 'leak'; $env.NUTORCH_SYNC_TOKEN = 'leak'; cd /tmp; $env.config.show_banner = false; $env.LAST_EXIT_CODE = 99; [$v]}}}
    "#,
    );
    stack.add_env_var("CASE".into(), Value::string("old", Span::unknown()));
    stack.add_env_var("PWD".into(), Value::string("/", Span::unknown()));
    stack.add_env_var("LAST_EXIT_CODE".into(), Value::int(7, Span::unknown()));
    let banner = stack.get_config(&engine).show_banner;
    update(
        &engine,
        &mut stack,
        &[
            ("Case", "new"),
            ("env_conversions", "attacker"),
            ("nutorch_sync_token", "attacker"),
        ],
    )
    .unwrap();
    assert!(stack.get_env_vars(&engine).contains_key("CASE"));
    assert_eq!(
        stack
            .get_env_var(&engine, "CASE")
            .unwrap()
            .as_list()
            .unwrap()[0]
            .as_str()
            .unwrap(),
        "new"
    );
    assert_eq!(
        stack
            .get_env_var(&engine, "UNRELATED")
            .unwrap()
            .as_str()
            .unwrap(),
        "original"
    );
    assert!(stack.get_env_var(&engine, "NUTORCH_SYNC_TOKEN").is_none());
    assert_eq!(
        stack.get_env_var(&engine, "PWD").unwrap().as_str().unwrap(),
        "/"
    );
    assert_eq!(
        stack
            .get_env_var(&engine, "LAST_EXIT_CODE")
            .unwrap()
            .as_int()
            .unwrap(),
        7
    );
    assert_eq!(stack.get_config(&engine).show_banner, banner);
}

#[test]
fn merge_is_typed_atomic_and_non_destructive() {
    let engine = EngineState::new();
    let mut stack = Stack::new();
    stack.add_env_var("KEEP".into(), Value::int(42, Span::unknown()));
    stack.add_env_var("ABSENT".into(), Value::string("retained", Span::unknown()));
    stack.add_env_var("PWD".into(), Value::string("/original", Span::unknown()));
    apply(
        &engine,
        &mut stack,
        vec![
            ("KEEP".into(), "42".into()),
            ("NEW".into(), "☃\n=$(no code)".into()),
            ("EMPTY".into(), "".into()),
            ("PATH".into(), ":/bin::/usr/bin:".into()),
            ("PWD".into(), "/changed".into()),
            ("TERMSURF_SOCKET".into(), "bad".into()),
        ],
    )
    .unwrap();
    assert_eq!(
        stack
            .get_env_var(&engine, "KEEP")
            .unwrap()
            .as_int()
            .unwrap(),
        42
    );
    assert_eq!(
        stack
            .get_env_var(&engine, "ABSENT")
            .unwrap()
            .as_str()
            .unwrap(),
        "retained"
    );
    assert_eq!(
        stack.get_env_var(&engine, "NEW").unwrap().as_str().unwrap(),
        "☃\n=$(no code)"
    );
    assert_eq!(
        stack
            .get_env_var(&engine, "EMPTY")
            .unwrap()
            .as_str()
            .unwrap(),
        ""
    );
    assert_eq!(
        stack.get_env_var(&engine, "PWD").unwrap().as_str().unwrap(),
        "/original"
    );
    assert!(stack.get_env_var(&engine, "TERMSURF_SOCKET").is_none());
    let path = stack.get_env_var(&engine, "PATH").unwrap();
    let path: Vec<_> = path
        .as_list()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(path, ["", "/bin", "", "/usr/bin", ""]);
    stack.add_env_var(
        "PRIVATE".into(),
        Value::record(Record::new(), Span::unknown()),
    );
    assert!(
        apply(
            &engine,
            &mut stack,
            vec![
                ("NEW".into(), "lost".into()),
                ("PRIVATE".into(), "bad".into())
            ]
        )
        .is_err()
    );
    assert_eq!(
        stack.get_env_var(&engine, "NEW").unwrap().as_str().unwrap(),
        "☃\n=$(no code)"
    );
    assert!(matches!(
        stack.get_env_var(&engine, "PRIVATE").unwrap(),
        Value::Record { .. }
    ));
    apply(&engine, &mut stack, vec![("KEEP".into(), "43".into())]).unwrap();
    assert_eq!(
        stack
            .get_env_var(&engine, "KEEP")
            .unwrap()
            .as_str()
            .unwrap(),
        "43"
    );
    apply(&engine, &mut stack, vec![("KEEP".into(), "44".into())]).unwrap();
    assert_eq!(
        stack
            .get_env_var(&engine, "KEEP")
            .unwrap()
            .as_str()
            .unwrap(),
        "44"
    );
}

#[test]
fn reserved_names_are_filtered_before_export_conversion() {
    let engine = EngineState::new();
    let mut stack = Stack::new();
    stack.add_env_var(
        "PROMPT_COMMAND".into(),
        Value::record(Record::new(), Span::unknown()),
    );
    stack.add_env_var(
        "PRIVATE".into(),
        Value::record(Record::new(), Span::unknown()),
    );
    stack.add_env_var("PUBLIC".into(), Value::int(7, Span::unknown()));
    assert_eq!(
        exported(&engine, &stack).unwrap(),
        vec![("PUBLIC".into(), "7".into())]
    );
}

#[test]
fn protection_and_duplicates_match_nushell_case_insensitive_lookup() {
    for name in [
        "pwd",
        "Config",
        "nuTorch_sync_token",
        "termsurf_SOCKET",
        "Prompt_COMMAND",
    ] {
        assert!(transport::reserved(name));
    }
    assert!(
        transport::validate_entries(&[("VAR".into(), "1".into()), ("var".into(), "2".into())])
            .is_err()
    );
    let engine = EngineState::new();
    let mut stack = Stack::new();
    stack.add_env_var(
        "PATH".into(),
        Value::list(
            vec![Value::string("/old", Span::unknown())],
            Span::unknown(),
        ),
    );
    stack.add_env_var("PWD".into(), Value::string("/original", Span::unknown()));
    apply(
        &engine,
        &mut stack,
        vec![
            ("path".into(), "/new:".into()),
            ("pwd".into(), "/bad".into()),
        ],
    )
    .unwrap();
    assert_eq!(
        stack
            .get_env_var(&engine, "PATH")
            .unwrap()
            .as_list()
            .unwrap()
            .len(),
        2
    );
    assert!(stack.get_env_vars(&engine).contains_key("PATH"));
    assert_eq!(
        stack.get_env_var(&engine, "PWD").unwrap().as_str().unwrap(),
        "/original"
    );
}
