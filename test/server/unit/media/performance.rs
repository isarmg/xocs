use super::*;
use std::time::Instant;

// Run with `CARGO_PROFILE_TEST_OPT_LEVEL=3 cargo test -p blog --lib upload_parser_scheduling_latency -- --ignored --nocapture`.
// This isolates ready-task latency during the parser's largest supported upload;
// it measures scheduling fairness, rather than HTTP throughput or disk writes.
#[test]
#[ignore = "manual optimized upload scheduling benchmark"]
fn upload_parser_scheduling_latency() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut video = include_bytes!("../../../browser/fixtures/background.webm").to_vec();
    let segment = video
        .windows(4)
        .position(|bytes| bytes == [0x18, 0x53, 0x80, 0x67])
        .unwrap();
    assert_eq!(video[segment + 4], 1);
    video[segment + 4..segment + 12].copy_from_slice(&[1, 255, 255, 255, 255, 255, 255, 255]);
    let padding = MediaKind::Video.max_bytes() - video.len() - 5;
    video.push(0xec);
    video.extend_from_slice(&(0x1000_0000_u32 | u32::try_from(padding).unwrap()).to_be_bytes());
    video.resize(MediaKind::Video.max_bytes(), 0);
    let body = Bytes::from(video);
    let mut expected_filename = None;
    for offload in [false, true] {
        let mut ready_latencies = Vec::new();
        let mut completion_times = Vec::new();
        for _ in 0..7 {
            let body = body.clone();
            let (ready, complete, filename) = runtime.block_on(async move {
                let start = Instant::now();
                let ready = tokio::spawn(async move { start.elapsed() });
                let parse = move || MediaUpload::parse(MediaKind::Video, &HeaderMap::new(), body);
                let upload = if offload {
                    tokio::task::spawn_blocking(parse).await.unwrap().unwrap()
                } else {
                    parse().unwrap()
                };
                let complete = start.elapsed();
                let filename = upload.filename.as_str().to_owned();
                (ready.await.unwrap(), complete, filename)
            });
            if let Some(expected) = &expected_filename {
                assert_eq!(&filename, expected);
            } else {
                expected_filename = Some(filename);
            }
            ready_latencies.push(ready.as_secs_f64() * 1000.0);
            completion_times.push(complete.as_secs_f64() * 1000.0);
        }
        ready_latencies.sort_by(f64::total_cmp);
        completion_times.sort_by(f64::total_cmp);
        eprintln!(
            "offload={offload}, bytes={}, samples=7, ready_task_median_ms={:.3}, parse_complete_median_ms={:.3}",
            body.len(),
            ready_latencies[3],
            completion_times[3]
        );
    }
}
