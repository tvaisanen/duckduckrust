#![allow(dead_code)]

use reqwest;
use scraper::{Html, Selector};
use std::io::Write;
use std::{env, error, fs, io};

use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, Widget};
use tui::{Frame, Terminal};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Clone, Debug)]
struct SearchResult {
    href: String,
    summary: String,
}

const DUCKDUCKGO: &str = "https://html.duckduckgo.com/html?q=";

#[derive(Clone, Debug)]
struct App {
    query_string: String,
    search_results: Vec<SearchResult>,
}

impl Default for App {
    fn default() -> App {
        App {
            query_string: "".to_string(),
            search_results: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Invalid arguments!");
    }

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    app.query_string = args[1].clone();


    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;

    // execute!(
    //     terminal.backend_mut(),
    //     LeaveAlternateScreen,
    //     DisableMouseCapture
    // )?;

    // terminal.show_cursor()?;

    if let Err(err) = res.await {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {

    let client = reqwest::Client::new();

    let query = [DUCKDUCKGO.to_string(), app.query_string.clone()].concat();

    println!("{:?}", query);

    let response = client.post(query).send().await.unwrap();

    println!("{:?}", response);

    let response_body = match response.status() {
        reqwest::StatusCode::OK => response.text_with_charset("utf-8").await.unwrap(),
        _ => {
            println!("{:?}", response.text_with_charset("utf-8").await.unwrap());
            panic!("Error: !");
        }
    };

    // TODO: refactor to query/search

    // let response_body = fs::read_to_string("sample.html").unwrap();
    let document = Html::parse_document(&response_body);

    let result_selector = Selector::parse(".result").unwrap();
    let result_title_selector = Selector::parse(".result__a").unwrap();
    let result_summary_selector = Selector::parse(".result__snippet").unwrap();

    // for each result get
    app.search_results = document
        .select(&result_selector)
        .map(|node| {
            let title_node = node.select(&result_title_selector).next().unwrap();
            let summary = node.select(&result_summary_selector).next().unwrap();
            let href = title_node.value().attr("href").unwrap();

            let summary_components = summary
                .text()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            SearchResult {
                href: href.to_string(),
                summary: summary_components.join(""),
            }
        })
        .collect::<Vec<SearchResult>>();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read().expect("Key to be readable.") {
            match key.code {
                KeyCode::Char(c) => {
                    // TODO: navigation here
                    println!("{}", c);
                }
                _ => {}
            };
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let list_items = app
        .search_results
        .clone()
        .into_iter()
        .map(|r| ListItem::new(r.href))
        .collect::<Vec<ListItem>>();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title("Search Results: page 1")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    f.render_widget(list, chunks[0]);
}
