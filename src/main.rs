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

    let filtered_groups: Vec<Group> = if let Some(term) = search_term {
        let term_lower = term.to_lowercase();
        groups
            .into_iter()
            .filter(|group| {
                group.name.to_lowercase().contains(&term_lower)
                    || group
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&term_lower))
            })
            .collect()
    } else {
        groups
    };

    let mut groups_html = String::new();
    for group in filtered_groups {
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
            group.name, group.description, tags_html, group.url
        ));
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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, search])
}
