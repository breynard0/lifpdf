use crate::flag::is_time_discrepancy;
use crate::parse::*;
use crate::table_data::gen_table_row;
use hayro::{RenderSettings, render};
use hayro_interpret::InterpreterSettings;
use oxidize_pdf::text::table::GridStyle;
use oxidize_pdf::{Color, Document, Font, HeaderStyle, Page, Table, TableOptions, TextAlign};
use std::sync::Arc;

pub fn gen_timesheet_pdf(event: RaceEvent) -> Result<Document, Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    doc.set_title(event.event.event_name);
    let mut pages = vec![Page::a4()];

    let width = pages[0].width();
    let height = pages[0].height();

    // Title
    let mut flow = pages[0].text_flow();
    flow.at(0.0, height * 0.95)
        .set_font(Font::HelveticaBold, 10.0)
        .set_alignment(TextAlign::Center)
        .write_wrapped(&format!("{} - Results", event.event.event_code))?
        .write_paragraph(&format!("Start: {}", event.event.start_time))?;
    pages[0].add_text_flow(&flow);

    // Skaters Table
    // Splitting into two in order to not have a line above the table
    let mut skater_table_header = Table::with_equal_columns(7, width * 0.95);
    let mut skater_table = Table::with_equal_columns(7, width * 0.95);

    for competitor in event.competitors.iter().rev() {
        skater_table
            .add_row_with_alignment(gen_table_row(competitor.clone()), TextAlign::Center)?;
    }

    skater_table_header.add_header_row(vec![
        "Place".to_string(),
        "ID".to_string(),
        "Lane".to_string(),
        "First Name".to_string(),
        "Last Name".to_string(),
        "Affiliation".to_string(),
        "Time (PF)".to_string(),
    ])?;

    skater_table_header.set_position(
        (width * 0.05) / 2.0,
        flow.cursor_position().1 - skater_table_header.get_height(),
    );
    skater_table.set_position(
        (width * 0.05) / 2.0,
        flow.cursor_position().1 - skater_table.get_height() - skater_table_header.get_height(),
    );

    skater_table_header.set_options(TableOptions {
        grid_style: GridStyle::None,
        header_style: Some(HeaderStyle {
            bold: true,
            font: Font::HelveticaBold,
            background_color: Color::white(),
            text_color: Color::black(),
        }),
        ..Default::default()
    });
    skater_table.set_options(TableOptions {
        grid_style: GridStyle::Horizontal,
        ..Default::default()
    });

    pages[0].add_table(&skater_table_header)?;
    pages[0].add_table(&skater_table)?;

    let flags_y = flow.cursor_position().1
        - skater_table.get_height()
        - skater_table_header.get_height()
        - 20.0;

    // Flag values that seem incorrect
    let mut flags_space_taken = 0.0;
    for competitor in &event.competitors {
        if let Some(pdf_time) = competitor.time {
            let has_discrepancy = is_time_discrepancy(pdf_time, &competitor.splits);
            if has_discrepancy {
                flags_space_taken += pages[0]
                    .text()
                    .at(width * 0.05, flags_y - flags_space_taken)
                    .set_font(Font::HelveticaBold, 10.0)
                    .write_line(&format!(
                        "Potential mismatch, lane {}, place {}",
                        competitor.lane.unwrap_or(255),
                        competitor.place.unwrap_or(255)
                    ))?
                    .font_size();
            }
        }
    }

    let transponder_y = flags_y - flags_space_taken - 20.0;
    pages[0]
        .text()
        .at(width * 0.05, transponder_y)
        .set_font(Font::HelveticaBold, 10.0)
        .write_line("Transponder Times")?;

    let mut transponder_table =
        Table::with_equal_columns(event.competitors.len() + 1, width * 0.95);

    let mut rows_left = true;
    let mut i = 0;
    let mut rows = vec![];
    while rows_left {
        let mut cur_row = vec![(i + 1).to_string()];

        rows_left = false;
        for competitor in &event.competitors {
            if i < competitor.splits.len() {
                rows_left = true;
                if competitor.splits[i].subsecond >= 0.0 {
                    cur_row.push(competitor.splits[i].to_string());
                } else {
                    cur_row.push("Junk".to_string())
                }
            } else {
                cur_row.push(String::new());
            }
        }

        if rows_left {
            rows.push(cur_row);
        }

        i += 1;
    }

    let mut number = 0;
    let mut counting_transp_table = Table::with_equal_columns(1, 10.0);
    while transponder_y - 20.0 - counting_transp_table.get_height() > height * 0.1 {
        counting_transp_table.add_row(vec![String::new()])?;
        number += 1;
    }

    rows.reverse();
    let (rows_b, rows_a) = match rows.len() > number {
        true => {
            let split = rows.split_at(rows.len() - number);
            (split.0.to_vec(), split.1.to_vec())
        }
        false => (vec![], rows),
    };

    for row in rows_a {
        number += 1;
        transponder_table.add_row(row.clone())?;
    }

    if rows_b.len() != 0 {
        pages.push(Page::a4());
        let mut other_table = Table::with_equal_columns(event.competitors.len() + 1, width * 0.95);

        for row in rows_b {
            other_table.add_row(row.clone())?;
        }

        other_table.set_position(
            (width * 0.05) / 2.0,
            pages[1].height() * 0.95 - other_table.get_height(),
        );

        pages[1].add_table(&other_table)?;
    }

    let mut transponder_headers = vec!["Place".to_string()];
    for i in 1..=event.competitors.len() {
        transponder_headers.push(i.to_string());
    }
    transponder_table.add_header_row(transponder_headers)?;

    transponder_table.set_position(
        (width * 0.05) / 2.0,
        transponder_y - transponder_table.get_height() - 20.0,
    );

    transponder_table.set_options(TableOptions {
        header_style: Some(HeaderStyle {
            bold: true,
            font: Font::HelveticaBold,
            background_color: Color::white(),
            text_color: Color::black(),
        }),
        ..Default::default()
    });

    pages[0].add_table(&transponder_table)?;

    for page in pages {
        doc.add_page(page);
    }
    doc.save("./out.pdf").unwrap();

    Ok(doc)
}

pub fn pdf_to_image(
    document: &mut Document,
) -> Result<(Vec<Vec<u8>>, u32, u32), Box<dyn std::error::Error>> {
    let mut buffer = vec![];
    document.write(&mut buffer)?;

    let data = Arc::new(buffer);
    let hayro_pdf = hayro::Pdf::new(data).expect("Internal movement of PDF data somehow failed");

    let interpreter_settings = InterpreterSettings::default();

    let render_upscaling = 4.0;

    let render_settings = RenderSettings {
        x_scale: render_upscaling,
        y_scale: render_upscaling,
        ..Default::default()
    };

    let mut out = vec![];
    for page in hayro_pdf.pages().iter() {
        let pixmap = render(page, &interpreter_settings, &render_settings);
        let vec1 = pixmap.take_u8();
        out.push(vec1);
        // std::fs::write("./out.png", pixmap.take_png()).unwrap();
    }

    Ok((
        out,
        (render_upscaling * hayro_pdf.pages()[0].render_dimensions().0) as u32,
        (render_upscaling * hayro_pdf.pages()[0].render_dimensions().1) as u32,
    ))
}
