use beancount_parser_lima::{BeancountParser, BeancountSources, ParseError, ParseSuccess};
use std::{
    borrow::Cow,
    io::{self, BufRead, BufReader, Read, Write, stdin, stdout},
    path::Path,
};

use crate::api::{booking, types::raw};

use super::{
    json_rpc::*,
    types::{Report, raw::*},
};

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
        use RequestMethod as Method;

        match serde_json::from_str::<Request>(request) {
            Err(e) => write_error(None, ERROR_PARSE, Cow::Owned(e.to_string()), w).unwrap(),

            Ok(Request {
                id,
                jsonrpc,
                method,
            }) => {
                if jsonrpc != JSONRPC_VERSION {
                    write_error(
                        id,
                        ERROR_INVALID_REQUEST,
                        Cow::Owned(format!("JSON-RPC protocol must be 2.0, found {}", jsonrpc)),
                        w,
                    )
                    .unwrap()
                } else {
                    match (&self.0, method) {
                        (Ok(healthy), Method::Status) => healthy.status(id, w).unwrap(),

                        (Ok(healthy), Method::ParserDirectives) => {
                            healthy.parser_directives(id, w).unwrap()
                        }

                        (Ok(healthy), Method::ParserFormatReport(Params { params })) => {
                            healthy.parser_format_report(id, &params, w).unwrap()
                        }

                        (Ok(healthy), Method::Book(optional)) => {
                            healthy.book(id, optional.params.as_ref(), w).unwrap()
                        }

                        (Err(unhealthy), _) => write_error(
                            id,
                            ERROR_BEANFILE_IO_ERROR,
                            Cow::Borrowed(unhealthy.as_str()),
                            w,
                        )
                        .unwrap(),
                    }
                }
            }
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
    fn parser_directives<W>(&self, id: Option<Id>, w: W) -> io::Result<()>
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

    fn parser_format_report<W>(
        &self,
        id: Option<Id>,
        reports: &[Report<'a>],
        w: W,
    ) -> io::Result<()>
    where
        W: Write + Copy,
    {
        let mut buf = Vec::new();
        let mut sep = false;
        for report in reports {
            if sep {
                buf.write_all(b"\n")?;
            }

            self.sources.write_report(
                &mut buf,
                report.kind.into(),
                report.message,
                report.label,
                &((&report.span).into()),
            )?;

            sep = true;
        }

        let response = ResultResponse::new(id, ResultData::Report(String::from_utf8_lossy(&buf)));
        write_response(&response, w)
    }

    fn book<W>(&self, id: Option<Id>, directives: Option<&Vec<Directive>>, w: W) -> io::Result<()>
    where
        W: Write + Copy,
    {
        if let Some(_directives) = directives {
            // TODO
            write_error(
                id,
                ERROR_INTERNAL,
                Cow::Borrowed("Booking directives other than as-parsed is not yet supported"),
                w,
            )
        } else if let Ok(ParseSuccess {
            directives,
            options,
            ..
        }) = &self.parsed
        {
            match booking::book(directives, options) {
                Ok(booking::LoadSuccess {
                    directives,
                    warnings,
                }) => {
                    // TODO warnings
                    let response = ResultResponse::new(id, ResultData::Booked(directives));
                    write_response(&response, w)
                }

                Err(_) => todo!(),
            }
        } else {
            // TODO format parse errors and return
            write_error(
                id,
                ERROR_PARSE,
                Cow::Borrowed("parse errors, cannot book"),
                w,
            )
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
