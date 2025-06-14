use crate::data::data::{AssetClass, Symbol, get_data};
use crate::{Action, HOTKEY_STYLE, View};
use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};
use image::{DynamicImage, ImageBuffer, Luma, Rgba};
use plotters::{
    coord::types::{RangedCoordf32, RangedCoordi32, RangedCoordu32, RangedCoordusize},
    prelude::*,
};
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
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{
    StatefulImage,
    protocol::{
        ImageSource, StatefulProtocolType,
        kitty::{Kitty, StatefulKitty},
    },
};
use std::str::FromStr;
use strum::IntoEnumIterator;
use tokio::spawn;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

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

async fn image_loader(
    // rx: &mut UnboundedReceiver<Action>,
    tx: UnboundedSender<Action>,
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

pub struct InstrumentList {
    instruments: Vec<Instrument>,
    state: TableState,
    scroll_state: ScrollbarState,
    // image_loader: JoinHandle<Result<()>>,
    // image_protocol: StatefulProtocol,
    image_state: StatefulProtocol,
}
impl InstrumentList {
    pub(crate) fn new(tx: UnboundedSender<Action>) -> Self {
        let instruments = Symbol::iter()
            .map(|v| Instrument {
                symbol: v.to_string(),
                asset_class: v.asset_class(),
            })
            .collect::<Vec<_>>();

        // let (width, height) = (600u32, 400u32);
        // let mut buf = vec![0u8; width as usize * height as usize * 3];

        // creating a dummy image initially, until rendering the real one, using the actual font size
        let some_font_size = (7, 14); // "This is the only way to create a picker on windows, for now."
        let picker = Picker::from_fontsize(some_font_size);
        let dummy_image =
            DynamicImage::ImageRgba8(ImageBuffer::from_pixel(1, 1, Rgba([0, 0, 0, 0])));
        let stateful_protocol = picker.new_resize_protocol(dummy_image);

        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new((instruments.len() - 1) * ITEM_HEIGHT),
            instruments,
            // image_loader: spawn(image_loader(tx.clone())),
            // image_protocol: Picker::from_query_stdio()?.new_resize_protocol(StatefulImage::default()),
            image_state: stateful_protocol,
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
                    self.instruments
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

        // todo: create the actual image
        let stateful_image: StatefulImage<StatefulProtocol> = StatefulImage::default();
        f.render_stateful_widget(stateful_image, image_area, &mut self.image_state);
    }

    pub(crate) async fn on_event(
        &mut self,
        key_event: KeyEvent,
        tx: &UnboundedSender<Action>,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => tx.send(Action::Quit)?,
            KeyCode::Char('j') | KeyCode::Down => {
                self.next_row().await?;
                tx.send(Action::RequestImage)?;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous_row().await?;
                tx.send(Action::RequestImage)?;
            }
            KeyCode::Char('N') => tx.send(Action::ChangeView(View::Notes))?,
            _ => {}
        };
        Ok(())
    }

    pub(crate) async fn on_action(&mut self, action: Option<Action>) -> Result<()> {
        if let Some(action) = action {
            match action {
                Action::RequestImage => self.load_image().await?,
                _ => (), // 'Quit' and 'ChangeView' are handled in main run loop
            }
        }
        Ok(())
    }

    async fn next_row(&mut self) -> Result<()> {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.instruments.len() - 1 {
                    // cycle through the list
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

    async fn previous_row(&mut self) -> Result<()> {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.instruments.len() - 1
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

    async fn load_image(&mut self) -> Result<()> {
        // what should happen here?:
        // 1. pull the data for the selected instrument
        if let Some(selected) = self.state.selected() {
            if let Some(instrument) = self.instruments.get(selected) {
                let symbol = Symbol::from_str(instrument.symbol.as_str())?;
                let data = get_data(symbol).await;
                // let (width, height) = cell_rect_to_pixel_size(rect, font_size)
                // if let Some(symbol) = Symbol::try_from(instrument){

                // }
            };
        };

        /*
        2. draw the plot and store it in the statefull image

        plotters: integrate from c:/dev/rust/plotters_example
        ratatui_image: c:/dev/rust/test_ratatui_image

        render whatever is currently selected
        plotter needs a user provided u8 array/Vec (RGB pixel format)

        ratatui-image crate:
        StatefulProtocol = Picker::from.. .new_resize_protocol(
            DynamicImage:
                - from ImageReader::new(buffered_reader...consider wrapping the reader with a BufReader::new())
                - from ImageBuffer
                - DynamicImage::new - Creates a dynamic image backed by a buffer depending on the color type given.
                )

        let mut picker = Picker::from_query_stdio()?;
        let (cell_width_px, cell_height_px) = picker.font_size();
        let bm: &mut ratatui::prelude::Buffer = f.buffer_mut();
        */

        Ok(())
    }
}

/// (width, height) in pixels
fn cell_rect_to_pixel_size(rect: Rect, font_size: (u32, u32)) -> (u32, u32) {
    (
        rect.width as u32 * font_size.0,
        rect.height as u32 * font_size.1,
    )
}
