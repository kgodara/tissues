

// Spec mapping fields desired for modal and gql queries

// Target Fields:
/*
    Team
    Creation Date

    Title
    Description

    Labels


    Assignee
    Creator
    Status
    Priority
    Cycle
    Project
*/

// Fields only used for expanded view:
//     Title
//     Description
//     Creation Date
//     various "name"/"color" fields for objects

// indexes:
/*
["team"]["name"] --> TODO: team name needs to be fetched
["createdAt"]

["title"]
["description"]

["labels"]["nodes"]["name"] --> TODO: label name & color needs to be fetched

["assignee"]["displayName"] --> TODO: assignee displayName needs to be fetched
["creator"]["displayName"] --> TODO: creator displayName needs to be fetched
["state"]["name"]
["priority"] --> Improve this at some point?
["cycle"] --> TODO: need to fetch cycle object "id" & "name"
["project"] --> TODO: need to fetch project: "id", "name", "color"
*/





use tui::{
    backend::Backend,
    layout::{Constraint, Rect, Layout, Direction, Alignment},
    style::{Color, Modifier, Style},
    text::{ Span, Spans },
    widgets::{Block, Borders, BorderType, Row, Table, Cell, Paragraph, Wrap },
    Frame
};

use crate::util::{
    table::{ empty_str_to_fallback,
        format_cell_fields,
        get_row_height,
    },
    ui::{ style_color_from_hex_str },
    layout::{ widths_from_rect },
};

use crate::linear::types::{ Label, Issue };

use crate::constants::{ 
    table_columns::{ ISSUE_MODAL_HEADER_COLUMNS }
};

pub fn render_and_layout<B>(f: &mut Frame<B>, chunk: Rect, issue: &Issue, scroll_tick: u64 )
where
  B: Backend,
{
    // Render border around issue modal
    let border_div = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(chunk);
    
    let border_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));
    
    let inner_rect = border_block.inner(border_div[0]);
    f.render_widget(border_block, border_div[0]);


    let header_div = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(inner_rect);


    // render team and createdAt fields in header
    let widths: Vec<Constraint> = widths_from_rect( &header_div[0], &*ISSUE_MODAL_HEADER_COLUMNS);

    let cell_fields: Vec<String> = empty_str_to_fallback(
        &[
            issue.team.name.as_deref().unwrap_or(""),
            &issue.created_at.clone(),
        ],
        &ISSUE_MODAL_HEADER_COLUMNS
    );

    // Get the formatted Strings for each cell field
    // no need to use row_min_render_height() here, ISSUE_MODAL_HEADER_COLUMNS all have max_height = 1
    let cell_fields_formatted: Vec<String> = format_cell_fields(&cell_fields, &widths, &ISSUE_MODAL_HEADER_COLUMNS, None);
    let row_size: usize = get_row_height(&cell_fields_formatted);
    let cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

    let mut row = Row::new(cells).bottom_margin(0);
    row = row.height(row_size as u16);

    let table_block = Block::default()
        .borders(Borders::NONE)
        .border_style(Style::default());

    let t = Table::new(vec![row])
        .block(table_block)
        .widths(&widths);

    f.render_widget(t, header_div[0]);


    // Separate rest of content into two columns, one for title/desc, another for categorical info

    let content_and_categories_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(header_div[1]);

    // render projects, assignee, creator, etc.
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(15), Constraint::Percentage(14), Constraint::Percentage(14),
            Constraint::Percentage(14), Constraint::Percentage(14), Constraint::Percentage(14)
            ])
        .split(content_and_categories_cols[1]);
    
    let create_block = |title| {
        Block::default()
            .borders(Borders::TOP)
            .style(Style::default()/*.bg(Color::White).fg(Color::Black)*/)
            .title(Span::styled(title, Style::default().add_modifier(Modifier::BOLD)))
    };

    let create_colored_p = |data: String, title_str: String, hex_str_opt: Option<String>| {
        let mut p = Paragraph::new(data)
            .block(create_block(title_str))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        if let Some(hex_str) = hex_str_opt {
            if let Some(fg_color) = style_color_from_hex_str(&hex_str) {
                p = p.style(Style::default().fg(fg_color))
            }
        }
        p
    };

    let labels_vec: Vec<Label> = issue.labels.clone();

    let labels_spans: Vec<Spans> = labels_vec.iter()
        .map(|label_obj| {
            let mut span_style = Style::default();
            if let Some(label_color) = style_color_from_hex_str(label_obj.color.as_deref().unwrap_or("")) {
                span_style = span_style.fg(label_color);
            }
            Spans::from(Span::styled(&*label_obj.name.as_deref().unwrap_or(""), span_style))
        })
        .collect();

    let num_labels: u64 = labels_spans.len() as u64;

    let mut labels_p = Paragraph::new(labels_spans)
        .block(create_block(String::from("Labels")))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    
    if num_labels > 1 {
        labels_p = labels_p.scroll(( ((scroll_tick/3)%(num_labels-1)) as u16, 0));
    }

    f.render_widget(labels_p, content_chunks[0]);

    f.render_widget(
        create_colored_p(if let Some(name) = issue.assignee.as_ref().and_then(|x| x.display_name.clone()) { name.clone() } else { String::from("") },
            String::from("Assignee"),
            None),
        content_chunks[1]);
    
    f.render_widget(
        create_colored_p(if let Some(name) = issue.creator.as_ref().and_then(|x| x.display_name.clone()) { name.clone() } else { String::from("") },
            String::from("Creator"),
            None),
        content_chunks[2]);

    f.render_widget(create_colored_p(issue.state.name.clone(), String::from("State"), Some(issue.state.color.clone())),
        content_chunks[3]);
    
    f.render_widget(create_colored_p(issue.priority.to_string(), String::from("Priority"), None),
        content_chunks[4]);
    
    f.render_widget(
        create_colored_p(if let Some(name) = issue.cycle.name.as_ref() { name.clone() } else { String::from("") },
            String::from("Cycle"),
            None),
        content_chunks[5]);
    
    f.render_widget(
        create_colored_p(if let Some(name) = issue.project.as_ref().and_then(|obj| obj.name.clone()) { name.clone() } else { String::from("") },
            String::from("Project"),
            Some(if let Some(color) = issue.project.as_ref().and_then(|obj| obj.color.clone()) { color.clone() } else { String::from("") })),
        content_chunks[6]);    


    // render title & desc
    let content_div = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(content_and_categories_cols[0]);

    let title_p = Paragraph::new(issue.title.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });


    let desc_p = Paragraph::new(issue.description.as_ref().unwrap_or(&String::from("")).clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(title_p, content_div[0]);
    f.render_widget(desc_p, content_div[1]);




}
