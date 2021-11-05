
use std::borrow::Cow;
use std::cmp;

use tui::{
    layout::{ Constraint, Rect }
};

use textwrap::{ wrap };
use unicode_segmentation::UnicodeSegmentation;

use crate::constants::table_columns::{ TableColumn };
use crate::util::str::{ set_str_end_as_ellipsis };

// Accepts:
//     idx: 0-based index
//     len: length
// Returns:
//     1 if the column requires 1 unit of space for further columns, 0 if last column 
fn get_col_spacing(idx: u16, len: u16) -> u16 {
    if idx+1 < len {
        1
    } else {
        0
    }
}

// Accepts:
//     bbox: Rectangle bounding-box
//     columns: list of columns to size to bbox
// Returns: 
//     List of Width Constraints that maximize space-utilization for columns, ordered in same order as columns
pub fn widths_from_rect(bbox: &Rect, columns: &[TableColumn]) -> Vec<Constraint> {
    // Procedure: 
    //     Get Rectangle width
    //     Iterate over TableColumns and attempt to claim enough space to satisfy min-width
    //     Determine total (denominator) of 'priority' fields from all table columns
    //     Iterate over TableColumns and attempt to claim (priority/priority_sum)*width space if:
    //         (enough space remains) && (new space > min-width)
    //     Return Vec<Constraint::Length>

    // debug!("widths_from_rect - bbox.width: {:?}", bbox.width);

    // Get Rectangle width
    let mut remaining_units: u16 = bbox.width;

    let mut claimed_units: Vec<u16> = vec![0; columns.len()];


    // Iterate over TableColumns and attempt to claim enough space to satisfy min-width
    for (idx, col) in columns.iter().enumerate() {

        if remaining_units >= col.min_width {
            claimed_units[idx] = col.min_width;
            remaining_units -= col.min_width;
        } else {
            claimed_units[idx] = remaining_units;
            remaining_units -= remaining_units;
        }

        // If not last column, 1 unit will be claimed for spacing for the next column
        if remaining_units > 0 && (idx+1) < columns.len() {
            remaining_units -= 1;
        }
    }

    // debug!("widths_from_rect - remaining_units: {:?}", remaining_units);
    // debug!("widths_from_rect - claimed_units (min_widths): {:?}", claimed_units);

    // Determine total (denominator) of 'priority' fields from all table columns
    let priority_sum: u16 = columns.iter().fold(0, |acc, x| acc + x.priority);

    //     Iterate over TableColumns
    //     and attempt to claim (priority/priority_sum)*width space
    //     and ensure space remains for all min-widths of further columns
    //     
    //         if new_size > min-width && (remaining_units_without_current_col - new_size_with_col_spacing) > future_claimed_space:
    //             claim max remaining space <= new space
    //         else:
    //             continue;
    for (idx, col) in columns.iter().enumerate() {
        // (priority/priority_sum)*width
        let new_size: u16 = (((col.priority as f64) / (priority_sum as f64)) * (bbox.width as f64)).round() as u16;

        let col_spacing: u16 = get_col_spacing(idx as u16, columns.len() as u16);

        // size with column-spacing factored in
        let new_size_with_col_spacing: u16 = new_size - col_spacing;

        // debug!("widths_from_rect - remaining_units_without_current_col: {:?} + {:?} + {:?}", remaining_units, claimed_units[idx], col_spacing);

        // remaining units with current column removed
        let remaining_units_without_current_col = remaining_units + claimed_units[idx] + col_spacing;

        // debug!("widths_from_rect - idx, future_claimed_space: {:?}, {:?}", idx, future_claimed_space);

        // TODO: Instead of not modifying the size from min-width at all if new_size_with_col_spacing conditions not met,
        //     attempt to find: max(new_size) > col.min_width, that meets new_size_with_col_spacing conditions
        // what to look for:
        //     x = min((remaining_units_without_current_col - future_claimed_space), new_size_with_col_spacing)
        //
        
        // debug!("widths_from_rect - min({:?} - {:?}, {:?})", remaining_units_without_current_col, col_spacing, new_size_with_col_spacing);
        
        // upper bound: If I take out old col and add new col (with col_spacing), enough space?
        // upper bound: desired col priority relative to whole
        let greedy_new_size: u16 = cmp::min(remaining_units_without_current_col - col_spacing, new_size_with_col_spacing);

        // debug!("widths_from_rect - remaining_units_without_current_col, new_size_with_col_spacing: {:?}, {:?}", remaining_units_without_current_col, new_size_with_col_spacing);
        if greedy_new_size > col.min_width {

            // clear the currently claimed space for this column
            remaining_units += claimed_units[idx];
            
            // If not last column, also clear 1 unit claimed for spacing to next column
            remaining_units += col_spacing;

            // Assign new size

            // debug!("widths_from_rect - 1 claimed_units[{:?}] = {:?}", idx, greedy_new_size);
            claimed_units[idx] = greedy_new_size;
            remaining_units -= greedy_new_size;

            // If not last column, 1 unit will be claimed for spacing for the next column
            if remaining_units > 0 {
                remaining_units -= col_spacing;
            }
        }
    }

    // debug!("widths_from_rect - claimed_units: {:?}", claimed_units);

    claimed_units.iter()
    .map(|claimed| {
        Constraint::Length(*claimed)
    })
    .collect()

}

