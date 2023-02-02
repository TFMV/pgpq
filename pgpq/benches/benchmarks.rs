#![allow(unused)]

use arrow::datatypes::{DataType, Schema, TimeUnit};
use arrow::record_batch::RecordBatchReader;
use arrow_array::RecordBatch;
use bytes::BytesMut;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use pgpq::ArrowToPostgresBinaryEncoder;
use std::fs;
use std::path::PathBuf;

fn setup(row_limit: Option<usize>) -> (Vec<RecordBatch>, Schema) {
    let file = fs::File::open(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/yellow_tripdata_2022-01.parquet"),
    )
    .unwrap();
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).unwrap();
    let mut reader = builder.build().unwrap();
    let schema = Schema::new(reader.schema().fields().clone());
    let data: Vec<RecordBatch> = match row_limit {
        Some(n) => reader.into_iter().take(n).map(|v| v.unwrap()).collect(),
        None => reader.into_iter().map(|v| v.unwrap()).collect(),
    };
    (data, schema)
}

fn bench(input: (Vec<RecordBatch>, Schema)) {
    let (reader, schema) = input;
    let mut encoder = ArrowToPostgresBinaryEncoder::new(schema);
    let mut buff = BytesMut::new();
    encoder.write_header(&mut buff);
    for batch in reader {
        encoder.write_batch(batch, &mut buff);
    }
    encoder.write_footer(&mut buff);
}

pub fn benchmark_nyc_taxi_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_nyc_taxi_small");
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.sample_size(10); // the minimum
    group.bench_function("NYC Yello Taxi 100 rows", |b| {
        b.iter_with_setup(|| setup(Some(100)), bench)
    });
}

pub fn benchmark_nyc_taxi_full(c: &mut Criterion) {
    c.bench_function("NYC Yello Taxi Full", |b| {
        b.iter_with_setup(|| setup(None), bench)
    });
}

criterion_group!(benches, benchmark_nyc_taxi_small, benchmark_nyc_taxi_full);
criterion_main!(benches);