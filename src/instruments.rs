use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};
use futures::{channel::mpsc::UnboundedSender, SinkExt};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, List, ListState},
};

use crate::Action;

enum AssetType {
    Forex,
    Stock,
}

pub struct Instrument {
    pub name: String,
    // asset_type: AssetType,
}

pub struct InstrumentList {
    pub items: Vec<Instrument>,
    pub state: ListState,
}

impl InstrumentList {
    pub async fn on_event(&self, key_event: KeyEvent, tx: &mut UnboundedSender<Action>) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => {
                // 'q' sends Action::Quit
                tx.send(Action::Quit).await?
            },
            _ => {},
        }
        Ok(())
    }

    pub async fn on_action(&self, action: Option<Action>) {

    }

    pub fn render(&mut self, f: &mut Frame<'_>, my_area: Rect) {
        let hotkey_style = Style::new().add_modifier(Modifier::REVERSED);
        f.render_stateful_widget(
            List::new(self.items.iter().map(|i| Text::raw(i.name.clone())))
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .style(Color::LightMagenta)
                        .title(Line::from(" instruments ").left_aligned())
                        .title(
                            Line::from(vec![Span::styled("q", hotkey_style), "uit".into()])
                                .right_aligned(),
                        )
                        .title_bottom(
                            Line::from(vec![Span::styled("l", hotkey_style), "ist──".into()])
                                .left_aligned(),
                        ),
                ),
            my_area,
            &mut self.state,
        )
    }
}

impl Default for InstrumentList {
    fn default() -> Self {
        let mut items = Vec::new();
        items.push(Instrument {
            // asset_type: AssetType::Forex,
            name: "EURUSD".into(),
        });
        items.push(Instrument {
            // asset_type: AssetType::Stock,
            name: "AAPL".into(),
        });
        InstrumentList {
            items,
            state: ListState::default(),
        }
    }
}
