mod data {
    pub(crate) mod data;
}
mod views {
    pub(crate) mod instruments;
    pub(crate) mod notes;
}

use crate::{views::instruments::InstrumentList, views::notes::Notes};
use color_eyre::{Result, eyre::Ok};
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::FutureExt;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
};
use ratatui_image::picker::Picker;
use tokio::{
    self, sync::mpsc::{UnboundedSender, unbounded_channel},
};
use tokio_stream::StreamExt;

pub(crate) const HOTKEY_STYLE: ratatui::prelude::Style =
    Style::new().add_modifier(Modifier::REVERSED);

pub(crate) enum View {
    Instruments,
    Notes,
}

pub(crate) enum Action {
    Quit,
    RequestImageData,
    ChangeView(View),
}

pub(crate) struct State {
    pub(crate) current_view: View,
    pub(crate) instruments: InstrumentList,
    pub(crate) notes: Notes,
    pub(crate) running: bool, // use to exit the app
}

impl State {
    pub(crate) fn new(tx: UnboundedSender<Action>, picker: Picker) -> Self {
        let state = State {
            instruments: InstrumentList::new(tx, picker),
            notes: Notes::new(),
            current_view: View::Instruments,
            running: true,
        };
        state
    }
}

async fn run(terminal: &mut DefaultTerminal, picker: Picker) -> color_eyre::Result<()> {
    let (tx, mut rx) = unbounded_channel::<Action>();
    let mut state = State::new(tx.clone(), picker);
    let mut crossterm_event_stream = EventStream::new();

    loop {
        terminal.draw(|f| render(f, &mut state))?;

        let fe = crossterm_event_stream.next().fuse();
        let fa = rx.recv().fuse();

        tokio::select! {
            maybe_event = fe => on_event(maybe_event, &mut state, &tx).await?,
            maybe_action = fa => on_action(maybe_action, &mut state, &tx).await?,
        }

        if !state.running {
            break;
        }
    }
    Ok(())
}

async fn on_event(
    maybe_event: Option<Result<Event, std::io::Error>>,
    state: &mut State,
    tx: &UnboundedSender<Action>,
) -> Result<()> {
    match maybe_event {
        Some(std::result::Result::Ok(ev)) => {
            // only handling key events for now
            if let Event::Key(key_event) = ev {
                if key_event.kind == KeyEventKind::Press {
                    // delegate to the views
                    match &mut state.current_view {
                        View::Notes => {
                            state.notes.on_event(key_event, tx).await?;
                        }
                        View::Instruments => {
                            state.instruments.on_event(key_event, tx).await?;
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

async fn on_action(
    maybe_action: Option<Action>,
    state: &mut State,
    _tx: &UnboundedSender<Action>,
) -> Result<()> {
    // handle application wide actions: quit, help, change view
    match maybe_action {
        Some(Action::Quit) => {
            state.running = false;
            return Ok(());
        }
        Some(Action::ChangeView(ref view)) => match view {
            View::Instruments => {
                state.current_view = View::Instruments;
                return Ok(());
            }
            View::Notes => {
                state.current_view = View::Notes;
                return Ok(());
            }
        },
        Some(specific_screen_action) => {
            // delegate specific actions to the views
            match &mut state.current_view {
                View::Instruments => {
                    state
                        .instruments
                        .on_action(Some(specific_screen_action))
                        .await?;
                }
                View::Notes => {
                    state.notes.on_action(Some(specific_screen_action)).await?;
                }
            }
        }
        None => {}
    }
    Ok(())
}

/// called via ```terminal.draw(|f| render(f, &mut app_state))?;``` line inside [`fn@crate::run`] loop
/// to render the entire frame based on the current [`struct@crate::State`]
fn render(f: &mut Frame, state: &mut State) {
    let [my_area]: [Rect; 1] = Layout::vertical([Constraint::Fill(1)]).areas(f.area());
    match &mut state.current_view {
        View::Instruments => state.instruments.render(f, my_area),
        View::Notes => state.notes.render(f, my_area),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let picker = Picker::from_query_stdio()?;
    run(&mut terminal, picker).await?;
    ratatui::restore();
    Ok(())
}
