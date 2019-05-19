extern crate mdbook;
#[macro_use]
extern crate pretty_assertions;
extern crate select;
extern crate tempfile;
extern crate walkdir;

mod dummy_book;

use dummy_book::{assert_contains_strings, assert_doesnt_contain_strings, DummyBook};

use mdbook::config::Config;
use mdbook::errors::*;
use mdbook::utils::fs::{file_to_string, write_file};
use mdbook::MDBook;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::Builder as TempFileBuilder;
use walkdir::{DirEntry, WalkDir};

const BOOK_ROOT: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/dummy_book");
const TOC_TOP_LEVEL: &[&'static str] = &[
    "1. First Chapter",
    "2. Second Chapter",
    "Conclusion",
    "Dummy Book",
    "Introduction",
];
const TOC_SECOND_LEVEL: &[&'static str] =
    &["1.1. Nested Chapter", "1.2. Includes", "2.1. Nested Chapter", "1.3. Recursive"];

/// Make sure you can load the dummy book and build it without panicking.
#[test]
fn build_the_dummy_book() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();

    md.build().unwrap();
}

#[test]
fn by_default_mdbook_generates_rendered_content_in_the_book_directory() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();

    assert!(!temp.path().join("book").exists());
    md.build().unwrap();

    assert!(temp.path().join("book").exists());
    let index_file = md.build_dir_for("html").join("index.html");
    assert!(index_file.exists());
}

#[test]
fn make_sure_bottom_level_files_contain_links_to_chapters() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let dest = temp.path().join("book");
    let links = vec![
        r#"href="intro.html""#,
        r#"href="first/index.html""#,
        r#"href="first/nested.html""#,
        r#"href="second.html""#,
        r#"href="conclusion.html""#,
    ];

    let files_in_bottom_dir = vec!["index.html", "intro.html", "second.html", "conclusion.html"];

    for filename in files_in_bottom_dir {
        assert_contains_strings(dest.join(filename), &links);
    }
}

