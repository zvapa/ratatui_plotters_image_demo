use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};
use futures::{SinkExt, channel::mpsc::UnboundedSender};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{
        Block, BorderType, Borders, Cell, Padding, Paragraph, Row, ScrollbarState, Table, TableState
    }, Frame
};

use crate::Action;

enum AssetClass {
    Forex,
    Stock,
}
impl AssetClass {
    fn as_str(&self) -> &str {
        match self {
            AssetClass::Forex => "Forex",
            AssetClass::Stock => "Stock",
        }
    }
}

pub struct Instrument {
    symbol: String,
    asset_class: AssetClass,
}
impl Instrument {
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn asset_class(&self) -> &str {
        self.asset_class.as_str()
    }
}
impl<'a> From<&'a Instrument> for Row<'a> {
    fn from(value: &'a Instrument) -> Self {
        Row::new(vec![Cell::new(value.symbol()), Cell::new(value.asset_class())])
    }
}

pub struct InstrumentList {
    items: Vec<Instrument>,
    state: TableState,
    scroll_state: ScrollbarState,
}

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
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

    pub(crate) async fn on_event(
        &mut self,
        key_event: KeyEvent,
        tx: &mut UnboundedSender<Action>,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => {
                tx.send(Action::Quit).await?
            }
            KeyCode::Down => {
                self.next_row().await?
            },
            KeyCode::Up => {
                self.previous_row().await?
            },
            _ => {}
        };
        Ok(())
    }

    pub(crate) async fn on_action(&self, _action: Option<Action>) -> Result<()> {
        unimplemented!()
    }

    pub(crate) fn render(&mut self, f: &mut Frame<'_>, area: Rect) {
        let [table_area, image_area]: [Rect; 2] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(area.inner(Margin::new(1, 1)));

        let hotkey_style = Style::new().add_modifier(Modifier::REVERSED).bold();

        // table
        f.render_stateful_widget(
            Table::default()
                .widths([Constraint::Length(8), Constraint::Length(12)])
                .column_spacing(1)
                .style(Style::new().gray())
                .header(
                    Row::new(vec!["Symbol", "Asset Class"])
                        .style(Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
                )
                .rows(self.items.iter().map(|i| i.into()))
                .row_highlight_style(Style::new().reversed())
                .block(
                    Block::new()
                        .borders(Borders::RIGHT)
                        .border_type(BorderType::Plain)
                        .border_style(Style::new().yellow())
                        .padding(Padding::uniform(1)),
                ),
            table_area,
            &mut self.state,
        );

        // image
        f.render_widget(
            Paragraph::new("image_area")
                .green()
                .block(Block::new().padding(Padding::uniform(1))),
            image_area,
        );

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
                    Line::from(vec![
                        Span::styled("(↑/↓)", hotkey_style),
                        " select ".into(),
                    ])
                    .left_aligned(),
                ),
            area,
        );
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
