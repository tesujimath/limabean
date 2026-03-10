use beancount_parser_lima::{BeancountParser, BeancountSources, ParseError, ParseSuccess};
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
        Err(e) => Server(Err(format!(
            "Can't read {}: {}",
            path.to_string_lossy(),
            &e
        )))
        .serve(&stdin(), &stdout()),
    };
}

struct Server<'a>(Result<HealthyServer<'a>, String>);

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

        tracing::debug!("starting");

        loop {
            buf.clear();

            match reader.read_line(&mut buf) {
                Ok(n) => {
                    if n > 0 {
                        let json = buf.trim();
                        tracing::debug!("<- {}", json);
                        self.dispatch(json, w);
                    } else {
                        // that's all folks
                        tracing::debug!("<- EOF");
                        tracing::debug!("exit");
                        std::process::exit(0);
                    }
                }

                Err(e) => {
                    tracing::error!("<- {}", &e);
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
                    id, method: Status, ..
                }),
            ) => healthy.status(id, w).unwrap(),

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
                Cow::Borrowed(unhealthy.as_str()),
                w,
            )
            .unwrap(),
            (_, Err(e)) => write_error(None, ERROR_PARSE, Cow::Owned(e.to_string()), w).unwrap(),
        }
    }
}

impl<'a> HealthyServer<'a> {
    fn status<W>(&self, id: Option<Id>, w: W) -> io::Result<()>
    where
        W: Write + Copy,
    {
        let response = ResultResponse::new(id, ResultData::Ok);

        write_response(&response, w)
    }
}

impl<'a> HealthyServer<'a> {
    fn parser_directives_get<W>(&self, id: Option<Id>, w: W) -> io::Result<()>
    where
        W: Write + Copy,
    {
        if let Ok(ParseSuccess { directives, .. }) = &self.parsed {
            let response = ResultResponse::new(
                id,
                ResultData::RawDirectives(
                    directives
                        .iter()
                        .map(Into::<Directive>::into)
                        .collect::<Vec<_>>(),
                ),
            );

            write_response(&response, w)
        } else {
            tracing::error!("parse error, no directives to return");
            Ok(())
        }
    }
}

fn write_response<W>(response: &ResultResponse, mut w: W) -> io::Result<()>
where
    W: Write + Copy,
{
    match serde_json::to_string(&response) {
        Ok(json) => {
            tracing::debug!("-> {}", &json);
            writeln!(w, "{}", &json)
        }
        Err(e) => {
            tracing::error!("{}", &e);
            if let serde_json::error::Category::Io = e.classify() {
                Err(io::Error::new(
                    e.io_error_kind().unwrap(),
                    "while writing JSON",
                ))
            } else {
                write_error(response.id, ERROR_INTERNAL, Cow::Owned(e.to_string()), w)
            }
        }
    }
}

fn write_error<'a, 'b, W>(
    id: Option<Id<'a>>,
    code: ErrorCode,
    message: Cow<'b, str>,
    mut w: W,
) -> io::Result<()>
where
    W: Write + Copy,
{
    let response = ErrorResponse::new(id, code, message);
    match serde_json::to_string(&response) {
        Ok(json) => {
            tracing::debug!("-> {}", &json);
            writeln!(w, "{}", &json)
        }
        Err(e) => {
            tracing::error!("failed writing error {}", &e);

            panic!("can't even write error {}", &e)
        }
    }
}