#[test]
fn check_correct_cross_links_in_nested_dir() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let first = temp.path().join("book").join("first");
    let links = vec![
        r#"href="../intro.html""#,
        r#"href="../first/index.html""#,
        r#"href="../first/nested.html""#,
        r#"href="../second.html""#,
        r#"href="../conclusion.html""#,
    ];

    let files_in_nested_dir = vec!["index.html", "nested.html"];

    for filename in files_in_nested_dir {
        assert_contains_strings(first.join(filename), &links);
    }

    assert_contains_strings(
        first.join("index.html"),
        &[r##"href="#some-section" id="some-section""##],
    );

    assert_contains_strings(
        first.join("nested.html"),
        &[r##"href="#some-section" id="some-section""##],
    );
}

#[test]
fn check_correct_relative_links_in_print_page() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let first = temp.path().join("book");

    assert_contains_strings(
        first.join("print.html"),
        &[r##"<a href="second/../first/nested.html">the first section</a>,"##],
    );
}

#[test]
fn rendered_code_has_playpen_stuff() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let nested = temp.path().join("book/first/nested.html");
    let playpen_class = vec![r#"class="playpen""#];

    assert_contains_strings(nested, &playpen_class);

    let book_js = temp.path().join("book/book.js");
    assert_contains_strings(book_js, &[".playpen"]);
}

#[test]
fn chapter_content_appears_in_rendered_document() {
    let content = vec![
        ("index.html", "This file is just here to cause the"),
        ("intro.html", "Here's some interesting text"),
        ("second.html", "Second Chapter"),
        ("first/nested.html", "testable code"),
        ("first/index.html", "more text"),
        ("conclusion.html", "Conclusion"),
    ];

    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let destination = temp.path().join("book");

    for (filename, text) in content {
        let path = destination.join(filename);
        assert_contains_strings(path, &[text]);
    }
}

/// Apply a series of predicates to some root predicate, where each
/// successive predicate is the descendant of the last one. Similar to how you
/// might do `ul.foo li a` in CSS to access all anchor tags in the `foo` list.
macro_rules! descendants {
    ($root:expr, $($child:expr),*) => {
        $root
        $(
            .descendant($child)
        )*
    };
}

/// Make sure that all `*.md` files (excluding `SUMMARY.md`) were rendered
/// and placed in the `book` directory with their extensions set to `*.html`.
#[test]
fn chapter_files_were_rendered_to_html() {
    let temp = DummyBook::new().build().unwrap();
    let src = Path::new(BOOK_ROOT).join("src");

    let chapter_files = WalkDir::new(&src)
        .into_iter()
        .filter_entry(|entry| entry_ends_with(entry, ".md"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| path.file_name().and_then(OsStr::to_str) != Some("SUMMARY.md"));

    for chapter in chapter_files {
        let rendered_location = temp
            .path()
            .join(chapter.strip_prefix(&src).unwrap())
            .with_extension("html");
        assert!(
            rendered_location.exists(),
            "{} doesn't exits",
            rendered_location.display()
        );
    }
}

fn entry_ends_with(entry: &DirEntry, ending: &str) -> bool {
    entry.file_name().to_string_lossy().ends_with(ending)
}

/// Read the main page (`book/index.html`) and expose it as a DOM which we
/// can search with the `select` crate
fn root_index_html() -> Result<Document> {
    let temp = DummyBook::new()
        .build()
        .chain_err(|| "Couldn't create the dummy book")?;
    MDBook::load(temp.path())?
        .build()
        .chain_err(|| "Book building failed")?;

    let index_page = temp.path().join("book").join("index.html");
    let html = file_to_string(&index_page).chain_err(|| "Unable to read index.html")?;

    Ok(Document::from(html.as_str()))
}

#[test]
fn check_second_toc_level() {
    let doc = root_index_html().unwrap();
    let mut should_be = Vec::from(TOC_SECOND_LEVEL);
    should_be.sort();

    let pred = descendants!(Class("chapter"), Name("li"), Name("li"), Name("a"));

    let mut children_of_children: Vec<_> = doc
        .find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children_of_children.sort();

    assert_eq!(children_of_children, should_be);
}

#[test]
fn check_first_toc_level() {
    let doc = root_index_html().unwrap();
    let mut should_be = Vec::from(TOC_TOP_LEVEL);

    should_be.extend(TOC_SECOND_LEVEL);
    should_be.sort();

    let pred = descendants!(Class("chapter"), Name("li"), Name("a"));

    let mut children: Vec<_> = doc
        .find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children.sort();

    assert_eq!(children, should_be);
}

#[test]
fn check_spacers() {
    let doc = root_index_html().unwrap();
    let should_be = 1;

    let num_spacers = doc
        .find(Class("chapter").descendant(Name("li").and(Class("spacer"))))
        .count();
    assert_eq!(num_spacers, should_be);
}

/// Ensure building fails if `create-missing` is false and one of the files does
/// not exist.
#[test]
fn failure_on_missing_file() {
    let temp = DummyBook::new().build().unwrap();
    fs::remove_file(temp.path().join("src").join("intro.md")).unwrap();

    let mut cfg = Config::default();
    cfg.build.create_missing = false;

    let got = MDBook::load_with_config(temp.path(), cfg);
    assert!(got.is_err());
}

/// Ensure a missing file is created if `create-missing` is true.
#[test]
fn create_missing_file_with_config() {
    let temp = DummyBook::new().build().unwrap();
    fs::remove_file(temp.path().join("src").join("intro.md")).unwrap();

    let mut cfg = Config::default();
    cfg.build.create_missing = true;

    assert!(!temp.path().join("src").join("intro.md").exists());
    let _md = MDBook::load_with_config(temp.path(), cfg).unwrap();
    assert!(temp.path().join("src").join("intro.md").exists());
}

/// This makes sure you can include a Rust file with `{{#playpen example.rs}}`.
/// Specification is in `book-example/src/format/rust.md`
#[test]
fn able_to_include_playpen_files_in_chapters() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let second = temp.path().join("book/second.html");

    let playpen_strings = &[
        r#"class="playpen""#,
        r#"println!(&quot;Hello World!&quot;);"#,
    ];

    assert_contains_strings(&second, playpen_strings);
    assert_doesnt_contain_strings(&second, &["{{#playpen example.rs}}"]);
}

/// This makes sure you can include a Rust file with `{{#include ../SUMMARY.md}}`.
#[test]
fn able_to_include_files_in_chapters() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let includes = temp.path().join("book/first/includes.html");

    let summary_strings = &["<h1>Summary</h1>", ">First Chapter</a>"];
    assert_contains_strings(&includes, summary_strings);

    assert_doesnt_contain_strings(&includes, &["{{#include ../SUMMARY.md::}}"]);
}

