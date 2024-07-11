use anyhow::Result;

use std::io::Write;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
    QueueableCommand,
    style::{self, Stylize},
    queue,
    execute,
};

mod image;

fn ui_loop<W: Write>(term: &mut W, im: &image::Image) -> Result<()> {
    execute!(term, terminal::Clear(terminal::ClearType::All))?;
    let mut zoom = 1.0;
    let ws = terminal::window_size()?;
    let twidth = ws.columns as usize;
    let theight = ws.rows as usize * 2;
    let (mut iwidth, mut iheight) = im.size(zoom);
    if iwidth > twidth || iheight > theight {
        let z1 = (twidth as f32) / (iwidth as f32);
        let z2 = (theight as f32) / (iheight as f32);
        zoom = if z1 < z2 { z1 } else { z2 };
        let (w, h) = im.size(zoom);
        iwidth = w;
        iheight = h;
    }

    let mut pos = (0, 0);
    let mut offset = (0, 0);

    if iwidth < twidth {
        offset.0 = (twidth - iwidth) / 2;
    }
    if iheight < theight {
        offset.1 = (theight - iheight) / 4;
    }

    loop {
        im.draw(term, pos, offset, zoom)?;
        term.flush()?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    } else if key.code == KeyCode::Char('+') || key.code == KeyCode::Char('=') {
                        zoom += 0.01;
                    } else if key.code == KeyCode::Char('-') || key.code == KeyCode::Char('_') {
                        zoom -= 0.01;
                        if zoom < 0.01 {
                            zoom = 0.01;
                        }
                    } else if key.code == KeyCode::Char('h') || key.code == KeyCode::Char('a') {
                        if pos.0 > 0 {
                            pos.0 -= 1;
                        }
                    } else if key.code == KeyCode::Char('l') || key.code == KeyCode::Char('d') {
                        pos.0 += 1;
                    } else if key.code == KeyCode::Char('k') || key.code == KeyCode::Char('w') {
                        pos.1 += 1;
                    } else if key.code == KeyCode::Char('j') || key.code == KeyCode::Char('s') {
                        if pos.1 > 0 {
                            pos.1 -= 1;
                        }
                    } else if key.code == KeyCode::Char(' ') {
                        zoom = 1.0;
                        offset = (0, 0);
                        pos = (0, 0);
                        let (iwidth, iheight) = im.size(zoom);
                        if iwidth > twidth || iheight > theight {
                            let z1 = (twidth as f32) / (iwidth as f32);
                            let z2 = (theight as f32) / (iheight as f32);
                            zoom = if z1 < z2 { z1 } else { z2 };
                        }
                    }
                }
            },
            _ => {},
        }

        let ws = terminal::window_size()?;
        let twidth = ws.columns as usize;
        let theight = ws.rows as usize * 2;
        let (mut iwidth, mut iheight) = im.size(zoom);

        if iwidth < twidth {
            pos.0 = 0;
            offset.0 = (twidth - iwidth) / 2;
        }
        if iheight < theight {
            pos.1 = 0;
            offset.1 = (theight - iheight) / 4;
        }
    }

    Ok(())
}

fn init_panic_hook() {
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_tui();
        orig_hook(panic_info);
    }));
}

fn init_tui() -> Result<()> {
    let mut stdout = std::io::stdout();

    execute!(stdout, terminal::EnterAlternateScreen)?;

    if let Err(e) = terminal::enable_raw_mode() {
        let _ = execute!(stdout, terminal::LeaveAlternateScreen);
        return Err(e.into());
    }

    if let Err(e) = execute!(stdout, cursor::Hide) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout, terminal::LeaveAlternateScreen);
        return Err(e.into());
    }

    init_panic_hook();

    Ok(())
}

fn restore_tui() -> Result<()> {
    let mut stdout = std::io::stdout();

    if let Err(e) = execute!(stdout, cursor::Show) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout, terminal::LeaveAlternateScreen);
        return Err(e.into());
    }

    if let Err(e) = terminal::disable_raw_mode() {
        let _ = execute!(stdout, terminal::LeaveAlternateScreen);
        return Err(e.into());
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;

    Ok(())
}

fn ui(im: &image::Image) -> Result<()> {
    init_tui()?;

    if let Err(e) = ui_loop(&mut std::io::stdout(), im) {
        let _ = restore_tui();
        return Err(e);
    }

    restore_tui()
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return Ok(())
    }
    let im = image::Image::open(&args[1])?;
    ui(&im)
}
