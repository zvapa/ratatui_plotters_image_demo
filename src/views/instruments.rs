use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};
use futures::{SinkExt, channel::mpsc::UnboundedSender};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, StatefulWidget, Table, TableState, Widget,
    },
};
use crate::data::data::{AssetClass, Symbol, get_data};
use crate::{Action, View, HOTKEY_STYLE};

const ITEM_HEIGHT: usize = 1;

pub struct Instrument {
    symbol: String,
    asset_class: AssetClass,
}
impl Instrument {
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn asset_class(&self) -> &str {
        self.asset_class.as_ref()
    }
}

pub struct InstrumentList {
    items: Vec<Instrument>,
    state: TableState,
    scroll_state: ScrollbarState,
}
impl InstrumentList {
    pub(crate) fn new() -> Self {
        let mut items = Vec::new();
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "GBPUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "NVDA".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "GBPUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "NVDA".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "GBPUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "NVDA".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "GBPUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "NVDA".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "GBPUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "NVDA".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "GBPUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "NVDA".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "EURUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Forex,
            symbol: "GBPUSD".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "AAPL".into(),
        });
        items.push(Instrument {
            asset_class: AssetClass::Stock,
            symbol: "NVDA".into(),
        });
        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new((items.len() - 1) * ITEM_HEIGHT),
            items,
        }
    }

    pub(crate) fn render(&mut self, f: &mut Frame<'_>, main_area: Rect) {
        // outer block
        let outer_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::LightYellow))
            .title(Line::from(" Instruments ").left_aligned())
            .title(
                Line::from(vec![
                    Span::styled("N", HOTKEY_STYLE),
                    "otes──".into(),
                    Span::styled("q", HOTKEY_STYLE),
                    "uit ".into(),
                ])
                .right_aligned(),
            )
            .title_bottom(
                Line::from(vec![
                    Span::styled("j(↓)/h(↑)", HOTKEY_STYLE),
                    "(select)".into(),
                ])
                .left_aligned(),
            )
            .padding(Padding::uniform(1));

        f.render_widget(&outer_block, main_area);

        // split outer block inner area into 2 areas: for the table and the image
        let [table_area, image_area]: [Rect; 2] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(outer_block.inner(main_area));

        // table
        f.render_stateful_widget(
            Table::default()
                .widths([
                    Constraint::Length(3),
                    Constraint::Length(8),
                    Constraint::Length(12),
                ])
                .column_spacing(1)
                .style(Style::new().gray())
                .header(
                    Row::new(vec!["Ix", "Symbol", "Asset Class"])
                        .style(Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
                )
                .rows(
                    self.items
                        .iter()
                        .enumerate()
                        .map(|(i, item)| {
                            Row::new([
                                Cell::new(i.to_string()).style(Color::DarkGray),
                                Cell::new(item.symbol()),
                                Cell::new(item.asset_class()),
                            ])
                        })
                        .collect::<Vec<Row<'_>>>(),
                )
                // hack: empty footer, to fix scrollbar 'thumb' not visible on last row
                .footer(
                    Row::new([Cell::default(), Cell::default()])
                        .style(Style::default().bg(Color::DarkGray)),
                )
                .row_highlight_style(Style::new().reversed())
                .block(
                    Block::new()
                        .borders(Borders::RIGHT)
                        .border_type(BorderType::Plain)
                        .border_style(Style::new().dark_gray()),
                ),
            table_area,
            &mut self.state,
        );

        // scrollbar, same area as table (overlapping the edge)
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            table_area.inner(Margin {
                vertical: 1,
                horizontal: 2,
            }),
            &mut self.scroll_state,
        );

        // plotters: integrate from c:/dev/rust/plotters_example
        // ratatui_image: c:/dev/rust/test_ratatui_image

        let bm: &mut ratatui::prelude::Buffer = f.buffer_mut();

        let w = image_area.as_size().width;
        let h = image_area.as_size().height;
        f.render_widget(
            Paragraph::new(format!("{w}x{h}"))
                .green()
                .block(Block::new().padding(Padding::uniform(1))),
            image_area,
        );
    }

    pub(crate) async fn on_event(
        &mut self,
        key_event: KeyEvent,
        tx: &mut UnboundedSender<Action>,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => tx.send(Action::Quit).await?,
            KeyCode::Char('j') | KeyCode::Down => self.next_row().await?,
            KeyCode::Char('k') | KeyCode::Up => self.previous_row().await?,
            KeyCode::Char('N') => tx.send(Action::ChangeView(View::Notes)).await?,
            _ => {}
        };
        Ok(())
    }

    pub(crate) async fn on_action(&self, _action: Option<Action>) -> Result<()> {
        unimplemented!()
    }

    pub async fn next_row(&mut self) -> Result<()> {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);

        // send action to load the image for the selected instrument

        Ok(())
    }

    pub async fn previous_row(&mut self) -> Result<()> {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
        Ok(())
    }
}
