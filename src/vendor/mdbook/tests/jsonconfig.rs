extern crate mdbook;
use mdbook::config::BookConfig;
use mdbook::config::jsonconfig::JsonConfig;

use std::path::PathBuf;

// Tests that the `src` key is correctly parsed in the JSON config
#[test]
fn from_json_source() {
    let json = r#"{
        "src": "source"
    }"#;

    let parsed = JsonConfig::from_json(json).expect("This should parse");
    let config = BookConfig::from_jsonconfig("root", parsed);

    assert_eq!(config.get_source(), PathBuf::from("root/source"));
}

// Tests that the `title` key is correctly parsed in the JSON config
#[test]
fn from_json_title() {
    let json = r#"{
        "title": "Some title"
    }"#;

    let parsed = JsonConfig::from_json(json).expect("This should parse");
    let config = BookConfig::from_jsonconfig("root", parsed);

    assert_eq!(config.get_title(), "Some title");
}

// Tests that the `description` key is correctly parsed in the JSON config
#[test]
fn from_json_description() {
    let json = r#"{
        "description": "This is a description"
    }"#;

    let parsed = JsonConfig::from_json(json).expect("This should parse");
    let config = BookConfig::from_jsonconfig("root", parsed);

    assert_eq!(config.get_description(), "This is a description");
}

// Tests that the `author` key is correctly parsed in the JSON config
#[test]
fn from_json_author() {
    let json = r#"{
        "author": "John Doe"
    }"#;

    let parsed = JsonConfig::from_json(json).expect("This should parse");
    let config = BookConfig::from_jsonconfig("root", parsed);

    assert_eq!(config.get_authors(), &[String::from("John Doe")]);
}

// Tests that the `dest` key is correctly parsed in the JSON config
#[test]
fn from_json_destination() {
    let json = r#"{
        "dest": "htmlbook"
    }"#;

    let parsed = JsonConfig::from_json(json).expect("This should parse");
    let config = BookConfig::from_jsonconfig("root", parsed);

    let htmlconfig = config.get_html_config();

    assert_eq!(htmlconfig.get_destination(), PathBuf::from("root/htmlbook"));
}

// Tests that the `theme_path` key is correctly parsed in the JSON config
#[test]
fn from_json_output_html_theme() {
    let json = r#"{
        "theme_path": "theme"
    }"#;

    let parsed = JsonConfig::from_json(json).expect("This should parse");
    let config = BookConfig::from_jsonconfig("root", parsed);

    let htmlconfig = config.get_html_config();

    assert_eq!(htmlconfig.get_theme(), &PathBuf::from("root/theme"));
}
