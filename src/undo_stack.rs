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
