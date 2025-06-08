use color_eyre::{Result, eyre::Ok, owo_colors::OwoColorize};
use crossterm::event::{KeyCode, KeyEvent};
use futures::{SinkExt, channel::mpsc::UnboundedSender};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Styled, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, Paragraph, Row, ScrollbarState, Table, TableState,
    },
};

use crate::Action;

enum AssetClass {
    Forex,
    Stock,
}

pub struct Instrument {
    pub symbol: String,
    asset_class: AssetClass,
}

pub struct InstrumentList {
    items: Vec<Instrument>,
    state: TableState,
    scroll_state: ScrollbarState,
}

const ITEM_HEIGHT: usize = 1;

impl InstrumentList {
    pub(crate) fn new() -> Self {
        let mut items = Vec::new();
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new((items.len() - 1) * ITEM_HEIGHT),
            items,
        }
    }

    pub(crate) async fn on_event(
        &self,
        key_event: KeyEvent,
        tx: &mut UnboundedSender<Action>,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => {
                // 'q' sends Action::Quit
                tx.send(Action::Quit).await?
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) async fn on_action(&self, _action: Option<Action>) -> Result<()> {
        unimplemented!()
    }

    pub(crate) fn render(&mut self, f: &mut Frame<'_>, area: Rect) {
        let [table_area, image_area]: [Rect; 2] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(area.inner(Margin::new(1, 1)));

        let hotkey_style = Style::new().add_modifier(Modifier::REVERSED);
        // table
        // f.render_widget(
        //     Paragraph::new("table_area")
        //         .blue()
        //         .block(Block::bordered().border_type(BorderType::Rounded)),
        //     table_area,
        // );

        f.render_stateful_widget(
            Table::default()
                .widths([Constraint::Length(5), Constraint::Length(10)])
                // ...and they can be separated by a fixed spacing.
                .column_spacing(1)
                // You can set the style of the entire Table.
                .style(Style::new().blue())
                // It has an optional header, which is simply a Row always visible at the top.
                .header(Row::new(vec!["Symbol", "Asset Type"]).style(Style::new().bold()))
                .rows([
                    Row::new(vec!["Row11", "Row12"]),
                    Row::new(vec!["Row21", "Row22"]),
                    Row::new(vec!["Row31", "Row32"]),
                ])
                .row_highlight_style(Style::new().reversed())
                .block(
                    Block::new()
                        .borders(Borders::RIGHT)
                        .border_type(BorderType::Plain)
                        .border_style(Style::new().magenta()),
                ),
            table_area,
            &mut self.state,
        );

        // image
        f.render_widget(Paragraph::new("image_area").green(), image_area);

        // outer
        f.render_widget(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().yellow())
                .title(Line::from(" Instruments ").left_aligned())
                .title(
                    Line::from(vec![Span::styled("q", hotkey_style), "uit".into()]).right_aligned(),
                )
                .title_bottom(
                    Line::from(vec![Span::styled("l", hotkey_style), "ist──".into()])
                        .left_aligned(),
                ),
            area,
        );
    }
}
