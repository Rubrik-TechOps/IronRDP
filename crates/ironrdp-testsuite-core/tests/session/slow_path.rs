//! Slow-path graphics updates (Share Data PDU of type Update).
//!
//! The client advertises fast-path output, but some servers (xrdp) still send
//! some updates over slow path. They used to end the session with
//! `unhandled PDU: "Update PDU"`; bitmaps are now drawn like fast-path bitmap
//! updates and the other update types are ignored.

use ironrdp_graphics::image_processing::PixelFormat;
use ironrdp_session::fast_path::{ProcessorBuilder, UpdateKind};
use ironrdp_session::image::DecodedImage;

fn processor() -> ironrdp_session::fast_path::Processor {
    ProcessorBuilder {
        io_channel_id: 1003,
        user_channel_id: 1007,
        enable_server_pointer: false,
        pointer_software_rendering: false,
        bulk_decompressor: None,
    }
    .build()
}

/// TS_UPDATE_BITMAP_DATA with one uncompressed 16 bpp rectangle at (1,1)-(2,2),
/// every pixel white (0xFFFF).
fn slow_path_bitmap_update() -> Vec<u8> {
    let mut data = Vec::new();
    let push = |d: &mut Vec<u8>, v: u16| d.extend_from_slice(&v.to_le_bytes());
    push(&mut data, 0x0001); // updateType = UPDATETYPE_BITMAP
    push(&mut data, 1); // numberRectangles
                        // TS_BITMAP_DATA
    push(&mut data, 1); // destLeft
    push(&mut data, 1); // destTop
    push(&mut data, 2); // destRight (inclusive)
    push(&mut data, 2); // destBottom (inclusive)
    push(&mut data, 2); // width
    push(&mut data, 2); // height
    push(&mut data, 16); // bitsPerPixel
    push(&mut data, 0); // flags: uncompressed
    push(&mut data, 8); // bitmapLength: 2 rows x 4 bytes (no padding needed)
    data.extend_from_slice(&[0xFF; 8]);
    data
}

#[test]
fn slow_path_bitmap_update_is_drawn() {
    let mut image = DecodedImage::new(PixelFormat::RgbA32, 4, 4);
    let before = image.data().to_vec();

    let updates = processor()
        .process_slow_path_update(&mut image, &slow_path_bitmap_update())
        .expect("slow-path bitmap update is processed");

    assert_eq!(updates.len(), 1);
    match &updates[0] {
        UpdateKind::Region(rect) => {
            assert_eq!((rect.left, rect.top, rect.right, rect.bottom), (1, 1, 2, 2));
        }
        _ => panic!("expected a region update"),
    }
    assert_ne!(image.data(), before.as_slice(), "the bitmap was drawn onto the image");
    // Pixel (0,0) is outside the rectangle and untouched.
    assert_eq!(&image.data()[..4], &before[..4]);
}

#[test]
fn other_slow_path_updates_are_ignored_not_fatal() {
    let mut image = DecodedImage::new(PixelFormat::RgbA32, 4, 4);
    let before = image.data().to_vec();
    let mut p = processor();

    for (name, data) in [
        ("orders", vec![0x00, 0x00, 0x00, 0x00]),
        ("palette", vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("synchronize", vec![0x03, 0x00, 0x00, 0x00]),
        ("unknown type", vec![0x09, 0x00]),
        ("too short", vec![0x01]),
        ("empty", vec![]),
    ] {
        let updates = p
            .process_slow_path_update(&mut image, &data)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(updates.is_empty(), "{name}: no graphics update expected");
    }
    assert_eq!(image.data(), before.as_slice());
}

#[test]
fn malformed_slow_path_bitmap_is_an_error() {
    let mut image = DecodedImage::new(PixelFormat::RgbA32, 4, 4);
    let mut data = slow_path_bitmap_update();
    data.truncate(10); // rectangle header cut short
    assert!(processor().process_slow_path_update(&mut image, &data).is_err());
}

/// TS_UPDATE_BITMAP_DATA with one rectangle covering (1,1)-(2,1): 2x1 pixels.
fn bitmap_update(bpp: u16, flags: u16, bitmap: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();
    let push = |d: &mut Vec<u8>, v: u16| d.extend_from_slice(&v.to_le_bytes());
    push(&mut data, 0x0001); // UPDATETYPE_BITMAP
    push(&mut data, 1); // numberRectangles
    for v in [1u16, 1, 2, 1, 2, 1, bpp, flags] {
        // destLeft, destTop, destRight, destBottom, width, height, bpp, flags
        push(&mut data, v);
    }
    push(&mut data, u16::try_from(bitmap.len()).unwrap());
    data.extend_from_slice(bitmap);
    data
}

/// RGBA of pixel (x, y) in a 4x4 RgbA32 image.
fn pixel(image: &DecodedImage, x: usize, y: usize) -> [u8; 4] {
    let i = (y * 4 + x) * 4;
    image.data()[i..i + 4].try_into().unwrap()
}

const EXPECTED: [u8; 4] = [0x30, 0x20, 0x10, 0xFF]; // R, G, B, A

#[test]
fn rle_24_bpp_bitmap_is_drawn_with_bgr_byte_order() {
    // Interleaved RLE: REGULAR_COLOR_RUN (0x3 << 5) of length 2, then one B,G,R pixel.
    let rle = [0x62, 0x10, 0x20, 0x30];
    // BITMAP_COMPRESSION | NO_BITMAP_COMPRESSION_HDR
    let update = bitmap_update(24, 0x0001 | 0x0400, &rle);
    let mut image = DecodedImage::new(PixelFormat::RgbA32, 4, 4);

    let updates = processor().process_slow_path_update(&mut image, &update).unwrap();

    assert!(matches!(updates.as_slice(), [UpdateKind::Region(_)]));
    assert_eq!(pixel(&image, 1, 1), EXPECTED);
    assert_eq!(pixel(&image, 2, 1), EXPECTED);
    assert_eq!(pixel(&image, 3, 1), [0, 0, 0, 0], "outside the rectangle");
}

#[test]
fn uncompressed_24_bpp_bitmap_is_drawn() {
    // One row of two B,G,R pixels (6 bytes) padded to a multiple of 4.
    let raw = [0x10, 0x20, 0x30, 0x10, 0x20, 0x30, 0x00, 0x00];
    let update = bitmap_update(24, 0, &raw);
    let mut image = DecodedImage::new(PixelFormat::RgbA32, 4, 4);

    processor().process_slow_path_update(&mut image, &update).unwrap();

    assert_eq!(pixel(&image, 1, 1), EXPECTED);
    assert_eq!(pixel(&image, 2, 1), EXPECTED);
}

#[test]
fn uncompressed_32_bpp_bitmap_is_drawn() {
    let raw = [0x10, 0x20, 0x30, 0x00, 0x10, 0x20, 0x30, 0x00]; // B,G,R,X x2
    let update = bitmap_update(32, 0, &raw);
    let mut image = DecodedImage::new(PixelFormat::RgbA32, 4, 4);

    processor().process_slow_path_update(&mut image, &update).unwrap();

    assert_eq!(&pixel(&image, 1, 1)[..3], &EXPECTED[..3]);
    assert_eq!(&pixel(&image, 2, 1)[..3], &EXPECTED[..3]);
}
