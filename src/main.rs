//! Bad Apple demo for eepyOS on the Touchscreen E-Paper Input Module for Framework 16
#![no_main]
#![no_std]

extern crate panic_halt;

use eepy_sys::header::ProgramSlotHeader;
use eepy_sys::image::{refresh, write_image, RefreshBlockMode};
use eepy_sys::IMAGE_BYTES;
use eepy_sys::misc::get_time_micros;

#[link_section = ".header"]
#[used]
static HEADER: ProgramSlotHeader = ProgramSlotHeader::partial(
    "Bad Apple",
    env!("CARGO_PKG_VERSION"),
    main,
);

/// Contains the video data.
///
/// Each frame consists of a 16-bit length followed by the image data as a DEFLATE-compressed raw
/// binary monochrome 240x180 image.
///
/// Video is stored at 2FPS (as this is approximately the same as the display's maximum refresh rate);
/// all frames are concatenated into this one file.
static BAD_APPLE: &[u8] = include_bytes!("../badapple.bin");

/// The offset into the framebuffer that we want to write our image to (places the video in the
/// middle of the screen)
const BADAPPLE_FRAME_OFFSET: usize = 3540;

extern "C" fn main() {
    let mut framebuffer = [0u8; IMAGE_BYTES];
    let mut bad_apple = BAD_APPLE;

    let start_time = get_time_micros();
    let mut video_time = 0u64;

    while bad_apple.len() > 0 {
        // Skip frames as necessary so the video timing is correct
        // (the display takes slightly longer than 500ms to refresh; we are assuming here that it
        // will never be faster than 500ms)
        while (get_time_micros() - start_time - video_time) > 500000 {
            let len = u16::from_le_bytes([bad_apple[0], bad_apple[1]]) as usize;
            bad_apple = &bad_apple[len + 2..];
            video_time += 500000;
        }

        let len = u16::from_le_bytes([bad_apple[0], bad_apple[1]]) as usize;
        // Decompress the next frame directly into the framebuffer
        miniz_oxide::inflate::decompress_slice_iter_to_slice(
            &mut framebuffer[BADAPPLE_FRAME_OFFSET..],
            [&bad_apple[2..len + 2]].into_iter(),
            false,
            true,
        ).unwrap();

        // Display the frame
        write_image(&framebuffer);
        refresh(true, RefreshBlockMode::BlockFinish);

        bad_apple = &bad_apple[len + 2..];
        video_time += 500000;
    }
}