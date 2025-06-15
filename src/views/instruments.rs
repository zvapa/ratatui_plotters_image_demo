use crate::data::data::{AssetClass, Symbol, get_data};
use crate::{Action, HOTKEY_STYLE, View};
use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};
use image::{DynamicImage, ImageBuffer, RgbImage};
use plotters::{
    coord::types::{RangedCoordf32, RangedCoordi32},
    prelude::*,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Padding, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::str::FromStr;
use strum::IntoEnumIterator;
use tokio::sync::mpsc::UnboundedSender;

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
    instruments: Vec<Instrument>,
    state: TableState,
    scroll_state: ScrollbarState,
    picker: Picker,
}
impl InstrumentList {
    pub(crate) fn new(_tx: UnboundedSender<Action>, picker: Picker) -> Self {
        let instruments = Symbol::iter()
            .map(|v| Instrument {
                symbol: v.to_string(),
                asset_class: v.asset_class(),
            })
            .collect::<Vec<_>>();
        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new((instruments.len() - 1) * ITEM_HEIGHT),
            instruments,
            picker,
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

        // let stateful_image: StatefulImage<StatefulProtocol> = StatefulImage::default();
        // f.render_stateful_widget(stateful_image, image_area, &mut self.img_protocol);

        self.render_image(f, image_area).expect("Failed to render image");
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
                tx.send(Action::RequestImageData)?;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous_row().await?;
                tx.send(Action::RequestImageData)?;
            }
            KeyCode::Char('N') => tx.send(Action::ChangeView(View::Notes))?,
            _ => {}
        };
        Ok(())
    }

    pub(crate) async fn on_action(&mut self, action: Option<Action>) -> Result<()> {
        if let Some(action) = action {
            match action {
                Action::RequestImageData => {/* todo: offload image state init to another thread */},
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

    fn render_image(&self, f: &mut Frame<'_>, image_area: Rect) -> Result<()> {
        let (cell_width_px, cell_height_px) = self.picker.font_size();
        // let (width, height) = (600u32, 400u32);
        let (width, height) = cell_rect_to_pixel_size(&image_area, (cell_width_px, cell_height_px));
        let mut img_buf = vec![0u8; width as usize * height as usize * 3]; // RGB pixel format
        let root = BitMapBackend::with_buffer(&mut img_buf, (width as u32, height as u32))
            .into_drawing_area();
        root.fill(&BLACK)?;

        if let Some(selected) = self.state.selected() {
            if let Some(instrument) = self.instruments.get(selected) {
                let symbol = Symbol::from_str(instrument.symbol.as_str())?;
                let data = get_data(symbol);

                let (_, o, h, l, c) = data.first().unwrap();
                let (mut y_min, mut y_max) = (o.min(*h).min(*l).min(*c), o.max(*h).max(*l).max(*c));
                (y_min, y_max) =
                    data.iter()
                        .skip(1)
                        .fold((y_min, y_max), |(y_min, y_max), (_, o, h, l, c)| {
                            (
                                y_min.min(*o).min(*h).min(*l).min(*c),
                                y_max.max(*o).max(*h).max(*l).max(*c),
                            )
                        });

                let mut chart: ChartContext<
                    '_,
                    BitMapBackend<'_>,
                    Cartesian2d<RangedCoordi32, RangedCoordf32>,
                > = ChartBuilder::on(&root)
                    .x_label_area_size(25)
                    .right_y_label_area_size(45)
                    .build_cartesian_2d(-1 as i32..data.len() as i32, y_min..y_max)?;

                chart
                    .configure_mesh()
                    .disable_x_mesh()
                    .disable_y_mesh()
                    .axis_style(ShapeStyle {
                        color: plotters::style::Color::to_rgba(&WHITE),
                        filled: false,
                        stroke_width: 1,
                    })
                    .disable_x_axis()
                    .y_label_style(
                        TextStyle::from(("sans-serif", 15).into_font())
                            .color(&plotters::style::Color::to_rgba(&WHITE)),
                    )
                    .draw()?;

                chart.draw_series(data.iter().enumerate().map(|(ix, x)| {
                    CandleStick::new(
                        ix as i32,
                        x.1,
                        x.2,
                        x.3,
                        x.4,
                        plotters::style::Color::filled(&WHITE),
                        WHITE,
                        5,
                    )
                }))?;

                // manually call the present function to avoid the IO failure being ignored silently
                root.present()
                    .expect("Failed to draw chart result to backend");

                drop(chart); // to release the mutable borrow of buff
                drop(root); // also to release the mutable borrow of buff

                let rgb_img: RgbImage = ImageBuffer::from_raw(width as u32, height as u32, img_buf)
                    .expect("Failed to construct ImageBuffer");
                let mut stateful_protocol = self
                    .picker
                    .new_resize_protocol(DynamicImage::ImageRgb8(rgb_img));
                let stateful_image: StatefulImage<StatefulProtocol> = StatefulImage::default();
                f.render_stateful_widget(stateful_image, image_area, &mut stateful_protocol);
            };
        };

        Ok(())
    }
}

/// (width, height) in pixels
fn cell_rect_to_pixel_size(rect: &Rect, font_size: (u16, u16)) -> (u16, u16) {
    (rect.width * font_size.0, rect.height * font_size.1)
}
