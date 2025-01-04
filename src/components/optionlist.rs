use std::{collections::HashSet, hash::Hash};

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, List, ListState, Padding, Paragraph, StatefulWidget, Widget},
    Frame,
};

#[derive(Debug, Clone)]
pub struct OptionList {
    options: Vec<String>,
    title: String,
    highlight_style: Style,
    item_style: Style,
    selected_marker: String,
    unselected_marker: String,
}

#[derive(Debug, Clone)]
pub struct OptionListState {
    pub highlighted: Option<usize>,
    selected: HashSet<usize>,
    option_size: usize,
}

impl Default for OptionListState {
    fn default() -> Self {
        OptionListState {
            highlighted: Some(0),
            selected: HashSet::new(),
            option_size: 1,
        }
    }
}

impl OptionListState {
    pub fn new(option_size: usize) -> OptionListState {
        OptionListState {
            highlighted: Some(0),
            selected: HashSet::new(),
            option_size: option_size,
        }
    }

    pub fn highlight_next(&mut self) {
        if let Some(index) = self.highlighted {
            if index >= self.option_size - 1 {
                self.highlighted = Some(0);
            } else {
                self.highlighted = Some(index + 1);
            }
        } else {
            self.highlighted = Some(0);
        }
    }

    pub fn highlight_prev(&mut self) {
        if let Some(index) = self.highlighted {
            if index == 0 {
                self.highlighted = Some(self.option_size - 1);
            } else {
                self.highlighted = Some(index - 1);
            }
        } else {
            self.highlighted = Some(self.option_size - 1);
        }
    }

    pub fn select(&mut self) {
        self.selected.insert(self.highlighted.unwrap());
    }

    pub fn unselect(&mut self) {
        self.selected.remove(&self.highlighted.unwrap());
    }

    pub fn is_selected(&self, index: usize) -> bool {
        return self.selected.contains(&index);
    }

    fn selected(&self) -> HashSet<usize> {
        return self.selected.clone();
    }
}

impl OptionList {
    pub fn new(
        options: Vec<String>,
        selected_marker: String,
        unselected_marker: String,
        title: String,
        highlight_style: Style,
        item_style: Style,
    ) -> OptionList {
        return OptionList {
            options,
            title,
            highlight_style,
            item_style,
            selected_marker,
            unselected_marker,
        };
    }
}

impl StatefulWidget for OptionList {
    type State = OptionListState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let entries: Vec<String> = self
            .options
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if state.is_selected(index) {
                    format!("[ ] {:<3}", item)
                } else {
                    format!("[*] {:<3}", item)
                }
            })
            .collect();

        let entry_list = List::new(entries)
            .style(self.item_style)
            .highlight_style(self.highlight_style)
            .block(Block::bordered().title(self.title));

        let mut entry_state = ListState::default().with_selected(Some(state.highlighted.unwrap()));

        StatefulWidget::render(entry_list, area, buf, &mut entry_state);
    }
}
