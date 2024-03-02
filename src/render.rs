use miette::{Diagnostic, NamedSource, Report, Result, SourceSpan};
use std::cmp;
use thiserror::Error;

use anstyle::{Ansi256Color, AnsiColor, Color, RgbColor, Style};

use super::kubernetes::State;

#[derive(Error, Debug, Diagnostic)]
#[error("Invalid Color")]
#[diagnostic(help("Color cannot be coverted from hex"))]
struct InvalidColor {
    #[source_code]
    src: NamedSource<String>,

    #[label("This error occured")]
    bad_bit: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Missing configuration")]
#[diagnostic(help("Configuration value is missing"))]
struct MissingConfigValue {
    #[source_code]
    src: NamedSource<String>,

    #[label("This error occured")]
    bad_bit: SourceSpan,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Debug, Copy, Clone)]
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

#[derive(Debug, Clone)]
struct Item {
    pub name: String,
    pub selected: bool,
}

#[derive(Debug, Default)]
pub struct Render {
    normal_style: Style,
    selected_style: Style,
    selected_col_style: Style,
    selected_col_selected_style: Style,
}

impl Render {
    pub fn new(
        selected_bg: Option<&str>,
        selected_col_bg: Option<&str>,
        selected_col_selected_bg: Option<&str>,
    ) -> Result<Self> {
        let normal_style = Style::new();

        let selected_bg = match selected_bg {
            Some(s) => parse_color(s)?,
            None => {
                return Err(MissingConfigValue {
                    src: NamedSource::new(
                        "layout.kdl",
                        "\"selected_item_bg\" is missing".to_string(),
                    ),
                    bad_bit: (0, 0).into(),
                }
                .into());
            }
        };

        let selected_col_bg = match selected_col_bg {
            Some(s) => parse_color(s)?,
            None => {
                return Err(MissingConfigValue {
                    src: NamedSource::new(
                        "layout.kdl",
                        "\"selected_col_item_bg\" is missing".to_string(),
                    ),
                    bad_bit: (0, 0).into(),
                }
                .into());
            }
        };

        let selected_col_selected_bg = match selected_col_selected_bg {
            Some(s) => parse_color(s)?,
            None => {
                return Err(MissingConfigValue {
                    src: NamedSource::new(
                        "layout.kdl",
                        "\"selected_col_selected_item_bg\" is missing".to_string(),
                    ),
                    bad_bit: (0, 0).into(),
                }
                .into());
            }
        };

        let mut selected_style = Style::new();
        selected_style = selected_style.bg_color(Some(selected_bg));
        selected_style = selected_style.bold();

        let mut selected_col_style = Style::new();
        selected_col_style = selected_col_style.bg_color(Some(selected_col_bg));
        selected_col_style = selected_col_style.bold();

        let mut selected_col_selected_style = Style::new();
        selected_col_selected_style =
            selected_col_selected_style.bg_color(Some(selected_col_selected_bg));
        selected_col_selected_style = selected_col_selected_style.bold();

        Ok(Self {
            normal_style,
            selected_style,
            selected_col_style,
            selected_col_selected_style,
        })
    }

    pub fn render_cluster_state(
        &mut self,
        state: &State,
        selected_col: &ColType,
        rows: usize,
        cols: usize,
    ) {
        let mut output: Vec<Col> = vec![];

        if let Some(namespaces) = &state.namespaces {
            output.push(self.get_col(
                namespaces,
                ColType::Namespace,
                &state.selected_namespace,
                selected_col,
                "Namespaces",
                rows,
            ));
        }

        if let Some(resource_types) = &state.resource_types {
            output.push(self.get_col(
                resource_types,
                ColType::ResourceType,
                &state.selected_resource_type,
                selected_col,
                "Resource Types",
                rows,
            ));
        }

        if let Some(resources) = &state.resources {
            output.push(self.get_col(
                resources,
                ColType::Resource,
                &state.selected_resource,
                selected_col,
                "Resources",
                rows,
            ));
        }

        if let Some(resource_details) = &state.resource_details {
            output.push(self.get_col(
                resource_details,
                ColType::ResourceDetails,
                &state.selected_resource_details_line,
                selected_col,
                "Resource Details",
                rows,
            ));
        }

        if output.is_empty() {
            return;
        }

        self.render_table(output, cols);
    }

