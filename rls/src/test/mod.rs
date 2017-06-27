// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Utilities and infrastructure for testing. Tests in this module test the
// testing infrastructure *not* the RLS.

mod types;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use env_logger;

use analysis;
use build;
use server as ls_server;
use vfs;

use self::types::src;

use hyper::Url;
use serde_json;
use std::path::{Path, PathBuf};

const TEST_TIMEOUT_IN_SEC: u64 = 10;

#[test]
fn test_goto_def() {
    let (mut cache, _tc) = init_env("goto_def");

    let source_file_path = Path::new("src").join("main.rs");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));
    let url = Url::from_file_path(cache.abs_path(&source_file_path)).expect("couldn't convert file path to URL");
    let text_doc = format!("{{\"uri\":{}}}", serde_json::to_string(&url.as_str().to_owned())
                                                 .expect("couldn't convert path to JSON"));
    let messages = vec![Message::new("initialize", vec![("processId", "0".to_owned()),
                                                        ("capabilities", "null".to_owned()),
                                                        ("rootPath", root_path)]),
                        Message::new("textDocument/definition",
                                     vec![("textDocument", text_doc),
                                          ("position", cache.mk_ls_position(src(&source_file_path, 22, "world")))])];
    let (server, results) = mock_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    // TODO structural checking of result, rather than looking for a string - src(&source_file_path, 12, "world")
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains("\"start\":{\"line\":20,\"character\":8}")]);
}

#[test]
fn test_hover() {
    let (mut cache, _tc) = init_env("hover");

    let source_file_path = Path::new("src").join("main.rs");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));

    let url = Url::from_file_path(cache.abs_path(&source_file_path)).expect("couldn't convert file path to URL");
    let text_doc = format!("{{\"uri\":{}}}", serde_json::to_string(&url.as_str().to_owned())
                                                 .expect("couldn't convert path to JSON"));
    let messages = vec![Message::new("initialize", vec![("processId", "0".to_owned()),
                                                        ("capabilities", "null".to_owned()),
                                                        ("rootPath", root_path)]),
                        Message::new("textDocument/hover",
                                     vec![("textDocument", text_doc),
                                          ("position", cache.mk_ls_position(src(&source_file_path, 22, "world")))])];
    let (server, results) = mock_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains("[{\"language\":\"rust\",\"value\":\"&str\"}]")]);
}

#[test]
fn test_find_all_refs() {
    let (mut cache, _tc) = init_env("find_all_refs");

    let source_file_path = Path::new("src").join("main.rs");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));
    let url = Url::from_file_path(cache.abs_path(&source_file_path)).expect("couldn't convert file path to URL");
    let text_doc = format!("{{\"uri\":{}}}", serde_json::to_string(&url.as_str().to_owned())
                                                 .expect("couldn't convert path to JSON"));
    let messages = vec![format!(r#"{{
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 0,
        "params": {{
            "processId": "0",
            "capabilities": null,
            "rootPath": {}
        }}
    }}"#, root_path), format!(r#"{{
        "jsonrpc": "2.0",
        "method": "textDocument/references",
        "id": 42,
        "params": {{
            "textDocument": {},
            "position": {},
            "context": {{
                "includeDeclaration": true
            }}
        }}
    }}"#, text_doc, cache.mk_ls_position(src(&source_file_path, 10, "Bar")))];

    let (server, results) = mock_raw_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(0)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains(r#"{"start":{"line":9,"character":7},"end":{"line":9,"character":10}}"#)
                                                                     .expect_contains(r#"{"start":{"line":15,"character":14},"end":{"line":15,"character":17}}"#)
                                                                     .expect_contains(r#"{"start":{"line":23,"character":15},"end":{"line":23,"character":18}}"#)]);
}

#[test]
fn test_find_all_refs_no_cfg_test() {
    let (mut cache, _tc) = init_env("find_all_refs_no_cfg_test");

    let source_file_path = Path::new("src").join("main.rs");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));
    let url = Url::from_file_path(cache.abs_path(&source_file_path)).expect("couldn't convert file path to URL");
    let text_doc = format!("{{\"uri\":{}}}", serde_json::to_string(&url.as_str().to_owned())
                                                 .expect("couldn't convert path to JSON"));
    let messages = vec![format!(r#"{{
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 0,
        "params": {{
            "processId": "0",
            "capabilities": null,
            "rootPath": {}
        }}
    }}"#, root_path), format!(r#"{{
        "jsonrpc": "2.0",
        "method": "textDocument/references",
        "id": 42,
        "params": {{
            "textDocument": {},
            "position": {},
            "context": {{
                "includeDeclaration": true
            }}
        }}
    }}"#, text_doc, cache.mk_ls_position(src(&source_file_path, 10, "Bar")))];

    let (server, results) = mock_raw_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(0)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains(r#"{"start":{"line":9,"character":7},"end":{"line":9,"character":10}}"#)
                                                                     .expect_contains(r#"{"start":{"line":23,"character":15},"end":{"line":23,"character":18}}"#)]);
}

#[test]
fn test_borrow_error() {
    let (cache, _tc) = init_env("borrow_error");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));
    let messages = vec![format!(r#"{{
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 0,
        "params": {{
            "processId": "0",
            "capabilities": null,
            "rootPath": {}
        }}
    }}"#, root_path)];

    let (server, results) = mock_raw_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(0)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("\"secondaryRanges\":[{\"start\":{\"line\":2,\"character\":17},\"end\":{\"line\":2,\"character\":18},\"label\":\"first mutable borrow occurs here\"}"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);
}

