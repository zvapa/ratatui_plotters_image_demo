use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt, select};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::ToSpan,
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::Error;

#[derive(Default)]
struct TodoItem {
    is_done: bool,
    description: String,
}

#[derive(Default)]
pub struct AppState {
    // screen/mode: home
    items: Vec<TodoItem>,
    list_state: ListState,

    is_list_add_new: bool,
    // screen/mode: list add new
    input_value: String,

    // exit
    running: bool,
}

impl AppState {
    pub fn init() -> Self {
        let mut app_state = AppState::default();
        app_state.running = true;
        app_state.items.push(TodoItem {
            is_done: false,
            description: " add new items here ".to_string(),
        });
        app_state
    }
}

/*
(async?) fn start_image_loader(img_rx: mpsc::Receiver<Action>, tx:: mpsc::Sender<Action>) {
    loop {
        // Only reacts to ImageRequested messages
        if let Ok(Action::ImageRequested) = img_rx.recv() {
            let buffer = std::fs::read("img.png").unwrap_or_else(|_| vec![]);
            tx.send(Action::ImageLoaded(buffer)).ok();
        }
    }
}
 */

async fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app_state = AppState::init();
    let mut event_stream = EventStream::new();
    loop {
        let mut f = event_stream.next().fuse();
        select! {
            maybe_event = f => handle_event(maybe_event, &mut app_state).await?,
        }
        if !app_state.running {
            break;
        }
        terminal.draw(|f| render(f, &mut app_state))?;
    }
    Ok(())
}

async fn handle_event(
    maybe_event: Option<Result<Event, Error>>,
    app_state: &mut AppState,
) -> Result<()> {
    match maybe_event {
        Some(Ok(ev)) => {
            if let Event::Key(key_event) = ev {
                if key_event.kind == KeyEventKind::Press {
                    if app_state.is_list_add_new {
                        match handle_action_list_add_new(app_state, key_event) {
                            FormAction::None => {}
                            FormAction::Submit => {
                                app_state.is_list_add_new = false;
                                app_state.items.push(TodoItem {
                                    is_done: false,
                                    description: app_state.input_value.to_string(),
                                });
                                app_state.input_value.clear();
                            }
                            FormAction::Escape => {
                                app_state.is_list_add_new = false;
                                app_state.input_value.clear();
                            }
                        }
                    } else {
                        handle_action_home_screen(app_state, key_event);
                    }
                }
            }
        }
        Some(Err(err)) => {
            return Err(err.into());
        }
        None => {}
    }
    Ok(())
}

enum FormAction {
    None,
    Submit,
    Escape,
}

// another 'mode' ("list add new item")
fn handle_action_list_add_new(app_state: &mut AppState, key_event: KeyEvent) -> FormAction {
    match key_event.code {
        KeyCode::Char(c) => {
            app_state.input_value.push(c);
        }
        KeyCode::Backspace => {
            app_state.input_value.pop();
        }
        KeyCode::Esc => return FormAction::Escape,
        KeyCode::Enter => return FormAction::Submit,
        _ => {}
    }
    FormAction::None
}

// in 'home' screen (mode)
fn handle_action_home_screen(app_state: &mut AppState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') => {
            app_state.running = false;
        }
        KeyCode::Down => {
            app_state.list_state.select_next();
        }
        KeyCode::Up => {
            app_state.list_state.select_previous();
        }
        KeyCode::Insert => {
            app_state.is_list_add_new = true;
        }
        KeyCode::Char(' ') => {
            if let Some(selected_index) = app_state.list_state.selected() {
                if let Some(td_item) = app_state.items.get_mut(selected_index) {
                    td_item.is_done = !td_item.is_done; // toggle
                };
            };
        }
        KeyCode::Delete => {
            if let Some(selected_index) = app_state.list_state.selected() {
                app_state.items.remove(selected_index);
            };
        }
        _ => {}
    }
}

/// called via ```terminal.draw(|f| render(f, &mut app_state))?;``` line inside [`fn@crate::run`] loop
/// to render the entire frame based on the current [`struct@crate::AppState`]
fn render(f: &mut Frame, app_state: &mut AppState) {
    let [my_area]: [Rect; 1] = Layout::vertical([Constraint::Fill(1)]).areas(f.area());

    // add new list item 'mode'
    if app_state.is_list_add_new {
        render_list_add_new(f, app_state, my_area);
    } else {
        // 'normal' / 'home' mode
        render_normal_mode(f, app_state, my_area);
    }
}

fn render_normal_mode(f: &mut Frame<'_>, app_state: &mut AppState, my_area: Rect) {
    f.render_stateful_widget(
        List::new(app_state.items.iter().map(|i| {
            match i.is_done {
                false => ListItem::new(i.description.trim()).style(Color::LightGreen),
                true => ListItem::new(i.description.trim()).style(
                    Style::new()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::CROSSED_OUT),
                ),
            }
        }))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Color::LightMagenta)
                .title(" list ")
                .title_alignment(Alignment::Center),
        ),
        my_area,
        &mut app_state.list_state,
    )
}

fn render_list_add_new(f: &mut Frame<'_>, app_state: &mut AppState, my_area: Rect) {
    f.render_widget(
        Paragraph::new(app_state.input_value.to_string())
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .style(Color::LightYellow)
                    .title(" edit item ".to_span().into_centered_line()),
            ),
        my_area,
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    run(&mut terminal).await?;
    // App::new().run(&mut terminal).await?;

    ratatui::restore();
    Ok(())
}
