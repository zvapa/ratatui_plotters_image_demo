use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, ToSpan},
    widgets::{
        Block, BorderType, List, ListItem, ListState, Paragraph, Wrap,
    },
};

pub struct TodoItem {
    pub is_done: bool,
    pub description: String,
}

pub struct TodoList {
    pub items: Vec<TodoItem>,
    pub state: ListState,
    pub mode: TodoListMode,
    pub input_value: String,
}

impl Default for TodoList {
    fn default() -> Self {
        let mut items = Vec::new();
        items.push(TodoItem {
            is_done: false,
            description: " add new items here ".to_string(),
        });
        TodoList {
            items,
            state: ListState::default(),
            mode: TodoListMode::Display,
            input_value: String::default(),
        }
    }
}

impl TodoList {
    pub(crate) fn render(&mut self, f: &mut Frame<'_>, my_area: Rect) {
        match self.mode {
            TodoListMode::Display => {
                let hotkey_style = Style::new().add_modifier(Modifier::REVERSED);
                f.render_stateful_widget(
                    List::new(self.items.iter().map(|i| {
                        match i.is_done {
                            false => ListItem::new(i.description.trim()).style(Color::LightGreen),
                            true => ListItem::new(i.description.trim()).style(
                                Style::new()
                                    .fg(Color::LightGreen)
                                    .add_modifier(Modifier::CROSSED_OUT),
                            ),
                        }
                    }))
                    .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                    .block(
                        Block::bordered()
                            .border_type(BorderType::Rounded)
                            .style(Color::LightMagenta)
                            .title(Line::from(" List ").left_aligned())
                            .title(
                                Line::from(vec![
                                    Span::styled("n", hotkey_style),
                                    "ew──".into(),
                                    Span::styled("q", hotkey_style),
                                    "uit".into(),
                                ])
                                .right_aligned(),
                            )
                            .title_bottom(
                                Line::from(vec![
                                    Span::styled("i", hotkey_style),
                                    "nstruments──".into(),
                                ])
                                .left_aligned(),
                            ),
                    ),
                    my_area,
                    &mut self.state,
                )
            }
            TodoListMode::Insert => {
                f.render_widget(
                    Paragraph::new(self.input_value.to_string())
                        .wrap(Wrap { trim: true })
                        .block(
                            Block::bordered()
                                .border_type(BorderType::Rounded)
                                .style(Color::LightYellow)
                                .title(" Edit New Item ".to_span().into_left_aligned_line()),
                        ),
                    my_area,
                );
            }
        }
    }

    pub(crate) async fn on_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.mode {
            TodoListMode::Display => {
                match key_event.code {
                    KeyCode::Down => {
                        self.state.select_next();
                    }
                    KeyCode::Up => {
                        self.state.select_previous();
                    }
                    KeyCode::Char('n') => {
                        self.mode = TodoListMode::Insert;
                    }
                    KeyCode::Char(' ') => {
                        if let Some(selected_index) = self.state.selected() {
                            if let Some(td_item) = self.items.get_mut(selected_index) {
                                td_item.is_done = !td_item.is_done; // toggle done
                            };
                        };
                    }
                    KeyCode::Delete => {
                        if let Some(selected_index) = self.state.selected() {
                            self.items.remove(selected_index);
                        };
                    }
                    KeyCode::Char('i') => {
                        // action to change the view to "instruments"
                    }
                    KeyCode::Char('q') => {
                        // action to exit the application"
                    }
                    _ => {}
                }
            }
            TodoListMode::Insert => {
                match TodoListMode::handle_list_insert(self, key_event).await? {
                    FormAction::None => {}
                    FormAction::Submit(item) => {
                        self.items.push(item);
                        self.input_value.clear();
                        self.mode = TodoListMode::Display;
                    }
                    FormAction::Escape => {
                        self.mode = TodoListMode::Display;
                        self.input_value.clear();
                    }
                }
            }
        }
        Ok(())
    }
}

pub enum FormAction {
    None,
    Submit(TodoItem),
    Escape,
}

#[derive(PartialEq)]
pub enum TodoListMode {
    Display,
    Insert,
}

impl TodoListMode {
    async fn handle_list_insert(td_list: &mut TodoList, key_event: KeyEvent) -> Result<FormAction> {
        match key_event.code {
            KeyCode::Char(c) => {
                td_list.input_value.push(c);
            }
            KeyCode::Backspace => {
                td_list.input_value.pop();
            }
            KeyCode::Esc => return Ok(FormAction::Escape),
            KeyCode::Enter => {
                return Ok(FormAction::Submit(TodoItem {
                    is_done: false,
                    description: td_list.input_value.to_string(),
                }));
            }
            _ => {}
        }
        Ok(FormAction::None)
    }
}
