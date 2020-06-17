use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl2::EventPump;
use std::process::exit;

pub enum EmulatorAction {
    Nothing,
    Quit,
    SaveState,
    LoadState,
    IncreaseFPS,
    DecreaseFPS,
    Reset,
    Mute
}


#[derive(Default)]
pub struct Controls {
    p1: bool,
    p2: bool,
    fire: bool,
    left: bool,
    right: bool,
    coin_slot: bool,
    tilt: bool, // game over and game reset

    pub lives: u8, // from 3 to 6
    pub extra_ship: bool, // if true: extra ship at 1000 points, else: at 1500p
}

impl Controls {
    pub fn new() -> Self {
        Controls {
            lives: 3,
            ..Controls::default()
        }

    }

    pub fn send_input(&mut self, event_pump: &mut EventPump) -> EmulatorAction {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return EmulatorAction::Quit,

                Event::KeyDown { keycode: Some(Keycode::Space), .. } => self.fire = true,
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => self.fire = false,

                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => self.p1 = true,
                Event::KeyUp { keycode: Some(Keycode::Num1), .. } => self.p1 = false,

                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => self.p2 = true,
                Event::KeyUp { keycode: Some(Keycode::Num2), .. } => self.p2 = false,

                Event::KeyDown { keycode: Some(Keycode::Left), .. } => self.left = true,
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => self.left = false,

                Event::KeyDown { keycode: Some(Keycode::Right), .. } => self.right = true,
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => self.right = false,

                Event::KeyDown { keycode: Some(Keycode::C), .. } => self.coin_slot = true,
                Event::KeyUp { keycode: Some(Keycode::C), .. } => self.coin_slot = false,

                Event::KeyDown { keycode: Some(Keycode::T), .. } => self.tilt = true,
                Event::KeyUp { keycode: Some(Keycode::T), .. } => self.tilt = false,

                Event::KeyDown { keycode: Some(Keycode::F1), .. } => return EmulatorAction::SaveState,
                Event::KeyDown { keycode: Some(Keycode::F2), .. } => return EmulatorAction::LoadState,
                Event::KeyDown { keycode: Some(Keycode::KpPlus), .. } => return EmulatorAction::IncreaseFPS,
                Event::KeyDown { keycode: Some(Keycode::KpMinus), .. } => return EmulatorAction::DecreaseFPS,
                Event::KeyDown { keycode: Some(Keycode::R), .. } => return EmulatorAction::Reset,
                Event::KeyDown { keycode: Some(Keycode::M), .. } => return EmulatorAction::Mute,

                Event::KeyDown { keycode: Some(Keycode::Num3), .. } => self.lives = 3,
                Event::KeyDown { keycode: Some(Keycode::Num4), .. } => self.lives = 4,
                Event::KeyDown { keycode: Some(Keycode::Num5), .. } => self.lives = 5,
                Event::KeyDown { keycode: Some(Keycode::Num6), .. } => self.lives = 6,

                Event::KeyDown { keycode: Some(Keycode::X), .. } => self.extra_ship = !self.extra_ship,
                _ => {}
            }
        }

        EmulatorAction::Nothing
    }


    pub fn read_controls(&self, port: u8) -> u8 {
        match port {
            0 => {
                let fire_bit = (self.fire as u8) << 4;
                let left_bit = (self.left as u8) << 5;
                let right_bit = (self.right as u8) << 6;
                return right_bit | left_bit | fire_bit | 0b1111;
            }
            1 => {
                let put_coin = (self.coin_slot as u8) << 0;
                let p2_start = (self.p2 as u8) << 1;
                let p1_start = (self.p1 as u8) << 2;
                let always_true_bit = 1 << 3;
                let p1_shot = (self.fire as u8) << 4;
                let p1_left = (self.left as u8) << 5;
                let p1_right = (self.right as u8) << 6;
                return p1_right | p1_left | p1_shot | p2_start | p1_start | put_coin | always_true_bit ;
            }
            2 => {
                let lives = (self.lives - 3) & 0b11;
                let tilt = (self.tilt as u8) << 2;
                let extra_ship_at_1000 = (self.extra_ship as u8) << 3;

                let p1_shot = (self.fire as u8) << 4;
                let p1_left = (self.left as u8) << 5;
                let p1_right = (self.right as u8) << 6;
                return p1_right | p1_left | p1_shot | extra_ship_at_1000 | tilt | lives;
            }
            _ => unreachable!()
        }
    }
}

