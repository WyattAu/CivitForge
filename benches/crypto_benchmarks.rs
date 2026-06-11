use criterion::{black_box, criterion_group, criterion_main, Criterion};
use civit_crypto::abac::conditions::{
    AbacContext, AbacEnvironment, AbacResource, AbacSubject, DevicePosture, PolicyCondition,
    ConditionType,
};
use civit_crypto::abac::engine::{AbacEngine, AbacPolicy, Effect as AbacEffect};
use civit_crypto::cel::{CelEnvironment, CelExpression, CelEvaluator, CelType, CelValue};
use civit_crypto::hash::{HashAlgorithm, HashService};
use civit_crypto::hmac::HmacService;
use civit_crypto::policy::{
    Action, Condition, Effect, PolicyEngine, PolicyStatement, Resource, Subject,
};
use civit_crypto::repo_keys::RepoEncryptionKey;
use std::collections::HashMap;
use uuid::Uuid;

fn bench_sha256(c: &mut Criterion) {
    let mut group = c.benchmark_group("sha256");
    for size in [1_024, 10_024, 100_024] {
        let data = vec![0xABu8; size];
        group.bench_with_input(format!("hash_{size}_bytes"), &data, |b, data| {
            b.iter(|| black_box(HashService::hash(HashAlgorithm::Sha256, data)));
        });
    }
    group.finish();
}

fn bench_hmac(c: &mut Criterion) {
    let key = b"benchmark-secret-key-for-hmac";
    let data = b"The quick brown fox jumps over the lazy dog";
    c.bench_function("hmac_sha256_sign", |b| {
        b.iter(|| black_box(HmacService::sign(key, data)));
    });
}

fn bench_aes_gcm(c: &mut Criterion) {
    let master = [0x42u8; 32];
    let repo_id = Uuid::nil();
    let key = RepoEncryptionKey::derive(&master, repo_id).unwrap();

    let mut group = c.benchmark_group("aes256gcm");
    for size in [64, 1_024, 10_024] {
        let plaintext = vec![0xCDu8; size];
        let (nonce, ciphertext) = key.encrypt(&plaintext).unwrap();

        group.bench_function(format!("encrypt_{size}_bytes"), |b| {
            b.iter(|| black_box(key.encrypt(&plaintext).unwrap()));
        });
        group.bench_function(format!("decrypt_{size}_bytes"), |b| {
            b.iter(|| {
                let mut ct = ciphertext.clone();
                key.decrypt(&nonce, &mut ct).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_cel(c: &mut Criterion) {
    let env = {
        let mut user_map = HashMap::new();
        user_map.insert("role".to_string(), CelValue::String("admin".to_string()));
        user_map.insert("age".to_string(), CelValue::Int(30));
        user_map.insert(
            "permissions".to_string(),
            CelValue::List(vec![
                CelValue::String("read".into()),
                CelValue::String("write".into()),
            ]),
        );
        CelEnvironment::new()
            .with_variable(
                "user",
                CelValue::Map(user_map),
                CelType::Map(Box::new(CelType::String), Box::new(CelType::Dyn)),
            )
            .with_variable(
                "source",
                CelValue::Map({
                    let mut m = HashMap::new();
                    m.insert("ip".to_string(), CelValue::String("10.0.0.1".to_string()));
                    m
                }),
                CelType::Map(Box::new(CelType::String), Box::new(CelType::Dyn)),
            )
    };
    let evaluator = CelEvaluator::new(env);

    let mut group = c.benchmark_group("cel");
    let exprs = [
        CelExpression::parse("user.role == \"admin\""),
        CelExpression::parse("user.age > 18 && user.role == \"admin\""),
        CelExpression::parse("\"read\" in user.permissions"),
        CelExpression::parse("size(user.permissions) > 0"),
        CelExpression::parse("!user.suspended"),
    ];
    for (i, expr) in exprs.iter().enumerate() {
        group.bench_with_input(format!("evaluate_{i}"), expr, |b, e| {
            b.iter(|| black_box(evaluator.evaluate(e)));
        });
    }
    group.finish();
}

fn bench_policy_evaluation(c: &mut Criterion) {
    let subject = Subject {
        id: "admin-1".into(),
        roles: vec!["admin".into()],
        groups: vec!["org-1".into()],
        attributes: HashMap::new(),
    };
    let attrs: HashMap<String, String> = vec![("visibility".into(), "public".into())]
        .into_iter()
        .collect();

    let mut group = c.benchmark_group("policy");
    for rule_count in [1, 5, 10] {
        let statements: Vec<PolicyStatement> = (0..rule_count)
            .map(|i| PolicyStatement {
                id: format!("rule-{i}"),
                effect: Effect::Allow,
                actions: vec![Action::Get, Action::List],
                resources: vec![Resource::Repository],
                principals: None,
                conditions: Some(vec![Condition::RoleRequired("admin".into())]),
            })
            .collect();
        let engine = PolicyEngine::new(statements);
        group.bench_with_input(format!("evaluate_{rule_count}_rules"), &engine, |b, e| {
            b.iter(|| {
                black_box(e.evaluate(&subject, Action::Get, Resource::Repository, &attrs))
            });
        });
    }
    group.finish();
}

fn bench_abac_policy(c: &mut Criterion) {
    let mut group = c.benchmark_group("abac_policy");
    for rule_count in [1, 5, 10] {
        group.bench_function(format!("evaluate_{rule_count}_rules"), |b| {
            b.iter(|| {
                let mut engine = AbacEngine::new();
                for i in 0..rule_count {
                    engine.add_policy(
                        AbacPolicy::new(
                            format!("p{i}"),
                            format!("Rule {i}"),
                            AbacEffect::Allow,
                            i as u32,
                        )
                        .with_action("deploy")
                        .with_resource_type("server")
                        .with_condition(PolicyCondition::new(
                            ConditionType::RoleMatch,
                            "admin",
                            serde_json::Value::Null,
                        )),
                    );
                }
                let subject = AbacSubject::new("user-1").with_role("admin");
                let resource = AbacResource::new("res-1", "server");
                let env = AbacEnvironment::new().with_device_posture(DevicePosture::Managed);
                let ctx = AbacContext::new(subject, resource, "deploy", env);
                black_box(engine.evaluate(&ctx))
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_sha256,
    bench_hmac,
    bench_aes_gcm,
    bench_cel,
    bench_policy_evaluation,
    bench_abac_policy,
);
criterion_main!(benches);
