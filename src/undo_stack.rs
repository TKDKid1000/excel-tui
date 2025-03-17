use std::fmt::{Debug, Display};

use crate::undo_stack;

#[derive(Debug)]
pub struct UndoStack<T: Clone> {
    undo: Vec<T>,
    redo: Vec<T>,
}

impl<T> Default for UndoStack<T>
where
    T: Clone,
    T: PartialEq,
{
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }
}

impl<T> UndoStack<T>
where
    T: Clone,
    T: PartialEq,
    T: Debug,
{
    pub fn can_undo(self) -> bool {
        self.undo.len() > 0
    }

    pub fn undo(&mut self) -> Option<T> {
        if let Some(edit) = self.undo.pop() {
            self.redo.push(edit.clone());
            return Some(edit);
        }
        None
    }

    pub fn can_redo(self) -> bool {
        self.redo.len() > 0
    }

    pub fn redo(&mut self) -> Option<T> {
        if let Some(edit) = self.redo.pop() {
            self.undo.push(edit.clone());
            return Some(edit);
        }
        None
    }

    pub fn edit(&mut self, edit: T) {
        if let Some(last) = self.undo.last() {
            if last == &edit {
                return;
            }
        }
        self.redo.clear();
        self.undo.push(edit);
    }
}

impl<T> Display for UndoStack<T>
where
    T: Clone,
    T: PartialEq,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            self.undo.len(),
            self.undo.len() + self.redo.len()
        )
    }
}
