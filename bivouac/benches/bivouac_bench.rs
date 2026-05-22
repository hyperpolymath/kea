// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Benchmarks for Kea-Bivouac.
//!
//! These benchmarks measure performance of core operations like parsing and validation.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kea_bivouac::playbook::{Playbook, PlaybookTrigger, PlaybookAction};
use kea_bivouac::Config;
use std::path::PathBuf;

fn bench_playbook_parse(c: &mut Criterion) {
    let toml_content = r#"name = "benchmark-playbook"
description = "A benchmark playbook"
continue_on_error = false
timeout_secs = 300

[trigger]
type = "schedule"
cron = "0 0 * * *"

[[actions]]
type = "log"
level = "info"
message = "Benchmark action 1"

[[actions]]
type = "wait"
duration_secs = 10

[[actions]]
type = "log"
level = "info"
message = "Benchmark action 2"
"#;

    c.bench_function("playbook_parse_simple", |b| {
        b.iter(|| {
            let path = std::path::Path::new("bench.toml");
            Playbook::from_toml(black_box(toml_content), path)
        })
    });
}

fn bench_config_validate(c: &mut Criterion) {
    let config = Config {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        playbook_dir: PathBuf::from("/etc/kea/playbooks"),
        mtls: Default::default(),
        deployment: Default::default(),
    };

    c.bench_function("config_validate", |b| {
        b.iter(|| config.validate())
    });
}

fn bench_trigger_serde(c: &mut Criterion) {
    let trigger = PlaybookTrigger::Schedule {
        cron: black_box("0 0 * * *".to_string()),
    };

    c.bench_function("trigger_serialize", |b| {
        b.iter(|| toml::to_string(&trigger))
    });

    let trigger_str = toml::to_string(&trigger).unwrap();
    c.bench_function("trigger_deserialize", |b| {
        b.iter(|| toml::from_str::<PlaybookTrigger>(&trigger_str))
    });
}

fn bench_action_serde(c: &mut Criterion) {
    let action = PlaybookAction::Command {
        command: black_box("systemctl".to_string()),
        args: black_box(vec!["restart".to_string(), "nginx".to_string()]),
        timeout_secs: Some(60),
    };

    c.bench_function("action_serialize", |b| {
        b.iter(|| toml::to_string(&action))
    });

    let action_str = toml::to_string(&action).unwrap();
    c.bench_function("action_deserialize", |b| {
        b.iter(|| toml::from_str::<PlaybookAction>(&action_str))
    });
}

fn bench_playbook_with_many_actions(c: &mut Criterion) {
    let mut actions = vec![];
    for i in 0..50 {
        actions.push(PlaybookAction::Log {
            level: "info".to_string(),
            message: format!("Action {}", i),
        });
    }

    let playbook = Playbook {
        name: "many-actions".to_string(),
        description: "Playbook with many actions".to_string(),
        trigger: PlaybookTrigger::Manual,
        actions,
        continue_on_error: false,
        timeout_secs: 300,
    };

    c.bench_function("playbook_serialize_50_actions", |b| {
        b.iter(|| toml::to_string(&playbook))
    });

    let serialized = toml::to_string(&playbook).unwrap();
    c.bench_function("playbook_deserialize_50_actions", |b| {
        b.iter(|| {
            let path = std::path::Path::new("bench.toml");
            Playbook::from_toml(&serialized, path)
        })
    });
}

criterion_group!(
    benches,
    bench_playbook_parse,
    bench_config_validate,
    bench_trigger_serde,
    bench_action_serde,
    bench_playbook_with_many_actions
);
criterion_main!(benches);
