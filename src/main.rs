//! Bad Apple demo for eepyOS on the Touchscreen E-Paper Input Module for Framework 16
#![no_main]
#![no_std]

extern crate panic_halt;

use core::fmt::Write;
use core::num::NonZeroU64;
use eepy_gui::draw_target::EpdDrawTarget;
use eepy_gui::element::DEFAULT_TEXT_STYLE;
use eepy_sys::header::ProgramSlotHeader;
use eepy_sys::misc::get_time_micros;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::Text;

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
/// Video is stored at 30/17 ≈ 1.76 FPS (as this is approximately the same as the display's maximum
/// refresh rate); all frames are concatenated into this one file.
static BAD_APPLE: &[u8] = include_bytes!("../badapple.bin");

/// The offset into the framebuffer that we want to write our image to (places the video in the
/// middle of the screen)
const BADAPPLE_FRAME_OFFSET: usize = 3540;

const FRAME_TIME_MICROS: u64 = 566667;

#[cfg(feature = "timing-debug")]
fn draw_timing_debug(current_time: u64, video_time: u64, prev_time: Option<NonZeroU64>, draw_target: &mut EpdDrawTarget) {
    let mut debug_string = heapless::String::<32>::new();
    write!(debug_string, "R: {current_time:017}").unwrap();
    let text = Text::new(&debug_string, Point::new(10, 345), DEFAULT_TEXT_STYLE);
    text.draw(draw_target).unwrap();

    debug_string.clear();
    write!(debug_string, "V: {video_time:017}").unwrap();
    let text = Text::new(&debug_string, Point::new(10, 365), DEFAULT_TEXT_STYLE);
    text.draw(draw_target).unwrap();

    debug_string.clear();
    write!(debug_string, "Delta: {}", current_time as i64 - video_time as i64).unwrap();
    let text = Text::new(&debug_string, Point::new(10, 385), DEFAULT_TEXT_STYLE);
    text.draw(draw_target).unwrap();

    if let Some(t) = prev_time {
        let frametime = current_time - t.get();
        debug_string.clear();
        write!(debug_string, "Last frame: {frametime}").unwrap();
        let text = Text::new(&debug_string, Point::new(10, 405), DEFAULT_TEXT_STYLE);
        text.draw(draw_target).unwrap();
    }
}

extern "C" fn main() {
    let mut draw_target = EpdDrawTarget::default();
    let mut bad_apple = BAD_APPLE;

    let mut video_time = get_time_micros();

    #[cfg(feature = "timing-debug")]
    let mut prev_time: Option<NonZeroU64> = None;

    while bad_apple.len() > 0 {
        // If playback is ahead, wait
        let mut current_time = get_time_micros();
        while video_time > current_time {
            current_time = get_time_micros();
        }

        // If playback is behind, skip frames as necessary
        while bad_apple.len() > 0 && (current_time.saturating_sub(video_time)) > FRAME_TIME_MICROS {
            let len = u16::from_le_bytes([bad_apple[0], bad_apple[1]]) as usize;
            bad_apple = &bad_apple[len + 2..];
            video_time += FRAME_TIME_MICROS;
        }

        let len = u16::from_le_bytes([bad_apple[0], bad_apple[1]]) as usize;
        // Decompress the next frame directly into the framebuffer
        miniz_oxide::inflate::decompress_slice_iter_to_slice(
            &mut draw_target.framebuffer[BADAPPLE_FRAME_OFFSET..],
            [&bad_apple[2..len + 2]].into_iter(),
            false,
            true,
        ).unwrap();

        #[cfg(feature = "timing-debug")]
        draw_timing_debug(current_time, video_time, prev_time, &mut draw_target);

        // Display the frame
        draw_target.refresh(true);

        #[cfg(feature = "timing-debug")]
        {
            draw_target.clear(BinaryColor::Off).unwrap();
            prev_time = NonZeroU64::new(current_time);
        }

        bad_apple = &bad_apple[len + 2..];
        video_time += FRAME_TIME_MICROS;
    }
}