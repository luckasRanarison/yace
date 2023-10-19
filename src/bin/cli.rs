use clap::{arg, command, Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{self, Color, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand,
};
use std::{
    fs,
    io::{stdout, Error, ErrorKind, Write},
    panic::{self, PanicInfo},
    time::Duration,
};
use yace::{
    chip::Chip,
    display::{DisplayChange, HEIGHT, WIDTH},
    keyboard::Key,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Symbol used for pixels
    #[arg(short, long, default_value = "█")]
    pixel: char,

    /// Foreground color
    #[arg(short, long, default_value = "green")]
    fg: PixelColor,

    /// Background color
    #[arg(short, long, default_value = "black")]
    bg: PixelColor,

    /// Clock speed
    #[arg(short, long, default_value = "500")]
    clock: u64,

    /// Steps per cycle
    #[arg(short, long, default_value = "10")]
    steps: u8,

    /// ROM file path
    path: String,
}

#[derive(ValueEnum, Clone)]
enum PixelColor {
    Green,
    Red,
    Blue,
    Cyan,
    Yelllow,
    Purple,
    Grey,
    Black,
    White,
}

impl PixelColor {
    fn to_color(&self) -> Color {
        match self {
            PixelColor::Green => Color::Green,
            PixelColor::Red => Color::Red,
            PixelColor::Blue => Color::Blue,
            PixelColor::Cyan => Color::Cyan,
            PixelColor::Yelllow => Color::Yellow,
            PixelColor::Purple => Color::Magenta,
            PixelColor::Black => Color::Black,
            PixelColor::Grey => Color::Grey,
            PixelColor::White => Color::White,
        }
    }
}

#[derive(Debug)]
enum KeyboardEvent {
    Press(Key),
    Release,
    Exit,
}

impl KeyboardEvent {
    fn from_key_event(event: KeyEvent) -> Option<Self> {
        if let (KeyModifiers::CONTROL, KeyCode::Char('q') | KeyCode::Char('c')) =
            (event.modifiers, event.code)
        {
            return Some(Self::Exit);
        }

        if event.kind == KeyEventKind::Release {
            return Some(Self::Release);
        }

        let key = match event.code {
            KeyCode::Char('1') => Some(Key::K1),
            KeyCode::Char('2') => Some(Key::K2),
            KeyCode::Char('3') => Some(Key::K3),
            KeyCode::Char('4') => Some(Key::KC),
            KeyCode::Char('q') => Some(Key::K4),
            KeyCode::Char('w') => Some(Key::K5),
            KeyCode::Char('e') => Some(Key::K6),
            KeyCode::Char('r') => Some(Key::KD),
            KeyCode::Char('a') => Some(Key::K7),
            KeyCode::Char('s') => Some(Key::K8),
            KeyCode::Char('d') => Some(Key::K9),
            KeyCode::Char('f') => Some(Key::KE),
            KeyCode::Char('z') => Some(Key::KA),
            KeyCode::Char('x') => Some(Key::K0),
            KeyCode::Char('c') => Some(Key::KB),
            KeyCode::Char('v') => Some(Key::KF),
            _ => None,
        };

        key.map(|key| Self::Press(key))
    }
}

impl Cli {
    fn run(&self) -> Result<(), Error> {
        let bytes = fs::read(&self.path)?;
        let mut chip8 = Chip::new(&bytes);
        let mut cycle_timer = 0;

        init_screen()?;

        loop {
            chip8.tick();

            cycle_timer += 1;
            cycle_timer %= self.steps;

            if cycle_timer == 0 {
                chip8.update_timers();
            }

            if let Some(changes) = chip8.display.get_changes() {
                let buffer = chip8.display.get_buffer();

                self.draw_buffer(buffer, changes)?;
            }

            if let Some(event) = self.read_key()? {
                match event {
                    KeyboardEvent::Press(key) => chip8.keyboard.set_key(key),
                    KeyboardEvent::Release => chip8.keyboard.unset_key(),
                    KeyboardEvent::Exit => break,
                }
            }
        }

        cleanup()
    }

    fn draw_buffer(&self, buffer: &[u8], changes: &DisplayChange) -> Result<(), Error> {
        let x = changes.x as u16;
        let y = changes.y as u16;
        let buffer = buffer
            .into_iter()
            .enumerate()
            .map(|(i, &pixel)| {
                let color = if pixel == 1 { &self.fg } else { &self.bg };
                let fg = SetForegroundColor(color.to_color());
                let new_line = (i + 1) % WIDTH == 0 && (i + 1) != WIDTH * HEIGHT;
                let end = if new_line { "\r\n" } else { "" };

                format!("{}{}{}", fg, self.pixel, end)
            })
            .collect::<String>();

        stdout()
            .queue(cursor::MoveTo(x, y))?
            .queue(terminal::Clear(ClearType::FromCursorUp))?
            .queue(cursor::MoveTo(0, 0))?
            .queue(style::Print(buffer))?
            .flush()
    }

    fn read_key(&self) -> Result<Option<KeyboardEvent>, Error> {
        if event::poll(Duration::from_millis(1000 / self.clock))? {
            Ok(match event::read()? {
                Event::Key(event) => KeyboardEvent::from_key_event(event),
                _ => None,
            })
        } else {
            Ok(None)
        }
    }
}

fn init_screen() -> Result<(), Error> {
    let (width, height) = terminal::size()?;

    if (width as usize) < WIDTH || (height as usize) < HEIGHT {
        let message = format!("The required terminal size is {}x{}", WIDTH, HEIGHT);

        return Err(Error::new(ErrorKind::Other, message));
    }

    panic::set_hook(Box::new(|info: &PanicInfo| {
        println!("{:?}", info);
        cleanup().unwrap();
    }));

    stdout()
        .queue(terminal::EnterAlternateScreen)?
        .queue(cursor::Hide)?;

    terminal::enable_raw_mode()
}

fn cleanup() -> Result<(), Error> {
    stdout()
        .queue(terminal::LeaveAlternateScreen)?
        .queue(cursor::Show)?;

    terminal::disable_raw_mode()
}

fn main() {
    let cli = Cli::parse();

    if let Err(error) = cli.run() {
        println!("error: {}", error);
    }
}
