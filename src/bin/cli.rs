use clap::{arg, command, Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        KeyboardEnhancementFlags as KeyFlag, PushKeyboardEnhancementFlags as PushKeyFlag,
    },
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
    display::{HEIGHT, WIDTH},
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
    Press(u8),
    Release(u8),
    Exit,
}

impl KeyboardEvent {
    fn from_key_event(event: KeyEvent) -> Option<Self> {
        if let (KeyModifiers::CONTROL, KeyCode::Char('q') | KeyCode::Char('c')) =
            (event.modifiers, event.code)
        {
            return Some(Self::Exit);
        }

        let key = match event.code {
            KeyCode::Char('1') => Some(0x1),
            KeyCode::Char('2') => Some(0x2),
            KeyCode::Char('3') => Some(0x3),
            KeyCode::Char('4') => Some(0xC),
            KeyCode::Char('q') => Some(0x4),
            KeyCode::Char('w') => Some(0x5),
            KeyCode::Char('e') => Some(0x6),
            KeyCode::Char('r') => Some(0xD),
            KeyCode::Char('a') => Some(0x7),
            KeyCode::Char('s') => Some(0x8),
            KeyCode::Char('d') => Some(0x9),
            KeyCode::Char('f') => Some(0xE),
            KeyCode::Char('z') => Some(0xA),
            KeyCode::Char('x') => Some(0x0),
            KeyCode::Char('c') => Some(0xB),
            KeyCode::Char('v') => Some(0xF),
            _ => None,
        };

        key.map(|key| match event.kind {
            KeyEventKind::Release => Self::Release(key),
            _ => Self::Press(key),
        })
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

            if chip8.display.has_changed() {
                let buffer = chip8.display.get_buffer();
                self.draw_buffer(buffer)?;
            }

            if let Some(event) = self.read_key()? {
                match event {
                    KeyboardEvent::Press(key) => chip8.keyboard.set_key(key),
                    KeyboardEvent::Release(key) => chip8.keyboard.unset_key(key),
                    KeyboardEvent::Exit => break,
                }
            }
        }

        cleanup()
    }

    fn draw_buffer(&self, buffer: &[u8]) -> Result<(), Error> {
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
            .queue(terminal::Clear(ClearType::All))?
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
        .queue(PushKeyFlag(KeyFlag::REPORT_EVENT_TYPES))?
        .queue(PushKeyFlag(KeyFlag::REPORT_ALL_KEYS_AS_ESCAPE_CODES))?
        .queue(PushKeyFlag(KeyFlag::DISAMBIGUATE_ESCAPE_CODES))?
        .queue(terminal::EnterAlternateScreen)?
        .queue(cursor::Hide)?;

    terminal::enable_raw_mode()
}

fn cleanup() -> Result<(), Error> {
    stdout()
        .queue(event::PopKeyboardEnhancementFlags)?
        .queue(event::PopKeyboardEnhancementFlags)?
        .queue(event::PopKeyboardEnhancementFlags)?
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
