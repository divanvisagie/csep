/// Table formatting utilities
/// Calculate the maximum width needed for each column
pub fn calculate_column_widths(data: &[Vec<String>]) -> Vec<usize> {
    if data.is_empty() {
        return vec![];
    }
    
    let num_columns = data[0].len();
    let mut max_widths = vec![0; num_columns];
    
    for row in data {
        for (i, cell) in row.iter().enumerate() {
            if i < num_columns && cell.len() > max_widths[i] {
                max_widths[i] = cell.len();
            }
        }
    }
    
    max_widths
}

/// Format a table with proper padding and alignment
#[allow(dead_code)]
pub fn format_table(data: &[Vec<String>]) -> String {
    if data.is_empty() {
        return String::new();
    }
    
    let max_widths = calculate_column_widths(data);
    let mut result = String::new();
    
    for row in data {
        for (i, cell) in row.iter().enumerate() {
            if i < max_widths.len() {
                // Pad each cell to the maximum width for its column
                let padding = max_widths[i] - cell.len();
                result.push_str(cell);
                result.push_str(&" ".repeat(padding));
                
                // Add separator between columns (except last column)
                if i < row.len() - 1 {
                    result.push_str("  ");
                }
            }
        }
        result.push('\n');
    }
    
    result
}

/// Format a table with headers and separator
pub fn format_table_with_headers(headers: &[String], rows: &[Vec<String>]) -> String {
    // Calculate max widths including headers
    let mut all_data = vec![headers.to_vec()];
    all_data.extend(rows.to_vec());
    
    let max_widths = calculate_column_widths(&all_data);
    let mut result = String::new();
    
    // Format header
    for (i, header) in headers.iter().enumerate() {
        if i < max_widths.len() {
            let padding = max_widths[i] - header.len();
            result.push_str(header);
            result.push_str(&" ".repeat(padding));
            if i < headers.len() - 1 {
                result.push_str("  ");
            }
        }
    }
    result.push('\n');
    
    // Format separator line
    for (i, width) in max_widths.iter().enumerate() {
        result.push_str(&"-".repeat(*width));
        if i < max_widths.len() - 1 {
            result.push_str("--");
        }
    }
    result.push('\n');
    
    // Format data rows
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < max_widths.len() {
                let padding = max_widths[i] - cell.len();
                result.push_str(cell);
                result.push_str(&" ".repeat(padding));
                if i < row.len() - 1 {
                    result.push_str("  ");
                }
            }
        }
        result.push('\n');
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_column_widths() {
        let data = vec![
            vec!["Name".to_string(), "Description".to_string()],
            vec!["all-minilm-l6-v2".to_string(), "Small, fast model".to_string()],
            vec!["bge-small-en-v1.5".to_string(), "Modern balanced model".to_string()],
        ];
        
        let widths = calculate_column_widths(&data);
        assert_eq!(widths[0], 17); // "bge-small-en-v1.5".len()
        assert_eq!(widths[1], 19); // "Modern balanced model".len()
    }
    
    #[test]
    fn test_format_table() {
        let data = vec![
            vec!["Name".to_string(), "Category".to_string()],
            vec!["all-minilm-l6-v2".to_string(), "fast".to_string()],
            vec!["bge-small-en-v1.5".to_string(), "balanced".to_string()],
        ];
        
        let result = format_table(&data);
        println!("{}", result);
        assert!(result.contains("all-minilm-l6-v2"));
        assert!(result.contains("fast"));
    }
}
