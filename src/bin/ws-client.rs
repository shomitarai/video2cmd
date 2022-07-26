extern crate env_logger;
extern crate ws;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use std::{
    error::Error,
    io,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{canvas::Canvas, Block, Borders},
    Frame, Terminal,
};

use video2cmd::{image::SharedImage, opencv_wrapper, HEIGHT, IMAGE_HEIGHT, IMAGE_WIDTH, WIDTH};
use ws::connect;

fn create_map(width: usize, height: usize) -> Vec<(f64, f64)> {
    let mut maps = vec![];

    for y in 0..height {
        for x in 0..width {
            maps.push((x as f64, y as f64));
        }
    }
    maps
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

lazy_static! {
    pub static ref IMAGE: Mutex<SharedImage> = Mutex::new(SharedImage::default(
        IMAGE_HEIGHT as i32,
        IMAGE_WIDTH as i32,
        create_map(IMAGE_WIDTH, IMAGE_HEIGHT),
    ));
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, tick_rate: Duration, tx: Sender<i32>) {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f)).unwrap();

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if let KeyCode::Char('q') = key.code {
                    tx.send(1).unwrap();
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());
    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Video Capture"),
        )
        .marker(tui::symbols::Marker::Dot)
        .paint(|ctx| {
            ctx.draw(IMAGE.lock().unwrap().image());
        })
        .x_bounds([-1.0 * (WIDTH / 2.0), WIDTH / 2.0])
        .y_bounds([-1.0 * (HEIGHT / 2.0), HEIGHT / 2.0]);
    f.render_widget(canvas, chunks[0]);
}

fn run_tui(tx: Sender<i32>) {
    // setup terminal
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let tick_rate = Duration::from_millis(10);

    run_app(&mut terminal, tick_rate, tx);

    // restore terminal
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();
}

fn send_video_image(out: ws::Sender, rx: Receiver<i32>) {
    let mut cam = opencv_wrapper::get_cam().unwrap();
    loop {
        let v = opencv_wrapper::get_vec8(&mut cam).unwrap();
        if out.send(v).is_err() {
            out.close(ws::CloseCode::Normal).unwrap();
            break;
        };

        if rx.recv_timeout(Duration::from_millis(5)).is_ok() {
            out.close(ws::CloseCode::Normal).unwrap();
        }

        thread::sleep(Duration::from_millis(5));
    }
}

use std::sync::mpsc;

fn main() -> Result<()> {
    env_logger::init();

    if let Err(error) = connect("ws://127.0.0.1:3012", |out| {
        // if let Err(error) = connect("ws://127.0.0.1:3012", |out| {
        let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

        thread::spawn(move || {
            run_tui(tx);
        });

        let sender = out;
        thread::spawn(move || send_video_image(sender, rx));

        move |msg| match msg {
            ws::Message::Text(_) => Ok(()),
            ws::Message::Binary(v) => {
                IMAGE.lock().unwrap().update(v);
                Ok(())
            }
        }
    }) {
        println!("Failed to create WebSocket due to {:?}", &error);
    }
    Ok(())
}
