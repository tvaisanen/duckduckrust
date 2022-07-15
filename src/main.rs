#![allow(dead_code)]

use reqwest;
use scraper::{Html, Selector};
use std::{env, error, fs, io};

use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, Widget};
use tui::Terminal;

#[derive(Debug)]
struct SearchResult {
    href: String,
    summary: String,
}

const DUCKDUCKGO: &str = "https://html.duckduckgo.com/html?q=";

struct App {
    search_results: Vec<SearchResult>
}

impl Default for App {
    fn default() -> App {
        App {
            search_results: Vec::new()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let stdout = io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Invalid arguments!");
    }

    let client = reqwest::Client::new();

    let query = [DUCKDUCKGO.to_string(), args[1].to_string()].concat();

    // println!("{:?}", query);

    // let response = client.post(query).send().await.unwrap();

    // println!("{:?}", response);

    // let response_body = match response.status() {
    //     reqwest::StatusCode::OK => response.text_with_charset("utf-8").await.unwrap(),
    //     _ => {
    //         println!("{:?}", response.text_with_charset("utf-8").await.unwrap());
    //         panic!("Error: !");
    //     }
    // };

    let response_body = fs::read_to_string("sample.html").unwrap();
    let document = Html::parse_document(&response_body);

    let result_selector = Selector::parse(".result").unwrap();
    let result_title_selector = Selector::parse(".result__a").unwrap();
    let result_summary_selector = Selector::parse(".result__snippet").unwrap();

    // for each result get
    let results = document
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

    let items = results
        .into_iter()
        .map(|r| ListItem::new(r.href))
        .collect::<Vec<ListItem>>();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Search Results: page 1")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");

    terminal.clear()?;

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());
        f.render_widget(list, chunks[0]);
    })?;

    Ok(())
}