/// Ensure cyclic includes are capped so that no exceptions occur
#[test]
fn recursive_includes_are_capped() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let recursive = temp.path().join("book/first/recursive.html");
    let content = &["Around the world, around the world
Around the world, around the world
Around the world, around the world"];
    assert_contains_strings(&recursive, content);
}

#[test]
fn example_book_can_build() {
    let example_book_dir = dummy_book::new_copy_of_example_book().unwrap();

    let md = MDBook::load(example_book_dir.path()).unwrap();

    md.build().unwrap();
}

#[test]
fn book_with_a_reserved_filename_does_not_build() {
    let tmp_dir = TempFileBuilder::new().prefix("mdBook").tempdir().unwrap();
    let src_path = tmp_dir.path().join("src");
    fs::create_dir(&src_path).unwrap();

    let summary_path = src_path.join("SUMMARY.md");
    let print_path = src_path.join("print.md");

    fs::File::create(print_path).unwrap();
    let mut summary_file = fs::File::create(summary_path).unwrap();
    writeln!(summary_file, "[print](print.md)").unwrap();

    let md = MDBook::load(tmp_dir.path()).unwrap();
    let got = md.build();
    assert!(got.is_err());
}

#[test]
fn by_default_mdbook_use_index_preprocessor_to_convert_readme_to_index() {
    let temp = DummyBook::new().build().unwrap();
    let mut cfg = Config::default();
    cfg.set("book.src", "src2")
        .expect("Couldn't set config.book.src to \"src2\".");
    let md = MDBook::load_with_config(temp.path(), cfg).unwrap();
    md.build().unwrap();

    let first_index = temp.path().join("book").join("first").join("index.html");
    let expected_strings = vec![
        r#"href="../first/index.html""#,
        r#"href="../second/index.html""#,
        "First README",
    ];
    assert_contains_strings(&first_index, &expected_strings);
    assert_doesnt_contain_strings(&first_index, &vec!["README.html"]);

    let second_index = temp.path().join("book").join("second").join("index.html");
    let unexpected_strings = vec!["Second README"];
    assert_doesnt_contain_strings(&second_index, &unexpected_strings);
}

#[test]
fn theme_dir_overrides_work_correctly() {
    let book_dir = dummy_book::new_copy_of_example_book().unwrap();
    let book_dir = book_dir.path();
    let theme_dir = book_dir.join("theme");

    let mut index = ::mdbook::theme::INDEX.to_vec();
    index.extend_from_slice(b"\n<!-- This is a modified index.hbs! -->");

    write_file(&theme_dir, "index.hbs", &index).unwrap();

    let md = MDBook::load(book_dir).unwrap();
    md.build().unwrap();

    let built_index = book_dir.join("book").join("index.html");
    dummy_book::assert_contains_strings(built_index, &["This is a modified index.hbs!"]);
}

#[cfg(feature = "search")]
mod search {
    extern crate serde_json;
    use dummy_book::DummyBook;
    use mdbook::utils::fs::file_to_string;
    use mdbook::MDBook;
    use std::fs::File;
    use std::path::Path;

