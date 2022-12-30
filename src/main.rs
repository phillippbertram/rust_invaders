use std::{
    error::Error,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use invaders::{
    frame::{new_frame, Drawable},
    invaders::Invaders,
    player::Player,
    render,
};
use rusty_audio::Audio;

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    let audio_profile = "phillipp";
    let audio_files = vec!["explode", "lose", "move", "pew", "startup", "win"];

    for file in audio_files {
        audio.add(file, format!("sounds/{}/{}.wav", audio_profile, file));
    }

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
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    // Gameloop
    'gameloop: loop {
        // delta
        let delta = instant.elapsed();
        instant = Instant::now();

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

                    event::KeyCode::Char(' ') | event::KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }

                    event::KeyCode::Char('q') | event::KeyCode::Esc => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => (),
                }
            }
        }

        // Updates
        player.update(delta);
        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }
        if invaders.update(delta) {
            audio.play("move");
        }

        // Draw & Render
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables.iter() {
            drawable.draw(&mut current_frame);
        }
        let _ = render_tx.send(current_frame);
        thread::sleep(Duration::from_millis(1));

        // win or loose
        if invaders.all_dead() {
            audio.play("win");
            break 'gameloop;
        }

        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    // cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;

    Ok(())
}
