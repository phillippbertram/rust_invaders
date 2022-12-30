use std::{error::Error, io, sync::mpsc, thread, time::Duration};

use crossterm::{
    cursor::{Hide, Show},
    event,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use invaders::{
    frame::{new_frame, Drawable},
    player::Player,
    render,
};
use rusty_audio::Audio;

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "sounds/explode.wav");
    audio.add("lose", "sounds/lose.wav");
    audio.add("move", "sounds/move.wav");
    audio.add("pew", "sounds/pew.wav");
    audio.add("startup", "sounds/startup.wav");
    audio.add("win", "sounds/win.wav");
    audio.play("startup");

    // Terminal
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    let mut player = Player::new();

    // Gameloop
    'gameloop: loop {
        // per frame init
        let mut current_frame = new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    event::KeyCode::Left => {
                        player.move_left();
                    }

                    event::KeyCode::Right => {
                        player.move_right();
                    }

                    event::KeyCode::Char('q') | event::KeyCode::Esc => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => (),
                }
            }
        }

        // Draw & Render
        player.draw(&mut current_frame);
        let _ = render_tx.send(current_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;

    Ok(())
}
