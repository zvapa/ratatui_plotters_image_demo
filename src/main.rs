mod instruments;
mod todo_list;

use color_eyre::{Result, eyre::Ok};
use crossterm::event::{Event, EventStream, KeyEvent, KeyEventKind};
use futures::{channel::mpsc::UnboundedSender, select, FutureExt, StreamExt};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
};

use crate::{
    instruments::InstrumentList,
    todo_list::{FormAction, TodoItem, TodoList},
};

pub(crate) enum View {
    Instruments(InstrumentList),
    List(TodoList),
}

pub(crate) enum Action {
    Quit,
    ImageAction,
    FormAction(FormAction),
}

pub(crate) struct State {
    pub(crate) current_view: View,
    pub(crate) running: bool, // use to exit the app
}

impl State {
    pub fn new() -> Self {
        let state = State {
            current_view: View::Instruments(InstrumentList::default()),
            // current_view: View::List(TodoList::default()),
            running: true,
        };
        state
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

async fn run(terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
    let mut state = State::new();
    let mut crossterm_event_stream = EventStream::new();
    let (mut tx, mut rx) = futures::channel::mpsc::unbounded::<Action>(); // this should be part of State... ?
    loop {
        let mut fe = crossterm_event_stream.next().fuse();
        let mut fa = rx.next().fuse();
        select! {
            maybe_event = fe => on_event(maybe_event, &mut state, &mut tx).await?,
            maybe_action = fa => on_action(maybe_action, &mut state).await?,
            // other 'actions' here..
        }
        if !state.running {
            break;
        }
        terminal.draw(|f| render(f, &mut state))?;
    }
    Ok(())
}

async fn on_action(maybe_action: Option<Action>, state: &mut State) -> Result<()> {
    // if it's 'Quit' -> quit
    if let Some(Action::Quit) = maybe_action {
        state.running = false;
    }
    match &mut state.current_view {
        View::Instruments(instrument_list) => {
            // 'Instruments' view handles the action
        },
        View::List(todo_list) => {
            // 'ToDo list' view handles the action

        },
    }
    Ok(())
}

async fn on_event(
    maybe_event: Option<Result<Event, std::io::Error>>,
    state: &mut State,
    tx: &mut UnboundedSender<Action>
) -> color_eyre::Result<()> {
    match maybe_event {
        Some(std::result::Result::Ok(ev)) => {
            // only handling key events for now
            if let Event::Key(key_event) = ev {
                if key_event.kind == KeyEventKind::Press {
                    // delegate to the views
                    match &mut state.current_view {
                        View::List(td_list) => {
                            td_list.on_event(key_event).await?;
                        }
                        View::Instruments(instrument_list) => {
                            instrument_list.on_event(key_event, tx).await?;
                        }
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

/// called via ```terminal.draw(|f| render(f, &mut app_state))?;``` line inside [`fn@crate::run`] loop
/// to render the entire frame based on the current [`struct@crate::AppState`]
fn render(f: &mut Frame, state: &mut State) {
    let [my_area]: [Rect; 1] = Layout::vertical([Constraint::Fill(1)]).areas(f.area());

    match &mut state.current_view {
        View::Instruments(instrument_list) => instrument_list.render(f, my_area),
        View::List(todo_list) => todo_list.render(f, my_area),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    run(&mut terminal).await?;

    ratatui::restore();
    Ok(())
}
