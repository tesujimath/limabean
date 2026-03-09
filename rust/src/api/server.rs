use beancount_parser_lima::{BeancountParser, BeancountSources, ParseError, ParseSuccess};
use serde::Serialize;
use std::{
    borrow::Cow,
    io::{self, BufRead, BufReader, Read, Write, stdin, stdout},
    path::Path,
};

use super::{json_rpc::*, types::*};

pub fn serve(path: &Path) {
    match BeancountSources::try_from(path) {
        Ok(sources) => {
            let parser = BeancountParser::new(&sources);

            Server(Ok(HealthyServer::new(&sources, &parser))).serve(&stdin(), &stdout());
        }
        Err(e) => Server(Err(e)).serve(&stdin(), &stdout()),
    };
}

struct Server<'a>(io::Result<HealthyServer<'a>>);

struct HealthyServer<'a> {
    sources: &'a BeancountSources,
    _parser: &'a BeancountParser<'a>,
    parsed: Result<ParseSuccess<'a>, ParseError>,
}

impl<'a> HealthyServer<'a> {
    fn new(sources: &'a BeancountSources, parser: &'a BeancountParser<'a>) -> Self {
        let parsed = parser.parse();

        Self {
            sources,
            _parser: parser,
            parsed,
        }
    }
}

impl<'a> Server<'a> {
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
        use RequestMethod::*;

        match (&self.0, serde_json::from_str::<Request>(request)) {
            (
                Ok(healthy),
                Ok(Request {
                    id,
                    method: ParserDirectivesGet(_),
                    ..
                }),
            ) => healthy.parser_directives_get(id, w).unwrap(),

            (
                Ok(healthy),
                Ok(Request {
                    id,
                    method: DirectivesPut(_),
                    ..
                }),
            ) => todo!(),
            (Err(unhealthy), Ok(Request { id, .. })) => write_error(
                id,
                ERROR_BEANFILE_IO_ERROR,
                Cow::Owned(unhealthy.to_string()),
                w,
            )
            .unwrap(),
            (_, Err(e)) => {
                write_error("unknown", ERROR_PARSE, Cow::Owned(e.to_string()), w).unwrap()
            }
        }
    }
}

impl<'a> HealthyServer<'a> {
    fn parser_directives_get<W>(&self, id: &str, mut w: W) -> io::Result<()>
    where
        W: Write + Copy,
    {
        if let Ok(ParseSuccess { directives, .. }) = &self.parsed {
            let response = ResultResponse::new(
                "id",
                ResultData::ParserDirectives(
                    directives
                        .iter()
                        .map(Into::<Directive>::into)
                        .collect::<Vec<_>>(),
                ),
            );

            if let Err(e) = serde_json::to_writer(w, &response) {
                write_error(id, ERROR_INTERNAL, Cow::Owned(e.to_string()), w)?;
            }
            w.write_all(b"\n")
        } else {
            eprintln!("parser error, TODO return as no directives",);
            Ok(())
        }
    }
}

fn write_response<W>(response: &ResultResponse, mut w: W) -> io::Result<()>
where
    W: Write + Copy,
{
    if let Err(e) = serde_json::to_writer(w, &response) {
        if let serde_json::error::Category::Io = e.classify() {
            Err(io::Error::new(
                e.io_error_kind().unwrap(),
                "while writing JSON",
            ))
        } else {
            write_error(response.id, ERROR_INTERNAL, Cow::Owned(e.to_string()), w)
        }
    } else {
        w.write_all(b"\n")
    }
}

fn write_error<'a, W>(id: &str, code: ErrorCode, message: Cow<'a, str>, mut w: W) -> io::Result<()>
where
    W: Write + Copy,
{
    let response = ErrorResponse::new(id, code, message);
    match serde_json::to_string(&response) {
        Ok(json) => writeln!(w, "{}", &json),
        Err(e) => panic!("can't even write error {}", &e),
    }
}
