use sdl2::mixer::{Chunk, AUDIO_S16LSB, DEFAULT_CHANNELS};

pub struct AudioDevice {
    sounds: Vec<Chunk>,
    pub muted: bool,
    prev_port3: u8,
}

impl AudioDevice {
    pub fn new() -> Self {
        sdl2::mixer::open_audio(44_100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1_024).unwrap();
        sdl2::mixer::allocate_channels(10);

        AudioDevice { sounds: Self::load_sounds(), muted: false, prev_port3: 0 }
    }

    pub fn load_sounds() -> Vec<Chunk> {
        std::fs::read_dir("./sounds/").unwrap().map(|entry| {
            sdl2::mixer::Chunk::from_file(entry.unwrap().path()).unwrap()
        }).collect()
    }

    fn play_sound(&self, num: i32) {
        if !sdl2::mixer::Channel(num).is_playing()
        {
            sdl2::mixer::Channel(num).play(&self.sounds[num as usize], 0).unwrap();
        }
    }

    pub fn mute_unmute(&mut self) {
        if self.muted {
            for i in 0..8 {
                sdl2::mixer::Channel(i).set_volume(128 as i32);
            }
        } else {
            for i in 0..8 {
                sdl2::mixer::Channel(i).set_volume(0);
            }
        }
        self.muted = !self.muted;
    }

    pub fn play(&mut self, port: u8, acc: u8) {
        match port {
            0x3 => {
                if acc & (0x1 << 0) != 0 { // UFO sound
                    self.play_sound(0);
                } else if acc & (0x1 << 1) != 0 && self.prev_port3 & (0x1 << 1) == 0 { // Shot
                    self.play_sound(1);
                } else if acc & (0x1 << 2) != 0 { // Flash (player die)
                    self.play_sound(2);
                } else if acc & (0x1 << 3) != 0 { // Invader die
                    self.play_sound(3);
                }
                self.prev_port3 = acc;
            }
            0x5 => {
                if acc & (0x1 << 0) != 0 { // Fleet movement 1
                    self.play_sound(4);
                } else if acc & (0x1 << 1) != 0 { // Fleet movement 2
                    self.play_sound(5);
                } else if acc & (0x1 << 2) != 0 { // Fleet movement 3
                    self.play_sound(6);
                } else if acc & (0x1 << 3) != 0 { // Fleet movement 4
                    self.play_sound(7);
                } else if acc & (0x1 << 4) != 0 { // UFO Hit
                    self.play_sound(8);
                }
            }
            _ => unreachable!()
        }
    }
}