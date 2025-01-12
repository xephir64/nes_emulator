use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;

use bus::Bus;
use cpu::CPU;

use frame::Frame;
use ppu::NesPPU;
use rom::Rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

pub mod apu;
pub mod apu_channels;
pub mod bus;
pub mod cpu;
pub mod frame;
pub mod interrupt;
pub mod joypad;
pub mod opcode;
pub mod palette;
pub mod ppu;
pub mod ppu_registers;
pub mod render;
pub mod rom;
pub mod trace;

extern crate lazy_static;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("NES Emulator", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    // load rom
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No ROM specified");
        return;
    }

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

    let rom_path = &args[1];
    let mut rom = File::open(&rom_path).expect("Cannot open ROM");
    let mut rom_buffer = Vec::new();
    rom.read_to_end(&mut rom_buffer).unwrap();

    // load the game
    let rom = Rom::new(&rom_buffer).unwrap();
    let mut frame = Frame::new();

    // the game cycle
    let bus = Bus::new(rom, move |ppu: &NesPPU, joypad: &mut joypad::Joypad| {
        render::render(ppu, &mut frame);
        texture.update(None, &frame.data, 256 * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();

        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, false);
                    }
                }
                _ => { /* do nothing */ }
            }
        }
    });

    let mut cpu = CPU::new(bus);

    cpu.reset();
    cpu.run();

    /*cpu.run_with_callback(move |cpu| {
        println!("{}", trace::trace(cpu));
        // ::std::thread::sleep(std::time::Duration::new(0, 70_000));
    });*/
}
