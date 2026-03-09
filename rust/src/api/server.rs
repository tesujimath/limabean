use beancount_parser_lima::{BeancountParser, BeancountSources, ParseError, ParseSuccess};
use std::{
    io::{BufRead, BufReader, Read, Write, stdin, stdout},
    path::Path,
};

pub fn serve(path: &Path) {
    let sources = BeancountSources::try_from(path).unwrap_or_else(|e| {
        // TODO return error as JSON-RPC message, wait for status query
        eprintln!("can't open {}: {}", path.to_string_lossy(), &e);
        std::process::exit(1);
    });
    let parser = BeancountParser::new(&sources);

    let server = Server::new(&sources, &parser);

    server.serve(&stdin(), &stdout());
}

struct Server<'a> {
    sources: &'a BeancountSources,
    parser: &'a BeancountParser<'a>,
    parsed: Result<ParseSuccess<'a>, ParseError>,
}

impl<'a> Server<'a> {
    fn new(sources: &'a BeancountSources, parser: &'a BeancountParser<'a>) -> Self {
        let parsed = parser.parse();

        Self {
            sources,
            parser,
            parsed,
        }
    }

    fn serve<R, W>(&self, r: R, w: W)
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
                        self.dispatch(&buf, w);
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

    fn dispatch<W>(&self, request: &str, w: W)
    where
        W: Write + Copy,
    {
        eprintln!("dispatching {} as get directives", request);
        self.parser_directives_get(w);
    }

    fn parser_directives_get<W>(&self, w: W)
    where
        W: Write + Copy,
    {
        if let Ok(ParseSuccess { directives, .. }) = &self.parsed {
            // directives.iter().map(|d| d.into())
        } else {
            eprintln!("parser error, TODO return as no directives",);
        }
    }
}
