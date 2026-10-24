//! Accepted media formats, upload profiles and bounded container inspection.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MediaKind {
    Image,
    Video,
}

impl MediaKind {
    pub(super) const fn max_bytes(self) -> usize {
        match self {
            Self::Image => 10 * 1024 * 1024,
            Self::Video => 50 * 1024 * 1024,
        }
    }

    pub(super) const fn resource_type(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
        }
    }

    pub(super) const fn invalid_format_message(self) -> &'static str {
        match self {
            Self::Image => "仅支持 PNG、JPEG、GIF 和 WebP 图片",
            Self::Video => "仅支持包含视频轨道的 MP4 和 WebM 视频",
        }
    }

    pub(super) fn detect(self, bytes: &[u8]) -> Option<MediaFormat> {
        match self {
            Self::Image => image_type(bytes),
            Self::Video => video_type(bytes),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MediaFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Mp4,
    Webm,
}

impl MediaFormat {
    const ALL: [Self; 6] = [
        Self::Png,
        Self::Jpeg,
        Self::Gif,
        Self::Webp,
        Self::Mp4,
        Self::Webm,
    ];

    const fn descriptor(self) -> (&'static str, &'static str, MediaKind) {
        match self {
            Self::Png => ("png", "image/png", MediaKind::Image),
            Self::Jpeg => ("jpg", "image/jpeg", MediaKind::Image),
            Self::Gif => ("gif", "image/gif", MediaKind::Image),
            Self::Webp => ("webp", "image/webp", MediaKind::Image),
            Self::Mp4 => ("mp4", "video/mp4", MediaKind::Video),
            Self::Webm => ("webm", "video/webm", MediaKind::Video),
        }
    }

    pub(super) const fn extension(self) -> &'static str {
        self.descriptor().0
    }

    pub(super) const fn mime_type(self) -> &'static str {
        self.descriptor().1
    }

    pub(super) const fn kind(self) -> MediaKind {
        self.descriptor().2
    }

    pub(super) fn from_filename(name: &str) -> Option<Self> {
        let (hash, extension) = name.split_once('.')?;
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return None;
        }
        Self::ALL
            .into_iter()
            .find(|format| format.extension() == extension)
    }
}

pub(crate) fn is_media_filename(name: &str) -> bool {
    MediaFormat::from_filename(name).is_some()
}

fn image_type(bytes: &[u8]) -> Option<MediaFormat> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(MediaFormat::Png);
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some(MediaFormat::Jpeg);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(MediaFormat::Gif);
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(MediaFormat::Webp);
    }
    None
}

// Parse only bounded container metadata. The browser remains responsible for
// codec support; an image, audio-only container or renamed document is not a
// video upload, regardless of the request MIME type or filename.
fn video_type(bytes: &[u8]) -> Option<MediaFormat> {
    if bytes.get(4..8) == Some(b"ftyp") && mp4_video(bytes) {
        Some(MediaFormat::Mp4)
    } else if bytes.starts_with(b"\x1a\x45\xdf\xa3") && webm_video(bytes) {
        Some(MediaFormat::Webm)
    } else {
        None
    }
}

fn mp4_box<'a>(bytes: &'a [u8], budget: &mut usize) -> Option<(&'a [u8], &'a [u8], &'a [u8])> {
    *budget = budget.checked_sub(1)?;
    let size = u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?);
    let kind = bytes.get(4..8)?;
    let (header, size) = match size {
        0 => (8, bytes.len()),
        1 => (
            16,
            usize::try_from(u64::from_be_bytes(bytes.get(8..16)?.try_into().ok()?)).ok()?,
        ),
        size => (8, size as usize),
    };
    if size < header {
        return None;
    }
    Some((kind, bytes.get(header..size)?, bytes.get(size..)?))
}

fn mp4_track(bytes: &[u8], depth: u8, budget: &mut usize) -> Option<bool> {
    let mut remaining = bytes;
    let mut video = false;
    while !remaining.is_empty() {
        let (kind, payload, rest) = mp4_box(remaining, budget)?;
        remaining = rest;
        match (depth, kind) {
            (0, b"trak") | (1, b"mdia") => video |= mp4_track(payload, depth + 1, budget)?,
            (2, b"hdlr") => {
                if payload.len() < 24 {
                    return None;
                }
                video |= payload.get(8..12) == Some(b"vide");
            }
            _ => {}
        }
    }
    Some(video)
}

