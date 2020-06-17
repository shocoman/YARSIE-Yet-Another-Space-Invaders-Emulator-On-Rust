mod sdl_context;
mod shift_register;
mod controls;
mod screen;
mod audio;
mod bus;
mod i8080;


/*
Controls:
    Move left           - Left Arrow
    Move right          - Right Arrow
    Shoot               - Space
    One player start    - Key 1
    Two players start   - Key 2
    Toggle extra live at 1000 points (otherway on 1500p) - X

    Start lives         - Keys 3-6 (from 3 to 6 lives)
    Tilt                - Key T
    Reset               - Key R

    Save game state     - Key F1
    Load saved state    - Key F2
    Mute/Unmute sound   - Key M
*/


fn main() -> std::io::Result<()> {
    let mut emul = bus::MainBus::new();
    emul.run()?;

    Ok(())
}