// Accepts:
//     content: string to be formatted
//     width: max character-width
//     height: max line-height
// Returns: 
//     formatted string with new-lines (n = height-1) inserted at character width limits
//         & ellipses appended in case of truncation
pub fn format_str_with_wrap(content: &str, width: u16, height: u16) -> String {

    let mut wrapped_str = wrap(content, width as usize);
    // debug!("height, width, wrapped_str: {:?}, {:?}, {:?}", height, width, wrapped_str);
    let wrapped_len = wrapped_str.len();
    // debug!("format_str_with_wrap - width, height, wrapped_str: {:?}, {:?}, {:?}", width, height, wrapped_str);

    // For all lines beyond wrapped_str[height-1], merge into wrapped_str[height-1]? to improve truncation output

    // Thought: iterate backwards through input string,
    // matching against the wrapped_str[idx] line and cutting sections to the right of match index from matching (copy of original) string
    // this should allow for finding the true index of the original string that is beyond wrapped_str[height-1]
    // merge everything from [idx..] into wrapped_str[height-1]

    let mut compact_search_success: bool = false;
    let mut boundary_idx: usize = 0;
    if wrapped_len as u16 > height {
        compact_search_success = true;

        let mut remaining: &str = content;

        // for i in height..(wrapped_len as u16) {
        for i in 0..(wrapped_len as u16 - height) {

            // let temp = &*wrapped_str[((wrapped_len-1) - (i-height) as usize)].to_owned();
            let temp = &*wrapped_str[((wrapped_len-1) - i as usize)].to_owned();
            let find_opt = remaining.rfind(temp);
            if let Some(x) = find_opt {
                boundary_idx = x;
                // debug!("boundary_idx: {:?}", boundary_idx);
                remaining = &content[..x];
            } else {
                // debug!("rfind() failed");
                compact_search_success = false;
                break;
            }
        }
    }

    if compact_search_success {
        // debug!("original_line_idx: {:?}", (height-1));
        let original_line = wrapped_str[(height as usize)-1].to_mut();

        // get index of final line to be rendered in original string
        if let Some(original_line_end) = content[..boundary_idx].rfind(&original_line.to_owned()) {
            // can't just merge from this point,
            // can still be spaces in the original string that are between boundary_idx & original_line
            // need to use content[ content.rfind(original_line) + original_line.len() .. boundary_idx ] for original line
            // debug!("middle[x..y] - x, y: {:?}, {:?}", original_line_end + original_line.len(), boundary_idx);
            let middle = &content[ original_line_end + original_line.len() .. boundary_idx ];

            let to_compact: &str = &content[boundary_idx..];

            original_line.push_str(middle);
            original_line.push_str(to_compact);
            // debug!("compacted string: {:?}", original_line);

            // wrapped_str[(height as usize)-1] = Cow::Borrowed();

        }
    }

    let mut result: String = "".to_owned();

    let mut ellipsis_added: bool = false;
    let mut final_line: String = String::from("");

    // If not all lines can be rendered within height
    // set last str characters to ellipsis
    if wrapped_str.len() as u16 > height {
        // if possible, add ellipsis to the end of the last line that will be rendered
        // debug!("set_str_end_as_ellipsis() -- arg, wrapped_str: {:?}, {:?}", &wrapped_str[(height as usize)-1], wrapped_str);
        if let Some(modified_line) = set_str_end_as_ellipsis(&wrapped_str[(height as usize)-1], width as usize) {
            ellipsis_added = true;
            final_line = modified_line;
            // debug!("format_str_with_wrap - width, final_line: {:?}, {:?}", width, final_line);
        }
    }

    // debug!("format_str_with_wrap - ellipsis_added, final_line: {:?}, {:?}", ellipsis_added, final_line);

    for (idx, line) in wrapped_str.iter().enumerate() {
        // bound number of lines by height
        if idx >= height as usize {
            break;
        }

        if idx == (height as usize)-1 && ellipsis_added {
            result.push_str(&final_line);
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // debug!("format_str_with_wrap - content, result: {:?}, {:?}", content, result);

    result
}