//! EXIF extraction via `kamadak-exif`, with `imagesize` for header dimensions.
//!
//! Cloudreve only extracts media metadata when the storage policy enables it.
//! When it did not, the client reads the head of the original file: EXIF gives
//! `DateTimeOriginal` / `DateTime`, and the image header gives the pixel size
//! even for files without any EXIF block.

use std::io::Cursor;

use exif::{Tag, Value};
use time::PrimitiveDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::time::unix_millis;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExifData {
    pub taken_at: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

/// `2024:03:01 12:34:56` — EXIF timestamps carry no timezone.
const EXIF_DATETIME: &[BorrowedFormatItem<'static>] =
    format_description!("[year]:[month]:[day] [hour]:[minute]:[second]");

/// Parse capture time and pixel size from the first bytes of an image file.
pub fn parse_exif(bytes: &[u8]) -> ExifData {
    let mut data = exif::Reader::new()
        .read_from_container(&mut Cursor::new(bytes))
        .map(|exif| ExifData {
            taken_at: capture_time(&exif),
            width: dimension(&exif, &[Tag::PixelXDimension, Tag::ImageWidth]),
            height: dimension(&exif, &[Tag::PixelYDimension, Tag::ImageLength]),
        })
        .unwrap_or_default();

    if (data.width.is_none() || data.height.is_none())
        && let Ok(size) = imagesize::blob_size(bytes)
    {
        data.width = data.width.or_else(|| i32::try_from(size.width).ok());
        data.height = data.height.or_else(|| i32::try_from(size.height).ok());
    }
    data
}

fn capture_time(exif: &exif::Exif) -> Option<i64> {
    let field = find_field(
        exif,
        &[Tag::DateTimeOriginal, Tag::DateTimeDigitized, Tag::DateTime],
    )?;
    let Value::Ascii(values) = &field.value else {
        return None;
    };
    let raw = std::str::from_utf8(values.first()?).ok()?;
    PrimitiveDateTime::parse(raw.trim_end_matches('\0'), EXIF_DATETIME)
        .ok()
        .map(|value| unix_millis(value.assume_utc()))
}

fn dimension(exif: &exif::Exif, tags: &[Tag]) -> Option<i32> {
    i32::try_from(find_field(exif, tags)?.value.get_uint(0)?).ok()
}

/// Tags are matched by number: a tag is not always stored in the IFD its
/// context belongs to, and the context is part of `Tag`'s identity.
fn find_field<'a>(exif: &'a exif::Exif, tags: &[Tag]) -> Option<&'a exif::Field> {
    tags.iter().find_map(|wanted| {
        exif.fields()
            .find(|field| field.tag.number() == wanted.number())
    })
}
