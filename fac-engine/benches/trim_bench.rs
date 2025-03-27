use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use facto_loop_miner_fac_engine::admiral::trimmer::{
    string_space_shrinker_doubler, string_space_shrinker_state_slice,
    string_space_shrinker_state_vec,
};

fn new_bencher(c: &mut Criterion) {
    for (input_name, input) in [
        (
            "long",
            "   this   should   \n   split me  and I'm a teapot    ",
        ),
        ("sentence_clean", "I'm just a teapot"),
        ("sentence_newline", "I'm just\na teapot"),
        ("word", "teapot"),
    ] {
        for repeat_size in [1usize, 50, 999] {
            let param = format!("{input_name}{repeat_size}");
            let mut group = c.benchmark_group(&param);
            group.bench_with_input(
                BenchmarkId::from_parameter("state_slice"),
                &(input, repeat_size),
                |b, (b_input, b_repeat_size)| {
                    b.iter(|| string_space_shrinker_state_slice(b_input.repeat(*b_repeat_size)))
                },
            );
            group.bench_with_input(
                BenchmarkId::from_parameter("state_vec"),
                &(input, repeat_size),
                |b, (b_input, b_repeat_size)| {
                    b.iter(|| string_space_shrinker_state_vec(b_input.repeat(*b_repeat_size)))
                },
            );
            group.bench_with_input(
                BenchmarkId::from_parameter("doubler"),
                &(input, repeat_size),
                |b, (b_input, b_repeat_size)| {
                    b.iter(|| string_space_shrinker_doubler(b_input.repeat(*b_repeat_size)))
                },
            );
            group.finish()
        }
    }
}

criterion_group!(benches, new_bencher);
criterion_main!(benches);
