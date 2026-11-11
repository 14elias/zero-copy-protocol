
use bytes::Bytes;
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

use zero_copy_protocol::{
    bytes_parser,
    hybrid_parser,
    naive,
    nom_parser,
};

fn benchmark_parsers(c: &mut Criterion) {
    // One valid packet:
    // Magic = RUST
    // Version = 1
    // Message type = 1 (Text)
    // Payload length = 5
    // Payload = Hello
    static PACKET_DATA: [u8; 13] = [
        b'R', b'U', b'S', b'T',
        1, 1,
        0, 5,
        b'H', b'e', b'l', b'l', b'o',
    ];

    // Prepare input data outside the benchmark loops.
    let bytes_packet = Bytes::from_static(&PACKET_DATA);

    let mut group = c.benchmark_group("packet_parser");

    group.bench_function("naive_vec", |b| {
        b.iter(|| {
            let result = naive::parse_packet(black_box(&PACKET_DATA));
            black_box(result)
        });
    });

    group.bench_function("nom_borrowed", |b| {
        b.iter(|| {
            let result = nom_parser::parse_packet(black_box(&PACKET_DATA));
            black_box(result)
        });
    });

    group.bench_function("bytes_manual", |b| {
        b.iter(|| {
            let result =
                bytes_parser::parse_packet(black_box(&bytes_packet));
            black_box(result)
        });
    });

    group.bench_function("nom_bytes_hybrid", |b| {
        b.iter(|| {
            let result =
                hybrid_parser::parse_packet(black_box(&bytes_packet));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_parsers);
criterion_main!(benches);
