extern crate iron;
extern crate staticfile;
extern crate ws;

use std;
use std::path::Path;
use self::iron::{Iron, AfterMiddleware, IronResult, IronError, Request, Response, status, Set, Chain};
use clap::{ArgMatches, SubCommand, App};
use mdbook::MDBook;
use mdbook::errors::Result;
use {get_book_dir, open};
#[cfg(feature = "watch")]
use watch;

struct ErrorRecover;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about("Serve the book at http://localhost:3000. Rebuild and reload on change.")
        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'")
        .arg_from_usage("-d, --dest-dir=[dest-dir] 'The output directory for your book{n}(Defaults to ./book when omitted)'")
        .arg_from_usage("--curly-quotes 'Convert straight quotes to curly quotes, except for those that occur in code blocks and code spans'")
        .arg_from_usage("-p, --port=[port] 'Use another port{n}(Defaults to 3000)'")
        .arg_from_usage("-w, --websocket-port=[ws-port] 'Use another port for the websocket connection (livereload){n}(Defaults to 3001)'")
        .arg_from_usage("-i, --interface=[interface] 'Interface to listen on{n}(Defaults to localhost)'")
        .arg_from_usage("-a, --address=[address] 'Address that the browser can reach the websocket server from{n}(Defaults to the interface address)'")
        .arg_from_usage("-o, --open 'Open the book server in a web browser'")
}

// Watch command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    const RELOAD_COMMAND: &'static str = "reload";

    let book_dir = get_book_dir(args);
    let book = MDBook::new(&book_dir).read_config()?;

    let mut book = match args.value_of("dest-dir") {
        Some(dest_dir) => book.with_destination(Path::new(dest_dir)),
        None => book,
    };

    if args.is_present("curly-quotes") {
        book = book.with_curly_quotes(true);
    }

    let port = args.value_of("port").unwrap_or("3000");
    let ws_port = args.value_of("websocket-port").unwrap_or("3001");
    let interface = args.value_of("interface").unwrap_or("localhost");
    let public_address = args.value_of("address").unwrap_or(interface);
    let open_browser = args.is_present("open");

    let address = format!("{}:{}", interface, port);
    let ws_address = format!("{}:{}", interface, ws_port);

    book.set_livereload(format!(r#"
    <script type="text/javascript">
        var socket = new WebSocket("ws://{}:{}");
        socket.onmessage = function (event) {{
            if (event.data === "{}") {{
                socket.close();
                location.reload(true); // force reload from server (not from cache)
            }}
        }};

        window.onbeforeunload = function() {{
            socket.close();
        }}
    </script>
"#,
                                public_address,
                                ws_port,
                                RELOAD_COMMAND));

    book.build()?;

    let mut chain = Chain::new(staticfile::Static::new(book.get_destination()));
    chain.link_after(ErrorRecover);
    let _iron = Iron::new(chain).http(&*address).unwrap();

    let ws_server = ws::WebSocket::new(|_| |_| Ok(())).unwrap();

    let broadcaster = ws_server.broadcaster();

    std::thread::spawn(move || { ws_server.listen(&*ws_address).unwrap(); });

    let serving_url = format!("http://{}", address);
    println!("\nServing on: {}", serving_url);

    if open_browser {
        open(serving_url);
    }

    #[cfg(feature = "watch")]
    watch::trigger_on_change(&mut book, move |path, book| {
        println!("File changed: {:?}\nBuilding book...\n", path);
        match book.build() {
            Err(e) => println!("Error while building: {:?}", e),
            _ => broadcaster.send(RELOAD_COMMAND).unwrap(),
        }
        println!("");
    });

    Ok(())
}

impl AfterMiddleware for ErrorRecover {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        match err.response.status {
            // each error will result in 404 response
            Some(_) => Ok(err.response.set(status::NotFound)),
            _ => Err(err),
        }
    }
}
