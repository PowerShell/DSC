use crossterm::terminal::{size};

pub struct Table {
    header: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl Table {
    /// Create a new table.
    /// 
    /// # Arguments
    /// 
    /// * `header` - The header row
    /// 
    /// # Returns
    /// 
    /// * `Table` - The new table
    #[must_use]
    pub fn new(header: &[&str]) -> Table {
        let mut column_widths = Vec::new();
        for header_text in header {
            column_widths.push(header_text.len());
        }
        let header = header.iter().map(|s| (*s).to_string()).collect::<Vec<String>>();
        Table {
            header,
            rows: Vec::new(),
            column_widths,
        }
    }

    /// Add a row to the table.
    /// 
    /// # Arguments
    /// 
    /// * `row` - The row to add
    pub fn add_row(&mut self, row: Vec<String>) {
        for (i, column) in row.iter().enumerate() {
            if column.len() > self.column_widths[i] {
                self.column_widths[i] = column.len();
            }
        }
        self.rows.push(row);
    }

    /// Print the table to the console.
    pub fn print(&self) {
        let (width, _) = size().unwrap_or((80, 25));
        // make header bright green
        println!("\x1b[1;32m");
        let mut header_row = String::with_capacity(width as usize);
        let last_column = self.header.len() - 1;
        for (i, column) in self.header.iter().enumerate() {
            header_row.push_str(&format!("{:<width$}", column, width = self.column_widths[i]));
            if i != last_column {
                header_row.push_str("  ");
            }
        }
        // if header row is too wide, truncate
        if header_row.len() > width as usize {
            header_row.truncate(width as usize);
        }

        println!("{header_row}");
        println!("{}\x1b[0m", "-".repeat(header_row.len()));

        // TODO: make this smarter by splitting long text to multiple lines
        for row in &self.rows {
            let mut row_str = String::with_capacity(header_row.len());
            for (i, column) in row.iter().enumerate() {
                row_str.push_str(&format!("{:<width$}", column, width = self.column_widths[i]));
                if i != last_column {
                    row_str.push_str("  ");
                }
            }

            // if row is too wide and last character is not a space, truncate and add ellipsis unicode character
            if row_str.len() > width as usize {
                row_str.truncate(width as usize);
                if !row_str.ends_with(' ') {
                    row_str.pop();
                    row_str.push('…');
                }
            }

            println!("{row_str}");
        }
    }
}
