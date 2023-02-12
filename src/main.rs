use std::{time::{Duration, Instant}, thread, io::stdout};
use std::sync::mpsc;
use std::error::Error;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode}
};
use tui::{
    backend::CrosstermBackend, Terminal,
    layout::{Layout, Direction, Constraint},
    widgets::{Paragraph, Block, Borders, Tabs},
    text::{Spans, Span},
    style::{Style, Color}
};

enum Event<T> {
    Input(T),
    Tick
}

const MENU_ITEMS: [&str; 3] = ["Home", "Commands", "About"];
enum Menu {
    Home,
    Commands,
    About
}

impl From<Menu> for usize {
    fn from(value: Menu) -> Self {
        match value {
            Menu::Home => 0,
            Menu::Commands => 1,
            Menu::About => 2
        }
    }
}

impl From<usize> for Menu {
    fn from(value: usize) -> Self {
        match value {
            0 => Menu::Home,
            1 => Menu::Commands,
            2 => Menu::About
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    enable_raw_mode().expect("enable raw mode");
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let time_rate = Duration::from_millis(2000);
        let mut last_tick = Instant::now();
        loop {
            let timeout = time_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout).expect("Event error") {
                match event::read().expect("can read events") {
                    CEvent::Paste(p) => { println!("Pasted: {}", p); },
                    CEvent::Key(key) => { tx.send(Event::Input(CEvent::Key(key))).expect("send msg"); },
                    _ => {}
                }
            }

            if last_tick.elapsed() > time_rate {
                tx.send(Event::Tick).expect("send tick");
                last_tick = Instant::now()
            }
        }
    });

    let stdout = stdout();
    let backend =  CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut log_texts: Vec<Spans> = Vec::new();
    log_texts.push(Spans::from(Span::styled("Logs", Style::default().fg(Color::Red))));

    loop {
        let mut menu_selected_idx = Menu::Home;
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10)
                ]).split(size);

            let menus = MENU_ITEMS.iter().cloned().map(Spans::from).collect();
            let tabs = Tabs::new(menus)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().bg(Color::Green))
                .divider("|");
            let tabs = tabs.select(menu_selected_idx.into());
            log_texts.push(Spans::from(Span::raw(format!("menuIdx: {}", menu_selected_idx.into()))));
            rect.render_widget(tabs, chunks[0]);

            let b1 = Block::default().borders(Borders::ALL).title("Body");
            rect.render_widget(b1, chunks[1]);

            // let para_text = vec![
            //     Spans::from(vec![Span::raw("1st span"), Span::styled(" 2st span", Style::default().bg(Color::Yellow))])
            // ];

            let para = Paragraph::new(log_texts.clone())
                .alignment(tui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Logs"));
            let para = para.scroll((log_texts.len().try_into().unwrap(),0));
                // .style(tui::style::Style::default().fg(tui::style::Color::Red))
            rect.render_widget(para, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(i) => {
                match i {
                    CEvent::Key(k) => {
                        match k.code {
                            KeyCode::Char('q') => { break; },
                            KeyCode::Char(c) => { println!("K: {}", c.to_string()); }
                            KeyCode::Right => {
                                menu_selected_idx = std::cmp::min(menu_selected_idx.into(), MENU_ITEMS.len()).into();
                            }
                            _ => {}
                        }
                    },
                    CEvent::Paste(p) => { println!("{}", p); },
                    _ => {}
                }
            },
            Event::Tick => {}
        }
    }
    disable_raw_mode().expect("disable raw mode");

    Ok(())
}
