use beancount_parser_lima::{BeancountParser, BeancountSources, ParseError, ParseSuccess};
use std::{
    io::{self, BufRead, BufReader, Read, Write, stdin, stdout},
    path::Path,
};

use super::{json_rpc::*, types::*};

pub fn serve(path: &Path) -> io::Result<()> {
    let sources = BeancountSources::try_from(path).unwrap_or_else(|e| {
        // TODO return error as JSON-RPC message, wait for status query
        eprintln!("can't open {}: {}", path.to_string_lossy(), &e);
        std::process::exit(1);
    });
    let parser = BeancountParser::new(&sources);

    let server = Server::new(&sources, &parser);

    server.serve(&stdin(), &stdout())
}

struct Server<'a> {
    sources: &'a BeancountSources,
    _parser: &'a BeancountParser<'a>,
    parsed: Result<ParseSuccess<'a>, ParseError>,
}

impl<'a> Server<'a> {
    fn new(sources: &'a BeancountSources, parser: &'a BeancountParser<'a>) -> Self {
        let parsed = parser.parse();

        Self {
            sources,
            _parser: parser,
            parsed,
        }
    }

    fn serve<R, W>(&self, r: R, w: W) -> io::Result<()>
    where
        R: Read + Copy,
        W: Write + Copy,
    {
        let mut buf = String::new();
        let mut reader = BufReader::new(r);

        loop {
            buf.clear();

            match reader.read_line(&mut buf) {
                Ok(n) => {
                    if n > 0 {
                        self.dispatch(&buf, w)?;
                    } else {
                        // that's all folks
                        eprintln!("EOF on input, exiting");
                        std::process::exit(0);
                    }
                }

                Err(e) => {
                    eprintln!("Error {} on input, exiting", &e);
                    std::process::exit(1);
                }
            }
        }
    }

    fn dispatch<W>(&self, request: &str, w: W) -> io::Result<()>
    where
        W: Write + Copy,
    {
        use RequestMethod::*;

        match serde_json::from_str::<Request>(request) {
            Ok(Request {
                method: ParserDirectivesGet(_),
                ..
            }) => self.parser_directives_get(w),
            Ok(Request {
                method: DirectivesPut(_),
                ..
            }) => todo!(),
            Err(e) => {
                eprintln!("JSON decode error {}", &e);
                // TODO
                Ok(())
            }
        }
    }

    fn parser_directives_get<W>(&self, mut w: W) -> io::Result<()>
    where
        W: Write + Copy,
    {
        if let Ok(ParseSuccess { directives, .. }) = &self.parsed {
            let api_directives = directives
                .iter()
                .map(Into::<Directive>::into)
                .collect::<Vec<_>>();

            if let Err(e) = serde_json::to_writer(w, &api_directives) {
                eprint!("serde_json error {}", &e);
            }
            w.write_all(b"\n")
        } else {
            eprintln!("parser error, TODO return as no directives",);
            Ok(())
        }
    }
}