    fn read_book_index(root: &Path) -> serde_json::Value {
        let index = root.join("book/searchindex.js");
        let index = file_to_string(index).unwrap();
        let index = index.trim_left_matches("window.search = ");
        let index = index.trim_right_matches(";");
        serde_json::from_str(&index).unwrap()
    }

    #[test]
    fn book_creates_reasonable_search_index() {
        let temp = DummyBook::new().build().unwrap();
        let md = MDBook::load(temp.path()).unwrap();
        md.build().unwrap();

        let index = read_book_index(temp.path());

        let doc_urls = index["doc_urls"].as_array().unwrap();
        let get_doc_ref =
            |url: &str| -> String { doc_urls.iter().position(|s| s == url).unwrap().to_string() };

        let first_chapter = get_doc_ref("first/index.html#first-chapter");
        let introduction = get_doc_ref("intro.html#introduction");
        let some_section = get_doc_ref("first/index.html#some-section");
        let summary = get_doc_ref("first/includes.html#summary");
        let conclusion = get_doc_ref("conclusion.html#conclusion");

        let bodyidx = &index["index"]["index"]["body"]["root"];
        let textidx = &bodyidx["t"]["e"]["x"]["t"];
        assert_eq!(textidx["df"], 2);
        assert_eq!(textidx["docs"][&first_chapter]["tf"], 1.0);
        assert_eq!(textidx["docs"][&introduction]["tf"], 1.0);

        let docs = &index["index"]["documentStore"]["docs"];
        assert_eq!(docs[&first_chapter]["body"], "more text.");
        assert_eq!(docs[&some_section]["body"], "");
        assert_eq!(
            docs[&summary]["body"],
            "Dummy Book Introduction First Chapter Nested Chapter Includes Recursive Second Chapter Nested Chapter Conclusion"
        );
        assert_eq!(docs[&summary]["breadcrumbs"], "First Chapter » Summary");
        assert_eq!(docs[&conclusion]["body"], "I put &lt;HTML&gt; in here!");
    }

    // Setting this to `true` may cause issues with `cargo watch`,
    // since it may not finish writing the fixture before the tests
    // are run again.
    const GENERATE_FIXTURE: bool = false;

    fn get_fixture() -> serde_json::Value {
        if GENERATE_FIXTURE {
            let temp = DummyBook::new().build().unwrap();
            let md = MDBook::load(temp.path()).unwrap();
            md.build().unwrap();

            let src = read_book_index(temp.path());

            let dest = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/searchindex_fixture.json");
            let dest = File::create(&dest).unwrap();
            serde_json::to_writer_pretty(dest, &src).unwrap();

            src
        } else {
            let json = include_str!("searchindex_fixture.json");
            serde_json::from_str(json).expect("Unable to deserialize the fixture")
        }
    }

    // So you've broken the test. If you changed dummy_book, it's probably
    // safe to regenerate the fixture. If you haven't then make sure that the
    // search index still works. Run `cargo run -- serve tests/dummy_book`
    // and try some searches. Are you getting results? Do the teasers look OK?
    // Are there new errors in the JS console?
    //
    // If you're pretty sure you haven't broken anything, change `GENERATE_FIXTURE`
    // above to `true`, and run `cargo test` to generate a new fixture. Then
    // **change it back to `false`**. Include the changed `searchindex_fixture.json` in your commit.
    #[test]
    fn search_index_hasnt_changed_accidentally() {
        let temp = DummyBook::new().build().unwrap();
        let md = MDBook::load(temp.path()).unwrap();
        md.build().unwrap();

        let book_index = read_book_index(temp.path());

        let fixture_index = get_fixture();

        // Uncomment this if you're okay with pretty-printing 32KB of JSON
        //assert_eq!(fixture_index, book_index);

        if book_index != fixture_index {
            panic!("The search index has changed from the fixture");
        }
    }
}
