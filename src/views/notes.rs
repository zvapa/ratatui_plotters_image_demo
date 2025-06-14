use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, ToSpan},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Wrap},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{Action, HOTKEY_STYLE};

#[derive(PartialEq)]
pub(crate) struct Note {
    pub(crate) content: String,
}
impl Note {
    pub(crate) fn title(&self) -> String {
        self.content.chars().take(20).collect::<String>() + "..."
    }
}

pub(crate) struct Notes {
    pub items: Vec<Note>,
    pub state: ListState,
    pub mode: NotesMode,
    pub input_value: String,
}

impl Notes {
    pub(crate) fn new() -> Self {
        let mut items = Vec::new();
        items.push(Note {
            content: "add new or edit note...".to_string(),
        });
        Notes {
            items,
            state: ListState::default(),
            mode: NotesMode::DisplayList,
            input_value: String::default(),
        }
    }

    pub(crate) fn render(&mut self, f: &mut Frame<'_>, my_area: Rect) {
        match self.mode {
            NotesMode::DisplayList => f.render_stateful_widget(
                List::new(self.items.iter().map(|i| {
                    let title = i.title().to_owned();
                    ListItem::new(title).style(Color::LightMagenta)
                }))
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .style(Color::LightMagenta)
                        .title(Line::from(" Notes ").left_aligned())
                        .title(
                            Line::from(vec![
                                Span::styled("I", HOTKEY_STYLE),
                                "nstruments──".into(),
                                Span::styled("q", HOTKEY_STYLE),
                                "uit ".into(),
                            ])
                            .right_aligned(),
                        )
                        .title_bottom(
                            Line::from(vec![
                                Span::styled("j(↓)/h(↑)", HOTKEY_STYLE),
                                "(select)──".into(),
                                Span::styled("l", HOTKEY_STYLE),
                                "(edit)──".into(),
                                Span::styled("n", HOTKEY_STYLE),
                                "ew──".into(),
                                Span::styled("d", HOTKEY_STYLE),
                                "elete".into(),
                            ])
                            .left_aligned(),
                        ),
                ),
                my_area,
                &mut self.state,
            ),
            NotesMode::AddNew => {
                f.render_widget(
                    Paragraph::new(self.input_value.to_string())
                        .wrap(Wrap { trim: true })
                        .block(
                            Block::bordered()
                                .border_type(BorderType::Rounded)
                                .style(Color::LightGreen)
                                .title(" New Note ".to_span().into_left_aligned_line())
                                .title_bottom(
                                    Line::from(vec![
                                        Span::styled("Enter", HOTKEY_STYLE),
                                        "(save)──".into(),
                                        Span::styled("Esc", HOTKEY_STYLE),
                                        "(back)".into(),
                                    ])
                                    .left_aligned(),
                                ),
                        ),
                    my_area,
                );
            }
            NotesMode::Edit { selected_ix } => {
                if let Some(note) = self.items.get(selected_ix) {
                    f.render_widget(
                        Paragraph::new(note.content.to_string()).block(
                            Block::bordered()
                                .border_type(BorderType::Rounded)
                                .style(Color::LightCyan)
                                .title(" Edit Note ".to_span().into_left_aligned_line())
                                .title_bottom(
                                    Line::from(vec![
                                        Span::styled("Enter/Esc", HOTKEY_STYLE),
                                        "(back)".into(),
                                    ])
                                    .left_aligned(),
                                ),
                        ),
                        my_area,
                    );
                }
            }
        }
    }

    pub(crate) async fn on_event(
        &mut self,
        key_event: KeyEvent,
        tx: &UnboundedSender<Action>,
    ) -> Result<()> {
        match self.mode {
            NotesMode::DisplayList => match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => self.state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.state.select_previous(),
                KeyCode::Char('n') => {
                    self.mode = NotesMode::AddNew;
                }
                KeyCode::Char('l') => {
                    match self.state.selected() {
                        Some(selected_ix) => self.mode = NotesMode::Edit { selected_ix },
                        None => {
                            self.mode = NotesMode::AddNew;
                        }
                    };
                }
                KeyCode::Char('d') => {
                    if let Some(selected_index) = self.state.selected() {
                        self.items.remove(selected_index);
                    };
                }
                KeyCode::Char('I') => {
                    tx.send(Action::ChangeView(crate::View::Instruments))?
                }
                KeyCode::Char('q') => tx.send(Action::Quit)?,
                _ => {}
            },
            NotesMode::AddNew => match key_event.code {
                KeyCode::Char(c) => self.input_value.push(c),
                KeyCode::Backspace => {
                    self.input_value.pop();
                }
                KeyCode::Esc => {
                    self.input_value.clear();
                    self.mode = NotesMode::DisplayList;
                }
                KeyCode::Enter => {
                    self.items.push(Note {
                        content: self.input_value.to_string(),
                    });
                    self.input_value.clear();
                    self.mode = NotesMode::DisplayList;
                }
                _ => {}
            },
            NotesMode::Edit { selected_ix } => {
                if let Some(note) = self.items.get_mut(selected_ix) {
                    match key_event.code {
                        KeyCode::Char(c) => note.content.push(c),
                        KeyCode::Backspace => {
                            note.content.pop();
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            self.mode = NotesMode::DisplayList;
                        }
                        _ => {}
                    }
                } else {
                    self.mode = NotesMode::DisplayList;
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn on_action(&mut self, _action: Option<Action>) -> Result<()> {
        unimplemented!() // this screen supports no actions yet
    }
}

#[derive(PartialEq)]
pub enum NotesMode {
    DisplayList,
    AddNew,
    Edit { selected_ix: usize },
}
