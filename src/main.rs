use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use opencv::videoio::VideoCaptureTrait;
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    widgets::{
        canvas::{Canvas, Rectangle},
        Block, Borders,
    },
    Frame, Terminal,
};

use video2cmd::{image, opencv_wrapper};
use video2cmd::{HEIGHT, IMAGE_HEIGHT, IMAGE_WIDTH, WIDTH};

struct App {
    playground: Rect,
    vx: f64,
    vy: f64,
    dir_x: bool,
    dir_y: bool,
}

impl App {
    fn new() -> App {
        App {
            playground: Rect::new(10, 10, 100, 100),
            vx: 1.0,
            vy: 1.0,
            dir_x: true,
            dir_y: true,
        }
    }
}

fn create_map(width: usize, height: usize) -> Vec<(f64, f64)> {
    let mut maps = vec![];

    for y in 0..height {
        for x in 0..width {
            maps.push((x as f64, y as f64));
        }
    }
    maps
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut cam = opencv_wrapper::get_cam()?;

    let image_map = create_map(IMAGE_WIDTH, IMAGE_HEIGHT);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(10);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate, &mut cam, image_map);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: App,
    tick_rate: Duration,
    cam: &mut impl VideoCaptureTrait,
    image_map: Vec<(f64, f64)>,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app, cam, &image_map))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(
    f: &mut Frame<B>,
    _app: &App,
    cam: &mut impl VideoCaptureTrait,
    image_map: &Vec<(f64, f64)>,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let image = image::Image::new_from_cam(
        cam,
        image_map.clone(),
        IMAGE_HEIGHT as i32,
        IMAGE_WIDTH as i32,
    )
    .unwrap();

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Video Capture"),
        )
        .marker(tui::symbols::Marker::Dot)
        .paint(|ctx| {
            ctx.draw(&image);
        })
        .x_bounds([-1.0 * (WIDTH / 2.0), WIDTH / 2.0])
        .y_bounds([-1.0 * (HEIGHT / 2.0), HEIGHT / 2.0]);
    f.render_widget(canvas, chunks[0]);
}
