use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Texture;
use std::time::Duration;

pub struct SdlContext {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub canvas: sdl2::render::WindowCanvas,
    pub event_pump: sdl2::EventPump,
    pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

impl SdlContext {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();

        // get monitor size
        let monitor_size = sdl_context.video().unwrap().current_display_mode(0).unwrap();
        let win_size = monitor_size.w.min(monitor_size.h) as u32 - 100;

        let video_subsystem = sdl_context.video().unwrap();
        let mut window = video_subsystem
            .window("", win_size * 224/256, win_size)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();
        canvas.set_draw_color(Color::BLACK);
        let mut event_pump = sdl_context.event_pump().unwrap();
        let texture_creator = canvas.texture_creator();

        canvas.set_draw_color(Color::BLUE);

        SdlContext { sdl_context, video_subsystem, canvas, event_pump, texture_creator}
    }

    pub fn sleep_for(&self, elapsed: Duration, fps: f64) {
        std::thread::sleep(Duration::from_secs_f64(1.0 / fps)
            .checked_sub(elapsed).unwrap_or(Duration::from_millis(0)));
    }

}