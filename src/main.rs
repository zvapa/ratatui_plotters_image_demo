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
use futures::{
    FutureExt, SinkExt, StreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded},
    pin_mut, select,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
};
use ratatui_image::{StatefulImage, picker::Picker, protocol::StatefulProtocol};

pub(crate) const HOTKEY_STYLE: ratatui::prelude::Style =
    Style::new().add_modifier(Modifier::REVERSED);

pub(crate) enum View {
    Instruments,
    Notes,
}

pub(crate) enum Action {
    Quit,
    RequestImage,
    ImageLoaded(StatefulImage<StatefulProtocol>),
    ChangeView(View),
}

pub(crate) struct State {
    pub(crate) current_view: View,
    pub(crate) instruments: InstrumentList,
    pub(crate) notes: Notes,
    pub(crate) running: bool, // use to exit the app
}

impl State {
    pub(crate) fn new() -> Self {
        let state = State {
            instruments: InstrumentList::new(),
            notes: Notes::new(),
            current_view: View::Notes,
            running: true,
        };
        state
    }
}

async fn start_image_loader(
    img_rx: &mut UnboundedReceiver<Action>,
    tx: &mut UnboundedSender<Action>,
) -> Result<()> {
    // loop {
    //     // continually reacts for RequestImage messages only
    //     if let Some(Action::RequestImage) = img_rx.try_next()? {
    //         let im = StatefulImage::default();
    //         tx.send(Action::ImageLoaded(im)).await?
    //     }
    // }
    Ok(())
}

async fn run(terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
    let mut state = State::new();
    let mut crossterm_event_stream = EventStream::new();
    let (tx, mut rx) = unbounded::<Action>();
    // image loader's own channel to receive RequestImage
    let (_, mut img_rx) = unbounded::<Action>();

    let mut tx_clone = tx.clone();
    let fut_img_action = start_image_loader(&mut img_rx, &mut tx_clone);
    pin_mut!(fut_img_action);
    let mut fut_img_action_fused = fut_img_action.fuse();

    loop {
        let mut fut_event = crossterm_event_stream.next().fuse();
        let mut fut_action = rx.next().fuse();
        select! {
            maybe_event = fut_event => on_event(maybe_event, &mut state, &mut tx.clone()).await?,
            maybe_action = fut_action => on_action(maybe_action, &mut state, &mut tx.clone()).await?,
            _ = fut_img_action_fused => { },
        }
        if !state.running {
            break;
        }
        terminal.draw(|f| render(f, &mut state))?;
    }
    Ok(())
}

async fn on_event(
    maybe_event: Option<Result<Event, std::io::Error>>,
    state: &mut State,
    tx: &mut UnboundedSender<Action>,
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
    tx: &mut UnboundedSender<Action>,
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
                        state.instruments.on_action(Some(specific_screen_action)).await?;
                    }
                    View::Notes => {
                        state.notes.on_action(Some(specific_screen_action)).await?;
                    }
                }
            }
        None => {},
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
    run(&mut terminal).await?;
    ratatui::restore();
    Ok(())
}
