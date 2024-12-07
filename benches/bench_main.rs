use criterion::criterion_main;

mod day3;
mod day6;
mod day7;

criterion_main! {
    day3::benches,
    day6::benches,
    day7::benches,
}
