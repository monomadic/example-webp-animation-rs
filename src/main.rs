use std::fs::File;

use image::{codecs::webp::WebPDecoder, AnimationDecoder, DynamicImage, GenericImageView};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let (width, height): (u32, u32) = window.inner_size().into();

    let path: &str = &std::env::args().nth(1).unwrap_or("assets/anim.webp".into());

    let file = File::open(path).unwrap();
    let decoder = WebPDecoder::new(file).unwrap();
    let frames = decoder.into_frames().collect_frames().unwrap();
    let mut frames_iter = frames.into_iter();

    // Initialize Pixels
    let mut pixels = {
        let surface_texture = SurfaceTexture::new(width, height, &window);
        Pixels::new(width, height, surface_texture).unwrap()
    };
    pixels.clear_color(pixels::wgpu::Color::BLACK);

    event_loop.run(move |event, _, control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
            return;
        }

        if let Some(frame) = frames_iter.next() {
            let image = frame.clone().into_buffer();

            window.request_redraw();

            copy_and_resize(&image.into(), &mut pixels, width, height);
            pixels.render().unwrap();
        }
    });
}

/// Copies data to a pixelbuffer, resizing to fit the size of the target buffer.
pub fn copy_and_resize(image: &DynamicImage, pixels: &mut Pixels, width: u32, height: u32) {
    let (image_width, image_height) = image.dimensions();

    let image_aspect_ratio = image_width as f32 / image_height as f32;
    let frame_aspect_ratio = width as f32 / height as f32;

    let (new_width, new_height) = if image_aspect_ratio > frame_aspect_ratio {
        (width, (width as f32 / image_aspect_ratio) as u32)
    } else {
        ((height as f32 * image_aspect_ratio) as u32, height)
    };

    let x_offset = (width - new_width) / 2;
    let y_offset = (height - new_height) / 2;
    let frame_pixels = pixels.frame_mut();

    for y in 0..height {
        for x in 0..width {
            // pixel out of bounds
            if x < x_offset || x >= width - x_offset || y < y_offset || y >= height - y_offset {
                continue;
            }

            let source_x = ((x - x_offset) as f32 * (image_width as f32 / new_width as f32)) as u32;
            let source_y =
                ((y - y_offset) as f32 * (image_height as f32 / new_height as f32)) as u32;

            // Clamp the source_x and source_y values within valid bounds
            let clamped_source_x = source_x.min(image_width - 1);
            let clamped_source_y = source_y.min(image_height - 1);

            let pixel = image.get_pixel(clamped_source_x, clamped_source_y);
            let rgba = pixel.0;

            let position = ((y * width) + x) as usize;
            // Each pixel has 4 channels (RGBA), so we multiply the position by 4.
            frame_pixels[(position * 4)..(position * 4 + 4)].copy_from_slice(&rgba);
        }
    }
}
