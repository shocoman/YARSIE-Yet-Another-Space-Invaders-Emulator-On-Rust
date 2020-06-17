use crate::{
    shift_register::ShiftRegister,
    screen::ScreenDevice,
    audio::AudioDevice,
    controls::{Controls, EmulatorAction},
    sdl_context::SdlContext,
    i8080::I8080
};
use std::fs::File;
use std::io::{Error, Read};
use std::time::{Instant, Duration};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;


pub fn load_si_rom() -> Result<Vec<u8>, Error> {
    let rom_h: Vec<_> = File::open("./rom/invaders.h")?.bytes().collect();
    let rom_g: Vec<_> = File::open("./rom/invaders.g")?.bytes().collect();
    let rom_f: Vec<_> = File::open("./rom/invaders.f")?.bytes().collect();
    let rom_e: Vec<_> = File::open("./rom/invaders.e")?.bytes().collect();

    let rom = rom_h
        .iter()
        .chain(rom_g.iter())
        .chain(rom_f.iter())
        .chain(rom_e.iter())
        .flatten()
        .copied()
        .collect::<Vec<u8>>();

    Ok(rom)
}


pub struct MainBus{
    audio: AudioDevice,
    screen: ScreenDevice,
    cpu: I8080,
    shift_register: ShiftRegister,
    controls: Controls,
}

impl MainBus {
    pub fn new() -> Self {
        MainBus {
            audio: AudioDevice::new(),
            screen: ScreenDevice::new(),
            cpu: I8080::new(),
            shift_register: ShiftRegister::new(),
            controls: Controls::new(),
        }
    }


    pub fn run(&mut self) -> std::io::Result<()> {
        self.cpu.load_rom(&load_si_rom()?, 0x0);

        let mut sdl_context = SdlContext::new();
        let mut screen_texture = sdl_context.texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 224,256 ).unwrap();

        let mut emulator_save_state = [0_u8; 0x2000];
        let mut fps = 60.0;
        let mut clock_rate: u32 = 2_000_000;
        'running: loop {
            let start = Instant::now();
            sdl_context.canvas.clear();

            let action = self.controls.send_input(&mut sdl_context.event_pump);
            match action {
                EmulatorAction::Nothing => {},
                EmulatorAction::Quit => break 'running,
                EmulatorAction::SaveState => emulator_save_state.copy_from_slice(self.cpu.memory[0x2000..0x4000].as_ref()),
                EmulatorAction::LoadState => self.cpu.memory[0x2000..0x4000].copy_from_slice(emulator_save_state[..].as_ref()),
                EmulatorAction::IncreaseFPS => {clock_rate += 100_000; fps += 5.0;},
                EmulatorAction::DecreaseFPS => {
                    if clock_rate > 100_000 && fps > 5.0 {
                        fps -= 5.0;
                        clock_rate -= 100_000;
                    }
                },
                EmulatorAction::Reset => for i in &mut self.cpu.memory[0x2000..0x4000] { *i = 0; },
                EmulatorAction::Mute => self.audio.mute_unmute(),
            }

            self.execute_n_cycles((0.5 * clock_rate as f64 / fps) as usize);
            self.cpu.generate_interrupt(1);

            self.execute_n_cycles((0.5 * clock_rate as f64 / fps) as usize);
            self.cpu.generate_interrupt(2);

            self.screen.draw(&mut screen_texture, &mut sdl_context.canvas, &self.cpu.memory).unwrap();
            sdl_context.canvas.present();

            sdl_context.sleep_for(start.elapsed(), fps);

            // display some info in windows title
            sdl_context.canvas.window_mut().set_title(
                format!("Space Invaders Emulator. FPS: {:.2}; Clock rate: {} \
                        Start lives: {}; Extra ship: {}; Muted: {}", 1000.0 / start.elapsed().as_millis() as f64,
                        clock_rate, self.controls.lives, self.controls.extra_ship, self.audio.muted).as_ref()).unwrap();
        }

        Ok(())
    }

    fn execute_n_cycles(&mut self, n: usize) -> Option<()> {
        let mut current_rate = 0;
        while current_rate < n {
            let instr = self.cpu.read_instr()?;
            self.intercept_instr(instr);

            let len = self.cpu.execute(instr);
            current_rate += len;
        }
        Some(())
    }

    fn intercept_instr(&mut self, instr: u8) {
        match instr {
            0xd3 => self.write_port(self.cpu.read_memory(self.cpu.pc + 1), self.cpu.a),
            0xdb => self.cpu.a = self.read_port(self.cpu.read_memory(self.cpu.pc + 1)),
            _ => {}
        }
    }

    pub fn read_port(&self, port: u8) -> u8 {
        match port {
            0 => self.controls.read_controls(port), //
            1 => self.controls.read_controls(port), // 1st player
            2 => self.controls.read_controls(port), // 2nd player
            3 => self.shift_register.read_value(),
            _ => unreachable!()
        }
    }

    pub fn write_port(&mut self, port: u8, acc: u8) {
        match port {
            2 => self.shift_register.set_shift_amount(acc),
            3 => self.audio.play(port, acc), // discrete sounds
            4 => self.shift_register.put_value(acc),
            5 => self.audio.play(port, acc),     // another sound
            6 => { } // watch-dog timer?
            _ => unreachable!()
        }
    }
}