    fn get_col(
        &mut self,
        data: &[String],
        col_type: ColType,
        selected_data_index: &Option<usize>,
        selected_col: &ColType,
        header: &str,
        rows: usize,
    ) -> Col {
        let srt = match selected_data_index {
            Some(rt) => rt,
            None => &0,
        };

        let mut items: Vec<Item> = data
            .iter()
            .map(|r| Item {
                name: r.to_string(),
                selected: false,
            })
            .collect();

        items[*srt].selected = true;

        if items.len() > rows {
            let scroll = cmp::min(
                srt.saturating_sub(rows / 2),
                items.len().saturating_sub(rows - 2),
            );

            items = items[scroll..scroll + rows - 2].to_vec();
        }

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

    fn render_table(&mut self, table: Vec<Col>, cols: usize) {
        let mut output_rows: Vec<String> = vec![];

        let max_row_count = table.iter().map(|c| c.items.len()).max().unwrap();

        for _ in 0..max_row_count {
            output_rows.push("".to_owned());
        }

        let mut col_counter = 0;
        for col in &table {
            let mut counter = 0;
            col_counter += 1;
            let last_col = col_counter == table.len();

            for item in &col.items {
                counter += 1;
                let mut space_count = col.max_width - console::measure_text_width(&item.name);
                if last_col {
                    space_count = cols.saturating_sub(
                        console::measure_text_width(&item.name)
                            + console::measure_text_width(&output_rows[counter - 1])
                            + 2,
                    );
                }

                let selected_style = match (col.selected, item.selected) {
                    (true, true) => &self.selected_col_selected_style,
                    (true, false) => &self.selected_col_style,
                    (false, true) => &self.selected_style,
                    (false, false) => &self.normal_style,
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

            for i in counter + 1..=max_row_count {
                output_rows[i - 1] =
                    format!("{} {} ", output_rows[i - 1], " ".repeat(col.max_width));
            }
        }

        for output_row in output_rows {
            let mut print_text = output_row.clone();
            if console::measure_text_width(&output_row) > cols {
                print_text = console::truncate_str(&output_row, cols, "").to_string();
            }

            println!("{}{}", print_text, self.selected_col_style.render_reset());
        }
    }
}

fn hex_to_rgb(s: &str) -> Result<Vec<u8>> {
    if s.len() != 6 {
        return Err(InvalidColor {
            src: NamedSource::new(
                "render.rs",
                format!("hex string \"{}\" does not contain 6 characters", s),
            ),
            bad_bit: (0, s.len()).into(),
        }
        .into());
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(Report::msg))
        .collect()
}

fn parse_color(color: &str) -> Result<Color> {
    if color.starts_with('#') {
        let rgb = match hex_to_rgb(color.strip_prefix('#').unwrap()) {
            Ok(rgb) => rgb,
            Err(e) => {
                return Err(e);
            }
        };

        if rgb.len() != 3 {
            return Err(InvalidColor {
                src: NamedSource::new(
                    "render.rs",
                    format!("rgb does not contain 3 values for input \"{}\"", color),
                ),
                bad_bit: (0, color.len()).into(),
            }
            .into());
        }

        return Ok(RgbColor(
            *rgb.first().unwrap(),
            *rgb.get(1).unwrap(),
            *rgb.get(2).unwrap(),
        )
        .into());
    }

    if let Some(color) = color_by_name(color) {
        return Ok(color.into());
    }

    if let Ok(result) = color.parse::<u8>() {
        return Ok(Ansi256Color(result).into());
    }

    Err(InvalidColor {
        src: NamedSource::new(
            "render.rs",
            format!("color \"{}\" cannot be converted or found", color),
        ),
        bad_bit: (0, color.len()).into(),
    }
    .into())
}

fn color_by_name(color: &str) -> Option<AnsiColor> {
    match color {
        "black" => Some(AnsiColor::Black),
        "red" => Some(AnsiColor::Red),
        "green" => Some(AnsiColor::Green),
        "yellow" => Some(AnsiColor::Yellow),
        "blue" => Some(AnsiColor::Blue),
        "magenta" => Some(AnsiColor::Magenta),
        "cyan" => Some(AnsiColor::Cyan),
        "white" => Some(AnsiColor::White),
        _ => None,
    }
}