fn mp4_video(bytes: &[u8]) -> bool {
    fn parse(bytes: &[u8]) -> Option<bool> {
        let mut budget = 65_536;
        let (kind, header, mut remaining) = mp4_box(bytes, &mut budget)?;
        if kind != b"ftyp" || header.len() < 8 || !header.len().is_multiple_of(4) {
            return None;
        }
        let compatible = |brand: &[u8]| {
            matches!(
                brand,
                b"isom"
                    | b"iso2"
                    | b"iso3"
                    | b"iso4"
                    | b"iso5"
                    | b"iso6"
                    | b"iso7"
                    | b"iso8"
                    | b"iso9"
                    | b"mp41"
                    | b"mp42"
                    | b"avc1"
                    | b"dash"
                    | b"MSNV"
                    | b"M4V "
                    | b"M4VH"
                    | b"M4VP"
            )
        };
        if !compatible(&header[..4])
            && !header[8..]
                .as_chunks::<4>()
                .0
                .iter()
                .any(|brand| compatible(brand))
        {
            return None;
        }
        let mut video = false;
        let mut data = false;
        while !remaining.is_empty() {
            let (kind, payload, rest) = mp4_box(remaining, &mut budget)?;
            remaining = rest;
            match kind {
                b"moov" => video |= mp4_track(payload, 0, &mut budget)?,
                b"mdat" => data |= !payload.is_empty(),
                _ => {}
            }
        }
        Some(video && data)
    }
    parse(bytes) == Some(true)
}

fn ebml_vint(bytes: &[u8], id: bool) -> Option<(u64, usize, bool)> {
    let first = *bytes.first()?;
    let width = first.leading_zeros() as usize + 1;
    if width > if id { 4 } else { 8 } {
        return None;
    }
    let mut value = if id {
        first as u64
    } else {
        u64::from(first) & (0xff_u64 >> width)
    };
    for byte in bytes.get(1..width)? {
        value = (value << 8) | u64::from(*byte);
    }
    let unknown = !id && value == (1_u64 << (width * 7)) - 1;
    Some((value, width, unknown))
}

fn ebml_element<'a>(bytes: &'a [u8], budget: &mut usize) -> Option<(u64, &'a [u8], &'a [u8])> {
    *budget = budget.checked_sub(1)?;
    let (id, id_width, _) = ebml_vint(bytes, true)?;
    let (size, size_width, unknown) = ebml_vint(bytes.get(id_width..)?, false)?;
    let header = id_width + size_width;
    let size = if unknown {
        // Streaming WebM permits unknown Segment/Cluster lengths; these extend
        // to the parent boundary and cannot reinterpret arbitrary child data.
        if !matches!(id, 0x1853_8067 | 0x1f43_b675) {
            return None;
        }
        bytes.len().checked_sub(header)?
    } else {
        usize::try_from(size).ok()?
    };
    let end = header.checked_add(size)?;
    Some((id, bytes.get(header..end)?, bytes.get(end..)?))
}

fn webm_video(bytes: &[u8]) -> bool {
    fn parse(bytes: &[u8]) -> Option<bool> {
        let mut budget = 65_536;
        let (kind, header, mut remaining) = ebml_element(bytes, &mut budget)?;
        if kind != 0x1a45_dfa3 || header.len() > 4096 {
            return None;
        }
        let mut fields = header;
        let mut webm = false;
        while !fields.is_empty() {
            let (id, value, rest) = ebml_element(fields, &mut budget)?;
            fields = rest;
            if id == 0x4282 {
                webm = value == b"webm";
            }
        }
        if !webm {
            return None;
        }
        // Void/CRC metadata may precede the Segment, but no other document is
        // accepted alongside the video container.
        let segment = loop {
            let (id, value, rest) = ebml_element(remaining, &mut budget)?;
            remaining = rest;
            match id {
                0x1853_8067 if remaining.is_empty() => break value,
                0xec | 0xbf => {}
                _ => return None,
            }
        };
        let mut children = segment;
        let mut video = false;
        let mut data = false;
        while !children.is_empty() {
            let (id, value, rest) = ebml_element(children, &mut budget)?;
            children = rest;
            if id == 0x1f43_b675 {
                data |= !value.is_empty();
            } else if id == 0x1654_ae6b {
                let mut tracks = value;
                while !tracks.is_empty() {
                    let (id, value, rest) = ebml_element(tracks, &mut budget)?;
                    tracks = rest;
                    if id != 0xae {
                        continue;
                    }
                    let mut entry = value;
                    while !entry.is_empty() {
                        let (id, value, rest) = ebml_element(entry, &mut budget)?;
                        entry = rest;
                        if id == 0x83 {
                            if value.is_empty() || value.len() > 8 {
                                return None;
                            }
                            video |= value
                                .iter()
                                .fold(0_u64, |n, byte| (n << 8) | u64::from(*byte))
                                == 1;
                        }
                    }
                }
            }
        }
        Some(video && data)
    }
    parse(bytes) == Some(true)
}

#[cfg(test)]
#[path = "../../../../test/server/unit/media/formats.rs"]
mod tests;
