#[macro_use]
extern crate rocket;
use rocket::response::content::RawHtml;
use rocket::serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Group {
    name: String,
    description: String,
    url: String,
    tags: Vec<String>,
}

#[derive(FromForm)]
struct SearchQuery {
    term: Option<String>,
}

#[get("/")]
fn index() -> RawHtml<String> {
    render_groups(None)
}

#[get("/search?<term>")]
fn search(term: Option<String>) -> RawHtml<String> {
    render_groups(term)
}

fn render_groups(search_term: Option<String>) -> RawHtml<String> {
    let path = Path::new("groups.json");
    let file = File::open(&path).expect("File not found");
    let reader = BufReader::new(file);
    let groups: Vec<Group> = serde_json::from_reader(reader).expect("Error reading JSON");

    let filtered_groups: Vec<Group> = if let Some(term) = search_term.clone() {
        let terms: Vec<String> = term
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();
        groups
            .into_iter()
            .filter(|group| {
                terms.iter().any(|term| {
                    if term.starts_with("tag:") {
                        let tag_term = &term[4..];
                        group
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(tag_term))
                    } else {
                        group.name.to_lowercase().contains(term)
                            || group.description.to_lowercase().contains(term)
                            || group
                                .tags
                                .iter()
                                .any(|tag| tag.to_lowercase().contains(term))
                    }
                })
            })
            .collect()
    } else {
        groups
    };

    let mut groups_html = String::new();
    if filtered_groups.is_empty() {
        groups_html.push_str("<tr><td colspan=\"4\">No results found</td></tr>");
    } else {
        for group in filtered_groups {
            let description = replace_markdown(&group.description);
            let highlighted_name = if let Some(ref term) = search_term {
                highlight_matches(&group.name, term)
            } else {
                group.name.clone()
            };
            let highlighted_description = if let Some(ref term) = search_term {
                highlight_matches(&description, term)
            } else {
                description
            };

            let tags_html = group
                .tags
                .iter()
                .map(|tag| format!("<li>{}</li>", tag))
                .collect::<Vec<String>>()
                .join("");

            groups_html.push_str(&format!(
                "<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td><ul>{}</ul></td>
                    <td><a href=\"{}\">Join Group</a></td>
                </tr>",
                highlighted_name, highlighted_description, tags_html, group.url
            ));
        }
    }

    let mut template = String::new();
    let template_path = Path::new("index.html");
    let mut template_file = File::open(&template_path).expect("Template file not found");
    template_file
        .read_to_string(&mut template)
        .expect("Error reading template file");

    let html = template.replace("{{groups}}", &groups_html);

    RawHtml(html)
}

fn highlight_matches(text: &str, term: &str) -> String {
    let term_lower = term.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut highlighted = String::new();
    let mut last_index = 0;

    for (index, _) in text_lower.match_indices(&term_lower) {
        highlighted.push_str(&text[last_index..index]);
        highlighted.push_str(&format!(
            "<mark>{}</mark>",
            &text[index..index + term.len()]
        ));
        last_index = index + term.len();
    }

    highlighted.push_str(&text[last_index..]);
    highlighted
}

fn replace_markdown(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '*' {
            result.push_str("<b>");
            while let Some(&next_c) = chars.peek() {
                if next_c == '*' {
                    chars.next();
                    result.push_str("</b>");
                    break;
                } else {
                    result.push(next_c);
                    chars.next();
                }
            }
        } else if c == '_' {
            result.push_str("<i>");
            while let Some(&next_c) = chars.peek() {
                if next_c == '_' {
                    chars.next();
                    result.push_str("</i>");
                    break;
                } else {
                    result.push(next_c);
                    chars.next();
                }
            }
        } else if c == '!' {
            if let Some(&next_c) = chars.peek() {
                if next_c.is_digit(10) {
                    let color_code = chars.next().unwrap();
                    let color = match color_code {
                        '1' => "red",
                        '2' => "lime",
                        '3' => "dodgerblue",
                        '4' => "goldenrod",
                        '5' => "lightblue",
                        '6' => "magenta",
                        '7' => "pink",
                        '8' => "brown",
                        '9' => "black",
                        _ => "",
                    };
                    result.push_str(&format!("<span style=\"color:{}\">", color));
                    while let Some(&next_c) = chars.peek() {
                        if next_c == '!' {
                            chars.next();
                            result.push_str("</span>");
                            break;
                        } else {
                            result.push(next_c);
                            chars.next();
                        }
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, search])
}