#[test]
fn test_highlight() {
    let (mut cache, _tc) = init_env("highlight");

    let source_file_path = Path::new("src").join("main.rs");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));
    let url = Url::from_file_path(cache.abs_path(&source_file_path)).expect("couldn't convert file path to URL");
    let text_doc = format!("{{\"uri\":{}}}", serde_json::to_string(&url.as_str().to_owned())
                                                 .expect("couldn't convert path to JSON"));
    let messages = vec![format!(r#"{{
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 0,
        "params": {{
            "processId": "0",
            "capabilities": null,
            "rootPath": {}
        }}
    }}"#, root_path), format!(r#"{{
        "jsonrpc": "2.0",
        "method": "textDocument/documentHighlight",
        "id": 42,
        "params": {{
            "textDocument": {},
            "position": {}
        }}
    }}"#, text_doc, cache.mk_ls_position(src(&source_file_path, 22, "world")))];

    let (server, results) = mock_raw_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(0)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains(r#"{"start":{"line":20,"character":8},"end":{"line":20,"character":13}}"#)
                                                                     .expect_contains(r#"{"start":{"line":21,"character":27},"end":{"line":21,"character":32}}"#),]);
}

#[test]
fn test_rename() {
    let (mut cache, _tc) = init_env("rename");

    let source_file_path = Path::new("src").join("main.rs");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));
    let url = Url::from_file_path(cache.abs_path(&source_file_path)).expect("couldn't convert file path to URL");
    let text_doc = format!("{{\"uri\":{}}}", serde_json::to_string(&url.as_str().to_owned())
                                                 .expect("couldn't convert path to JSON"));
    let messages = vec![format!(r#"{{
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 0,
        "params": {{
            "processId": "0",
            "capabilities": null,
            "rootPath": {}
        }}
    }}"#, root_path), format!(r#"{{
        "jsonrpc": "2.0",
        "method": "textDocument/rename",
        "id": 42,
        "params": {{
            "textDocument": {},
            "position": {},
            "newName": "foo"
        }}
    }}"#, text_doc, cache.mk_ls_position(src(&source_file_path, 22, "world")))];

    let (server, results) = mock_raw_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(0)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains(r#"{"start":{"line":20,"character":8},"end":{"line":20,"character":13}}"#)
                                                                     .expect_contains(r#"{"start":{"line":21,"character":27},"end":{"line":21,"character":32}}"#)
                                                                     .expect_contains(r#"{"changes""#),]);
}

#[test]
fn test_completion() {
    let (mut cache, _tc) = init_env("completion");

    let source_file_path = Path::new("src").join("main.rs");

    let root_path = format!("{}", serde_json::to_string(&cache.abs_path(Path::new(".")))
                                      .expect("couldn't convert path to JSON"));
    let url = Url::from_file_path(cache.abs_path(&source_file_path)).expect("couldn't convert file path to URL");
    let text_doc = format!("{{\"uri\":{}}}", serde_json::to_string(&url.as_str().to_owned())
                                                 .expect("couldn't convert path to JSON"));
    let messages = vec![Message::new("initialize", vec![("processId", "0".to_owned()),
                                                        ("capabilities", "null".to_owned()),
                                                        ("rootPath", root_path)]),
                        Message::new("textDocument/completion",
                                     vec![("textDocument", text_doc.to_owned()),
                                          ("position", cache.mk_ls_position(src(&source_file_path, 22, "rld")))]),
                        Message::new("textDocument/completion",
                                     vec![("textDocument", text_doc.to_owned()),
                                          ("position", cache.mk_ls_position(src(&source_file_path, 25, "x)")))])];
    let (server, results) = mock_server(messages);
    // Initialise and build.
    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains("capabilities"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsBegin"),
                                       ExpectedMessage::new(None).expect_contains("diagnosticsEnd")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains("[{\"label\":\"world\",\"kind\":6,\"detail\":\"let world = \\\"world\\\";\"}]")]);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Continue);
    expect_messages(results.clone(), &[ExpectedMessage::new(Some(42)).expect_contains("[{\"label\":\"x\",\"kind\":5,\"detail\":\"u64\"}]")]);
}

