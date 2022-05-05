use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use datamodel::ios;
use std::time;

/// Helper function to load a file.
fn load_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}

//helper to convert the given json data to ahap
fn model_to_ahap(data: &str) {
    let (_, model) = datamodel::upgrade_to_latest(&datamodel::from_json(data).unwrap()).unwrap();
    ios::convert_to_transient_and_continuous_ahaps(model);
}

///benchmarks v0 deserialised and validated
fn v0_deserialise_validate(c: &mut Criterion) {
    //setup  data
    let half_sec = load_file("benches/data/v0-0.5.vij");
    let one_sec = load_file("benches/data/v0-1.vij");
    let ten_sec = load_file("benches/data/v0-10.vij");
    let sixty_sec = load_file("benches/data/v0-60.vij");
    let one_twenty_sec = load_file("benches/data/v0-120.vij");

    //create the group to measure
    let mut benchmark_group = c.benchmark_group("v0-deserialisation-validate");
    benchmark_group.measurement_time(time::Duration::from_secs(30));

    //run
    benchmark_group
        .bench_function("0.5 sec", |b| b.iter(|| datamodel::from_json(&half_sec)))
        .bench_function("1 sec", |b| b.iter(|| datamodel::from_json(&one_sec)))
        .bench_function("10 sec", |b| b.iter(|| datamodel::from_json(&ten_sec)))
        .bench_function("60 sec", |b| b.iter(|| datamodel::from_json(&sixty_sec)))
        .bench_function("120 sec", |b| {
            b.iter(|| datamodel::from_json(&one_twenty_sec))
        });

    benchmark_group.finish();
}

///benchmarks v1 deserialised and validated
fn v1_deserialise_validate(c: &mut Criterion) {
    //setup  data
    let half_sec = load_file("benches/data/v1-0.5.haptic");
    let one_sec = load_file("benches/data/v1-1.haptic");
    let ten_sec = load_file("benches/data/v1-10.haptic");
    let sixty_sec = load_file("benches/data/v1-60.haptic");
    let one_twenty_sec = load_file("benches/data/v1-120.haptic");

    //create the group to measure
    let mut benchmark_group = c.benchmark_group("v1-deserialisation-validate");
    benchmark_group.measurement_time(time::Duration::from_secs(30));

    //run
    benchmark_group
        .bench_function("0.5 sec", |b| b.iter(|| datamodel::from_json(&half_sec)))
        .bench_function("1 sec", |b| b.iter(|| datamodel::from_json(&one_sec)))
        .bench_function("10 sec", |b| b.iter(|| datamodel::from_json(&ten_sec)))
        .bench_function("60 sec", |b| b.iter(|| datamodel::from_json(&sixty_sec)))
        .bench_function("120 sec", |b| {
            b.iter(|| datamodel::from_json(&one_twenty_sec))
        });

    benchmark_group.finish();
}

///benchmarks v0 to the latest version for  0.5,1,10,60 and 120 seconds worth of data.
fn v0_to_latest(c: &mut Criterion) {
    let half_sec = datamodel::from_json(&load_file("benches/data/v0-0.5.vij")).unwrap();
    let one_sec = datamodel::from_json(&load_file("benches/data/v0-1.vij")).unwrap();
    let ten_sec = datamodel::from_json(&load_file("benches/data/v0-10.vij")).unwrap();
    let sixty_sec = datamodel::from_json(&load_file("benches/data/v0-60.vij")).unwrap();
    let one_twenty_sec = datamodel::from_json(&load_file("benches/data/v0-120.vij")).unwrap();

    let mut benchmark_group = c.benchmark_group("v0-to-latest");
    benchmark_group.measurement_time(time::Duration::from_secs(30));

    benchmark_group
        .bench_function("0.5 sec", |b| {
            b.iter(|| datamodel::upgrade_to_latest(&half_sec))
        })
        .bench_function("1 sec", |b| {
            b.iter(|| datamodel::upgrade_to_latest(&one_sec))
        })
        .bench_function("10 sec", |b| {
            b.iter(|| datamodel::upgrade_to_latest(&ten_sec))
        })
        .bench_function("60 sec", |b| {
            b.iter(|| datamodel::upgrade_to_latest(&sixty_sec))
        })
        .bench_function("120 sec", |b| {
            b.iter(|| datamodel::upgrade_to_latest(&one_twenty_sec))
        });

    benchmark_group.finish();
}

///benchmarks v0 to ahap or 0.5,1,10,60 and 120 seconds worth of data.
fn v0_to_ahap(c: &mut Criterion) {
    let half_sec = load_file("benches/data/v0-0.5.vij");
    let one_sec = load_file("benches/data/v0-1.vij");
    let ten_sec = load_file("benches/data/v0-10.vij");
    let sixty_sec = load_file("benches/data/v0-60.vij");
    let one_twenty_sec = load_file("benches/data/v0-120.vij");

    let mut benchmark_group = c.benchmark_group("v0-to-ahap");
    benchmark_group.measurement_time(time::Duration::from_secs(30));

    benchmark_group
        .bench_function("0.5 sec", |b| b.iter(|| model_to_ahap(&half_sec)))
        .bench_function("1 sec", |b| b.iter(|| model_to_ahap(&one_sec)))
        .bench_function("10 sec", |b| b.iter(|| model_to_ahap(&ten_sec)))
        .bench_function("60 sec", |b| b.iter(|| model_to_ahap(&sixty_sec)))
        .bench_function("120 sec", |b| b.iter(|| model_to_ahap(&one_twenty_sec)));

    benchmark_group.finish();
}

///benchmarks v1 to ahap for 0.5,1,10,60 and 120 seconds worth of data.
fn v1_to_ahap(c: &mut Criterion) {
    let half_sec = load_file("benches/data/v1-0.5.haptic");
    let one_sec = load_file("benches/data/v1-1.haptic");
    let ten_sec = load_file("benches/data/v1-10.haptic");
    let sixty_sec = load_file("benches/data/v1-60.haptic");
    let one_twenty_sec = load_file("benches/data/v1-120.haptic");

    let mut benchmark_group = c.benchmark_group("v1-to-ahap");
    benchmark_group.measurement_time(time::Duration::from_secs(30));

    benchmark_group
        .bench_function("0.5 sec", |b| b.iter(|| model_to_ahap(&half_sec)))
        .bench_function("1 sec", |b| b.iter(|| model_to_ahap(&one_sec)))
        .bench_function("10 sec", |b| b.iter(|| model_to_ahap(&ten_sec)))
        .bench_function("60 sec", |b| b.iter(|| model_to_ahap(&sixty_sec)))
        .bench_function("120 sec", |b| b.iter(|| model_to_ahap(&one_twenty_sec)));

    benchmark_group.finish();
}

criterion_group!(
    datamodel_benches,
    v0_deserialise_validate,
    v1_deserialise_validate,
    v0_to_latest,
    v0_to_ahap,
    v1_to_ahap
);
criterion_main!(datamodel_benches);
