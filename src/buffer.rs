use pbtree::PieceTable;

/// In-memory text buffer backed by a piece table.
///
/// All text positions are represented as character indices.
pub struct TextBuffer {
    table: PieceTable<char>,
    line_starts: Vec<usize>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            table: PieceTable::new(Vec::new()),
            line_starts: vec![0],
        }
    }
}

impl TextBuffer {
    /// Returns total number of characters in the buffer.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Inserts text at the given character index.
    pub fn insert_str(&mut self, pos: usize, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        self.table.insert(pos, &chars);
        self.rebuild_line_starts();
    }

    /// Inserts a single character at the given character index.
    pub fn insert_char(&mut self, pos: usize, ch: char) {
        self.table.insert(pos, &[ch]);
        self.rebuild_line_starts();
    }

    /// Deletes one character before the provided cursor index.
    pub fn delete_backward(&mut self, cursor_index: usize) {
        if cursor_index > 0 && cursor_index <= self.len() {
            self.table.delete(cursor_index - 1, 1);
            self.rebuild_line_starts();
        }
    }

    /// Returns number of logical lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns the character length of a line (without newline).
    pub fn line_len(&self, row: usize) -> usize {
        let Some(start) = self.line_starts.get(row).copied() else {
            return 0;
        };

        let end = if row + 1 < self.line_starts.len() {
            self.line_starts[row + 1].saturating_sub(1)
        } else {
            self.len()
        };

        end.saturating_sub(start)
    }

    /// Converts a `(row, col)` coordinate into a character index.
    pub fn index_from_row_col(&self, row: usize, col: usize) -> usize {
        if self.line_starts.is_empty() {
            return 0;
        }

        let clamped_row = row.min(self.line_starts.len().saturating_sub(1));
        let line_start = self.line_starts[clamped_row];
        let line_end = if clamped_row + 1 < self.line_starts.len() {
            self.line_starts[clamped_row + 1].saturating_sub(1)
        } else {
            self.len()
        };

        line_start.saturating_add(col).min(line_end)
    }

    /// Converts a character index into a `(row, col)` coordinate.
    pub fn row_col_from_index(&self, index: usize) -> (usize, usize) {
        let target = index.min(self.len());
        let row = match self.line_starts.binary_search(&target) {
            Ok(found_row) => found_row,
            Err(insertion_point) => insertion_point.saturating_sub(1),
        };
        let col = target.saturating_sub(self.line_starts[row]);

        (row, col)
    }

    /// Returns visible lines clipped to the requested viewport.
    ///
    /// This scans the buffer once and avoids allocating the full document text.
    pub fn for_each_visible_line<F>(
        &self,
        start_row: usize,
        max_rows: usize,
        max_cols: usize,
        mut on_line: F,
    ) where
        F: FnMut(usize, &str),
    {
        if max_rows == 0 {
            return;
        }

        let end_row = start_row.saturating_add(max_rows);
        let mut row = 0usize;
        let mut col = 0usize;
        let mut current_line = String::new();

        for ch in self.table.iter().copied() {
            if row >= end_row {
                break;
            }

            if ch == '\n' {
                if row >= start_row {
                    on_line(row - start_row, &current_line);
                    current_line.clear();
                }

                row += 1;
                col = 0;
                continue;
            }

            if row >= start_row && col < max_cols {
                current_line.push(ch);
            }
            col += 1;
        }

        if row >= start_row && row < end_row {
            on_line(row - start_row, &current_line);
        }
    }

    fn rebuild_line_starts(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0);

        for (index, ch) in self.table.iter().enumerate() {
            if *ch == '\n' {
                self.line_starts.push(index + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TextBuffer;

    fn buffer_from_text(text: &str) -> TextBuffer {
        let mut buffer = TextBuffer::default();
        buffer.insert_str(0, text);
        buffer
    }

    #[test]
    fn line_count_handles_trailing_newline() {
        let buffer = buffer_from_text("ab\ncd\n");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.line_len(0), 2);
        assert_eq!(buffer.line_len(1), 2);
        assert_eq!(buffer.line_len(2), 0);
    }

    #[test]
    fn row_col_to_index_clamps_to_line_end() {
        let buffer = buffer_from_text("abc\nxy");
        assert_eq!(buffer.index_from_row_col(0, 99), 3);
        assert_eq!(buffer.index_from_row_col(1, 99), 6);
        assert_eq!(buffer.index_from_row_col(99, 2), 6);
    }

    #[test]
    fn index_to_row_col_round_trip_examples() {
        let buffer = buffer_from_text("ab\nxyz\nq");

        let points = [(0usize, 0usize), (0, 2), (1, 0), (1, 3), (2, 1)];
        for (row, col) in points {
            let index = buffer.index_from_row_col(row, col);
            let (mapped_row, mapped_col) = buffer.row_col_from_index(index);
            assert_eq!((mapped_row, mapped_col), (row, col));
        }
    }

    #[test]
    fn delete_backward_updates_line_cache() {
        let mut buffer = buffer_from_text("a\nb");
        buffer.delete_backward(2);
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line_len(0), 2);
        assert_eq!(buffer.index_from_row_col(0, 2), 2);
    }
}
