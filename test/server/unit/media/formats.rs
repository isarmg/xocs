use super::*;

fn boxed(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(kind);
    bytes.extend_from_slice(payload);
    bytes
}

fn mp4(handler: &[u8; 4], brand: &[u8; 4]) -> Vec<u8> {
    let header = [brand.as_slice(), &[0; 4], brand.as_slice()].concat();
    let handler = [&[0; 8][..], handler.as_slice(), &[0; 12][..]].concat();
    let media = boxed(b"mdia", &boxed(b"hdlr", &handler));
    let movie = boxed(b"moov", &boxed(b"trak", &media));
    [boxed(b"ftyp", &header), movie, boxed(b"mdat", b"frame")].concat()
}

#[test]
fn video_formats_require_container_metadata_and_a_video_track() {
    let video_type = |bytes: &[u8]| {
        super::video_type(bytes).map(|format| (format.extension(), format.mime_type()))
    };
    let video = mp4(b"vide", b"mp42");
    assert_eq!(video_type(&video), Some(("mp4", "video/mp4")));
    assert_eq!(video_type(&mp4(b"soun", b"mp42")), None);
    assert_eq!(video_type(&mp4(b"vide", b"avif")), None);
    assert_eq!(video_type(b"<html>not a video</html>"), None);
    assert_eq!(video_type(b"\x89PNG\r\n\x1a\n"), None);
    for size in 0..video.len() {
        assert_eq!(video_type(&video[..size]), None, "truncated MP4 at {size}");
    }
    let mut oversized = video;
    oversized[..4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(video_type(&oversized), None);

    let webm = include_bytes!("../../../browser/fixtures/background.webm");
    assert_eq!(video_type(webm), Some(("webm", "video/webm")));
    let mut audio = webm.to_vec();
    let track = audio
        .windows(3)
        .position(|bytes| bytes == [0x83, 0x81, 0x01])
        .unwrap();
    audio[track + 2] = 2;
    assert_eq!(video_type(&audio), None);
    let mut document = webm.to_vec();
    let doctype = document
        .windows(4)
        .position(|bytes| bytes == b"webm")
        .unwrap();
    document[doctype..doctype + 4].copy_from_slice(b"html");
    assert_eq!(video_type(&document), None);
    for size in 0..webm.len() {
        assert_eq!(video_type(&webm[..size]), None, "truncated WebM at {size}");
    }
    assert!(
        ebml_vint(&[1, 255, 255, 255, 255, 255, 255, 255], VintKind::Size)
            .unwrap()
            .2
    );
}

#[test]
fn managed_video_names_keep_the_same_path_boundary_as_images() {
    let hash = "a".repeat(64);
    for extension in ["png", "jpg", "gif", "webp", "mp4", "webm"] {
        assert!(is_media_filename(&format!("{hash}.{extension}")));
    }
    for name in [
        format!("{hash}.html"),
        format!("{hash}.svg"),
        format!("{hash}.MP4"),
        format!("../{hash}.webm"),
        format!("{hash}.mp4/other"),
        format!("{}.webm", "A".repeat(64)),
    ] {
        assert!(!is_media_filename(&name), "{name}");
    }
}

#[test]
fn canonical_formats_are_unique_and_roundtrip_detected_upload_identity() {
    let mut extensions = std::collections::HashSet::new();
    let mut mime_types = std::collections::HashSet::new();
    let hash = "a".repeat(64);
    for format in MediaFormat::ALL {
        assert!(extensions.insert(format.extension()));
        assert!(mime_types.insert(format.mime_type()));
        let filename = format!("{hash}.{}", format.extension());
        assert_eq!(MediaFormat::from_filename(&filename), Some(format));
    }

    let mp4 = mp4(b"vide", b"mp42");
    for (kind, data) in [
        (
            MediaKind::Image,
            include_bytes!("../../../browser/fixtures/image.png").as_slice(),
        ),
        (MediaKind::Video, mp4.as_slice()),
        (
            MediaKind::Video,
            include_bytes!("../../../browser/fixtures/background.webm").as_slice(),
        ),
    ] {
        let format = kind.detect(data).expect("existing valid upload fixture");
        assert_eq!(format.kind(), kind);
        let filename = format!("{hash}.{}", format.extension());
        assert_eq!(MediaFormat::from_filename(&filename), Some(format));
    }
}