#[test]
fn test_parse_error_on_malformed_input() {
    let _ = env_logger::init();
    struct NoneMsgReader;

    impl ls_server::MessageReader for NoneMsgReader {
        fn read_message(&self) -> Option<String> { None }
    }

    let analysis = Arc::new(analysis::AnalysisHost::new(analysis::Target::Debug));
    let vfs = Arc::new(vfs::Vfs::new());
    let build_queue = Arc::new(build::BuildQueue::new(vfs.clone()));
    let reader = Box::new(NoneMsgReader);
    let output = Box::new(RecordOutput::new());
    let results = output.output.clone();
    let server = ls_server::LsService::new(analysis, vfs, build_queue, reader, output);

    assert_eq!(ls_server::LsService::handle_message(server.clone()),
               ls_server::ServerStateChange::Break);

    let error = results.lock().unwrap()
        .pop().expect("no error response");
    assert!(error.contains(r#""code": -32700"#))
}

// Initialise and run the internals of an LS protocol RLS server.
fn mock_server(messages: Vec<Message>) -> (Arc<ls_server::LsService>, LsResultList)
{
    let analysis = Arc::new(analysis::AnalysisHost::new(analysis::Target::Debug));
    let vfs = Arc::new(vfs::Vfs::new());
    let build_queue = Arc::new(build::BuildQueue::new(vfs.clone()));
    let reader = Box::new(MockMsgReader::new(messages));
    let output = Box::new(RecordOutput::new());
    let results = output.output.clone();
    (ls_server::LsService::new(analysis, vfs, build_queue, reader, output), results)
}

// Initialise and run the internals of an LS protocol RLS server.
fn mock_raw_server(messages: Vec<String>) -> (Arc<ls_server::LsService>, LsResultList)
{
    let analysis = Arc::new(analysis::AnalysisHost::new(analysis::Target::Debug));
    let vfs = Arc::new(vfs::Vfs::new());
    let build_queue = Arc::new(build::BuildQueue::new(vfs.clone()));
    let reader = Box::new(MockRawMsgReader::new(messages));
    let output = Box::new(RecordOutput::new());
    let results = output.output.clone();
    (ls_server::LsService::new(analysis, vfs, build_queue, reader, output), results)
}

struct MockMsgReader {
    messages: Vec<Message>,
    cur: Mutex<usize>,
}

impl MockMsgReader {
    fn new(messages: Vec<Message>) -> MockMsgReader {
        MockMsgReader {
            messages: messages,
            cur: Mutex::new(0),
        }
    }
}

struct MockRawMsgReader {
    messages: Vec<String>,
    cur: Mutex<usize>,
}

impl MockRawMsgReader {
    fn new(messages: Vec<String>) -> MockRawMsgReader {
        MockRawMsgReader {
            messages: messages,
            cur: Mutex::new(0),
        }
    }
}

// TODO should have a structural way of making params, rather than taking Strings
struct Message {
    method: &'static str,
    params: Vec<(&'static str, String)>,
}

impl Message {
    fn new(method: &'static str, params: Vec<(&'static str, String)>) -> Message {
        Message {
            method: method,
            params: params,
        }
    }
}

impl ls_server::MessageReader for MockMsgReader {
    fn read_message(&self) -> Option<String> {
        // Note that we hold this lock until the end of the function, thus meaning
        // that we must finish processing one message before processing the next.
        let mut cur = self.cur.lock().unwrap();
        let index = *cur;
        *cur += 1;

        if index >= self.messages.len() {
            return None;
        }

        let message = &self.messages[index];

        let params = message.params.iter().map(|&(k, ref v)| format!("\"{}\":{}", k, v)).collect::<Vec<String>>().join(",");
        // TODO don't hardcode the id, we should use fresh ids and use them to look up responses
        let result = format!("{{\"method\":\"{}\",\"id\":42,\"params\":{{{}}}}}", message.method, params);
        // println!("read_message: `{}`", result);

        Some(result)
    }
}

impl ls_server::MessageReader for MockRawMsgReader {
    fn read_message(&self) -> Option<String> {
        // Note that we hold this lock until the end of the function, thus meaning
        // that we must finish processing one message before processing the next.
        let mut cur = self.cur.lock().unwrap();
        let index = *cur;
        *cur += 1;

        if index >= self.messages.len() {
            return None;
        }

        let message = &self.messages[index];

        Some(message.clone())
    }
}

type LsResultList = Arc<Mutex<Vec<String>>>;

struct RecordOutput {
    output: LsResultList,
}

impl RecordOutput {
    fn new() -> RecordOutput {
        RecordOutput {
            output: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl ls_server::Output for RecordOutput {
    fn response(&self, output: String) {
        let mut records = self.output.lock().unwrap();
        records.push(output);
    }
}

// Initialise the environment for a test.
fn init_env(project_dir: &str) -> (types::Cache, TestCleanup) {
    let _ = env_logger::init();

    let path = &Path::new("test_data").join(project_dir);
    let tc = TestCleanup { path: path.to_owned() };
    (types::Cache::new(path), tc)
}

#[derive(Clone, Debug)]
struct ExpectedMessage {
    id: Option<u64>,
    contains: Vec<String>,
}

impl ExpectedMessage {
    fn new(id: Option<u64>) -> ExpectedMessage {
        ExpectedMessage {
            id: id,
            contains: vec![],
        }
    }

    fn expect_contains(&mut self, s: &str) -> &mut ExpectedMessage {
        self.contains.push(s.to_owned());
        self
    }
}

fn expect_messages(results: LsResultList, expected: &[&ExpectedMessage]) {
    let start_clock = SystemTime::now();
    let mut results_count = results.lock().unwrap().len();
    while (results_count != expected.len()) && (start_clock.elapsed().unwrap().as_secs() < TEST_TIMEOUT_IN_SEC) {
        thread::sleep(Duration::from_millis(100));
        results_count = results.lock().unwrap().len();
    }

    let mut results = results.lock().unwrap();

    println!("expect_messages: results: {:?},\nexpected: {:?}", *results, expected);
    assert_eq!(results.len(), expected.len());
    for (found, expected) in results.iter().zip(expected.iter()) {
        let values: serde_json::Value = serde_json::from_str(found).unwrap();
        assert!(values.get("jsonrpc").expect("Missing jsonrpc field").as_str().unwrap() == "2.0", "Bad jsonrpc field");
        if let Some(id) = expected.id {
            assert_eq!(values.get("id").expect("Missing id field").as_u64().unwrap(), id, "Unexpected id");
        }
        for c in expected.contains.iter() {
            found.find(c).expect(&format!("Could not find `{}` in `{}`", c, found));
        }
    }

    *results = vec![];
}

struct TestCleanup {
    path: PathBuf
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        use std::fs;

        let target_path = self.path.join("target");
        if fs::metadata(&target_path).is_ok() {
            fs::remove_dir_all(target_path).expect("failed to tidy up");
        }
    }
}
