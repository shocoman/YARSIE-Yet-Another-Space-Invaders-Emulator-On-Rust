use sdl2::render::{Texture, WindowCanvas};
use sdl2::rect::{Rect, Point};

pub struct ScreenDevice {}

impl ScreenDevice {
    pub fn new() -> Self {
        ScreenDevice{}
    }

    pub fn draw(&self, texture: &mut Texture, canvas: &mut WindowCanvas, memory: &[u8; 0x10000]) -> Result<(), String> {
        // 256x224 (32 bytes width)
        let gfx_start: usize = 0x2400;

        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {

            for width in 0..256 {
                for height in 0..224 {

                    let width_byte = width / 8;
                    let width_bit = width % 8;

                    let byte = memory[gfx_start + 32 * height + width_byte];

                    let offset = pitch * width + height * 3;
                    if byte >> width_bit as u8 & 0x1 == 0x1 {

                        if (205..223).contains(&width) { // red flying sausage
                            buffer[offset..=offset+2].copy_from_slice(&[255,0,0])
                        } else if (16..72).contains(&width) || width < 16 && (20..112).contains(&height)  { // green player and shields
                            buffer[offset..=offset+2].copy_from_slice(&[0,255,0])
                        } else { // white everything else including ALIENS
                            buffer[offset..=offset+2].copy_from_slice(&[255,255,255])
                        }

                    } else { // black background
                        buffer[offset..=offset+2].copy_from_slice(&[0,0,0])
                    }
                }
            }
        })?;

        let size = canvas.output_size().unwrap();
        let dst_rect = Rect::new(0, 0, size.0, size.1);
        let new_rect = Rect::new(0,0, size.1, size.0);
        let cnt = Point::new((size.0 / 2) as i32, (size.1 / 2) as i32);
        // let cnt = Point::new(200, 200);

        canvas.copy_ex(&texture, None, None, 0.0, None, false, true)?;
        Ok(())
    }
}