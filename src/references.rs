use std::cmp::{max, min};

use crate::spreadsheet::SpreadsheetCell;

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reference {
    // A 0-indexed reference to a cell
    // Actual Excel references are 1-indexed and use letters for rows, but this is an abstraction.
    row: Option<usize>,
    col: Option<usize>,
}

impl Reference {
    pub fn range(&self, other: &Reference) -> Vec<Reference> {
        let min_row = min(self.row, other.row).unwrap();
        let min_col = min(self.col, other.col).unwrap();
        let max_row = max(self.row, other.row).unwrap();
        let max_col = max(self.col, other.col).unwrap();

        let mut cells: Vec<Reference> = Vec::new();

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                cells.push(Reference {
                    row: Some(row),
                    col: Some(col),
                });
            }
        }

        cells
    }

    pub fn to_string(&self) -> String {
        if self.row.is_some() && self.col.is_some() {
            return format!(
                "({},{})",
                Reference::index_to_alpha(self.col.unwrap() as u32 + 1).unwrap(),
                self.row.unwrap() + 1
            );
        }
        if self.row.is_some() {
            return format!("{}", self.row.unwrap());
        }
        if self.col.is_some() {
            return format!("{}", self.row.unwrap());
        }
        return String::new();
    }

    pub fn alpha_to_index(alpha: &str) -> Option<u32> {
        // Converts an Excel alphabetized column id (A, BC, XFD, etc.) into a 1-indexed number
        let mut index = 0;
        for (rev_idx, c) in alpha.chars().into_iter().rev().enumerate() {
            if !c.is_ascii_alphabetic() {
                return None;
            }
            // 1-indexed alphabet index, found from subtracting the unicode
            // number for @ (the character before A) from the letter's number
            let alphabet_idx = c as u32 - '@' as u32;
            index += alphabet_idx * 26u32.pow(rev_idx as u32);
        }

        Some(index)
    }

    pub fn index_to_alpha(index: u32) -> Option<String> {
        // Converts a 1-indexed number into an Excel alphabetized column id (A, BC, XFD, etc.)
        let mut index_mut = index.clone();
        let mut letters = vec![];
        while index_mut > 0 {
            // Same trick as before
            letters.push('@' as u32 + index_mut % 26);

            index_mut /= 26;
        }

        Some(
            letters
                .iter()
                .rev()
                .map(|c| char::from_u32(*c).unwrap().to_string())
                .collect::<Vec<String>>()
                .join(""),
        )
    }

    pub fn get_cell(&self) -> SpreadsheetCell {
        // TODO: Handle when it's just a row or col (ie. A:A, 1:1, etc.)
        return SpreadsheetCell {
            row: self.row.unwrap_or(0),
            col: self.col.unwrap_or(0),
        };
    }
}

impl std::fmt::Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub fn parse_reference(text: &str) -> Option<Reference> {
    let mut row = String::new();
    let mut col = String::new();
    let mut pointer = 0;

    // TODO: Mas while-let chains...
    while let Some(c) = text.chars().nth(pointer) {
        if !c.is_ascii_alphabetic() {
            break;
        }
        col += c.to_string().as_str();
        pointer += 1;
    }

    while let Some(c) = text.chars().nth(pointer) {
        if !c.is_ascii_digit() {
            break;
        }
        row += c.to_string().as_str();
        pointer += 1;
    }

    // If both row and col are empty, or the text still has stuff to go.
    if (row.len() == 0 && col.len() == 0) || pointer != text.len() {
        return None;
    }

    Some(Reference {
        // TODO: IF-LET FUCKING CHAINING
        col: if col.len() > 0 && Reference::alpha_to_index(&col).is_some() {
            Some(
                (Reference::alpha_to_index(&col).unwrap() - 1)
                    .try_into()
                    .unwrap(),
            ) // Unwrap okay
        } else {
            None
        },
        row: if row.len() > 0 && row.parse::<usize>().is_ok() {
            Some(row.parse::<usize>().unwrap() - 1) // Unwrap okay
        } else {
            None
        },
    })
}
