extern crate iron;
extern crate staticfile;
extern crate ws;

use std;
use self::iron::{status, AfterMiddleware, Chain, Iron, IronError, IronResult, Request, Response,
                 Set};
use clap::{App, ArgMatches, SubCommand};
use mdbook::MDBook;
use mdbook::utils;
use mdbook::errors::*;
use {get_book_dir, open};
#[cfg(feature = "watch")]
use watch;

struct ErrorRecover;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about("Serve the book at http://localhost:3000. Rebuild and reload on change.")
        .arg_from_usage(
            "[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'",
        )
        .arg_from_usage("-p, --port=[port] 'Use another port{n}(Defaults to 3000)'")
        .arg_from_usage(
            "-w, --websocket-port=[ws-port] 'Use another port for the websocket connection \
             (livereload){n}(Defaults to 3001)'",
        )
        .arg_from_usage(
            "-i, --interface=[interface] 'Interface to listen on{n}(Defaults to localhost)'",
        )
        .arg_from_usage(
            "-a, --address=[address] 'Address that the browser can reach the websocket server \
             from{n}(Defaults to the interface address)'",
        )
        .arg_from_usage("-o, --open 'Open the book server in a web browser'")
}

// Watch command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    let port = args.value_of("port").unwrap_or("3000");
    let ws_port = args.value_of("websocket-port").unwrap_or("3001");
    let interface = args.value_of("interface").unwrap_or("localhost");
    let public_address = args.value_of("address").unwrap_or(interface);
    let open_browser = args.is_present("open");

    let address = format!("{}:{}", interface, port);
    let ws_address = format!("{}:{}", interface, ws_port);

    let livereload_url = format!("ws://{}:{}", public_address, ws_port);
    book.config
        .set("output.html.livereload-url", &livereload_url)?;

    book.build()?;

    let mut chain = Chain::new(staticfile::Static::new(book.build_dir_for("html")));
    chain.link_after(ErrorRecover);
    let _iron = Iron::new(chain)
        .http(&*address)
        .chain_err(|| "Unable to launch the server")?;

    let ws_server =
        ws::WebSocket::new(|_| |_| Ok(())).chain_err(|| "Unable to start the websocket")?;

    let broadcaster = ws_server.broadcaster();

    std::thread::spawn(move || {
        ws_server.listen(&*ws_address).unwrap();
    });

    let serving_url = format!("http://{}", address);
    info!("Serving on: {}", serving_url);

    if open_browser {
        open(serving_url);
    }

    #[cfg(feature = "watch")]
    watch::trigger_on_change(&mut book, move |path, book_dir| {
        info!("File changed: {:?}", path);
        info!("Building book...");

        // FIXME: This area is really ugly because we need to re-set livereload :(

        let livereload_url = livereload_url.clone();

        let result = MDBook::load(&book_dir)
            .and_then(move |mut b| {
                b.config.set("output.html.livereload-url", &livereload_url)?;
                Ok(b)
            })
            .and_then(|b| b.build());

        if let Err(e) = result {
            error!("Unable to load the book");
            utils::log_backtrace(&e);
        } else {
            let _ = broadcaster.send("reload");
        }
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
