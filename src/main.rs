mod cpu_state;

use cpu_state::ComputerState;
use macroquad::{
    audio::{load_sound_from_bytes, play_sound, play_sound_once, stop_sound, PlaySoundParams},
    prelude::*,
};

fn get_curr_keys() -> u16 {
    const KEY_MAPPING: [u16; 16] = [
        1 << 0x1,
        1 << 0x2,
        1 << 0x3,
        1 << 0xC,
        1 << 0x4,
        1 << 0x5,
        1 << 0x6,
        1 << 0xD,
        1 << 0x7,
        1 << 0x8,
        1 << 0x9,
        1 << 0xE,
        1 << 0xA,
        1 << 0x0,
        1 << 0xB,
        1 << 0xF,
    ];
    let mut pressed_keys: u16 = 0;
    if is_key_down(KeyCode::Key1) {
        pressed_keys |= KEY_MAPPING[0];
    }
    if is_key_down(KeyCode::Key2) {
        pressed_keys |= KEY_MAPPING[1];
    }
    if is_key_down(KeyCode::Key3) {
        pressed_keys |= KEY_MAPPING[2];
    }
    if is_key_down(KeyCode::Key4) {
        pressed_keys |= KEY_MAPPING[3];
    }
    if is_key_down(KeyCode::Q) {
        pressed_keys |= KEY_MAPPING[4];
    }
    if is_key_down(KeyCode::W) {
        pressed_keys |= KEY_MAPPING[5];
    }
    if is_key_down(KeyCode::E) {
        pressed_keys |= KEY_MAPPING[6];
    }
    if is_key_down(KeyCode::R) {
        pressed_keys |= KEY_MAPPING[7];
    }
    if is_key_down(KeyCode::A) {
        pressed_keys |= KEY_MAPPING[8];
    }
    if is_key_down(KeyCode::S) {
        pressed_keys |= KEY_MAPPING[9];
    }
    if is_key_down(KeyCode::D) {
        pressed_keys |= KEY_MAPPING[10];
    }
    if is_key_down(KeyCode::F) {
        pressed_keys |= KEY_MAPPING[11];
    }
    if is_key_down(KeyCode::Z) {
        pressed_keys |= KEY_MAPPING[12];
    }
    if is_key_down(KeyCode::X) {
        pressed_keys |= KEY_MAPPING[13];
    }
    if is_key_down(KeyCode::C) {
        pressed_keys |= KEY_MAPPING[14];
    }
    if is_key_down(KeyCode::V) {
        pressed_keys |= KEY_MAPPING[15];
    }

    pressed_keys
}

#[macroquad::main("CHIP8")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let clock_speed: u32 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .expect("pass in CPU clock speed Hz as second arg");

    let rom_file_path = &args.get(1).expect("need to provide rom file name");
    let rom = std::fs::read(rom_file_path).expect("ROM file not found");
    println!("ROM loaded: {} bytes", rom.len());

    let beep_file = include_bytes!("../sawtooth.wav");
    let beep = load_sound_from_bytes(beep_file)
        .await
        .expect("couldn't load beep sound");
    let mut computer = ComputerState::new(&rom);

    let mut curr_beeping = false;
    clear_background(BLACK);
    loop {
        if is_quit_requested() || is_key_released(KeyCode::Escape) {
            break;
        }

        let pixel_w = screen_width() / 64.0;
        let pixel_h = screen_height() / 32.0;

        let key_presses = get_curr_keys();
        computer.set_pressed_keys(key_presses);

        computer.advance_tick(clock_speed / 60);

        clear_background(BLACK);

        for y in 0..32 {
            for x in 0..64 {
                let display_on = (computer.display[y] >> (63 - x)) & 1 == 1;
                draw_rectangle(
                    pixel_w * x as f32,
                    pixel_h * y as f32,
                    pixel_w,
                    pixel_h,
                    if display_on {
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }
                    } else {
                        Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.1,
                        }
                    },
                );
            }
        }

        if computer.is_beeping() && !curr_beeping {
            play_sound(
                &beep,
                PlaySoundParams {
                    looped: true,
                    volume: 0.2,
                },
            );
            curr_beeping = true;
        } else if !computer.is_beeping() && curr_beeping {
            stop_sound(&beep);
            curr_beeping = false;
        }

        next_frame().await
    }
}
