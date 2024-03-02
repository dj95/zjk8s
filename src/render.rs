use anstyle::{Ansi256Color, Style};

use super::kubernetes::State;

#[derive(PartialEq, Debug)]
pub enum ColType {
    Namespace,
    ResourceType,
    Resource,
    ResourceDetails,
}

impl Default for ColType {
    fn default() -> Self {
        Self::Namespace
    }
}

#[derive(Debug)]
struct Col {
    pub max_width: usize,
    pub items: Vec<Item>,
    pub selected: bool,
}

#[derive(Debug)]
struct Item {
    pub name: String,
    pub selected: bool,
}

pub fn render_cluster_state(state: &State, selected_col: &ColType) {
    let mut output: Vec<Col> = vec![];

    if let Some(namespaces) = &state.namespaces {
        output.push(get_col(
            namespaces,
            ColType::Namespace,
            &state.selected_namespace,
            selected_col,
            "Namespaces",
        ));
    }

    if let Some(resource_types) = &state.resource_types {
        output.push(get_col(
            resource_types,
            ColType::ResourceType,
            &state.selected_resource_type,
            selected_col,
            "Resource Types",
        ));
    }

    if let Some(resources) = &state.resources {
        output.push(get_col(
            resources,
            ColType::Resource,
            &state.selected_resource,
            selected_col,
            "Resources",
        ));
    }

    if output.is_empty() {
        return;
    }

    render_table(output);
}

fn get_col(
    data: &Vec<String>,
    col_type: ColType,
    selected_data: &Option<String>,
    selected_col: &ColType,
    header: &str,
) -> Col {
    let srt = match selected_data {
        Some(rt) => rt,
        None => &data[0],
    };

    let mut items: Vec<Item> = data
        .iter()
        .map(|r| Item {
            name: r.to_string(),
            selected: r == srt,
        })
        .collect();

    items.insert(
        0,
        Item {
            name: header.to_string(),
            selected: false,
        },
    );

    Col {
        max_width: items
            .iter()
            .map(|i| console::measure_text_width(&i.name))
            .max()
            .unwrap(),
        items,
        selected: *selected_col == col_type,
    }
}

fn render_table(table: Vec<Col>) {
    let mut output_rows: Vec<String> = vec![];

    let max_row_count = table.iter().map(|c| c.items.len()).max().unwrap();

    for _ in 0..max_row_count {
        output_rows.push("".to_owned());
    }

    let normal_style = Style::new();

    let mut selected_style = Style::new();
    selected_style = selected_style.bg_color(Some(anstyle::Color::Ansi256(Ansi256Color(243))));
    selected_style = selected_style.bold();

    let mut selected_col_style = Style::new();
    selected_col_style =
        selected_col_style.bg_color(Some(anstyle::Color::Ansi256(Ansi256Color(233))));
    selected_col_style = selected_col_style.bold();

    let mut selected_col_selected_style = Style::new();
    selected_col_selected_style =
        selected_col_selected_style.bg_color(Some(anstyle::Color::Ansi256(Ansi256Color(243))));
    selected_col_selected_style = selected_col_selected_style.bold();

    for row in table {
        let mut counter = 0;

        for item in row.items {
            counter += 1;
            let space_count = row.max_width - console::measure_text_width(&item.name);

            let selected_style = match (row.selected, item.selected) {
                (true, true) => &selected_col_selected_style,
                (true, false) => &selected_col_style,
                (false, true) => &selected_style,
                (false, false) => &normal_style,
            };

            output_rows[counter - 1] = format!(
                "{}{}{} {}{} {}",
                output_rows[counter - 1],
                selected_style.render_reset(),
                selected_style.render(),
                item.name,
                " ".repeat(space_count),
                selected_style.render_reset(),
            );
        }

        for _ in counter..max_row_count {
            counter += 1;
            output_rows[counter - 1] = format!(
                "{} {} ",
                output_rows[counter - 1],
                " ".repeat(row.max_width)
            );
        }
    }

    for output_row in output_rows {
        println!("{}", output_row);
    }
}
