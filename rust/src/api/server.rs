use beancount_parser_lima::{
    self as parser, BeancountParser, BeancountSources, ParseError, ParseSuccess,
};
use std::{
    borrow::Cow,
    io::{self, BufRead, BufReader, Read, Write, stdin, stdout},
    path::Path,
};

use crate::api::{
    booking::{self, LoadError},
    json_rpc::*,
    types::{
        Report,
        parser_type_conversions::{from_annotated_errors_or_warnings, from_errors_or_warnings},
        raw::*,
    },
};

pub fn serve(path: &Path) {
    match BeancountSources::try_from(path) {
        Ok(sources) => {
            let parser = BeancountParser::new(&sources);

            Server(Ok(HealthyServer::new(&sources, &parser))).serve(&stdin(), &mut stdout());
        }
        Err(e) => Server(Err(format!(
            "Can't read {}: {}",
            path.to_string_lossy(),
            &e
        )))
        .serve(&stdin(), &mut stdout()),
    };
}

struct Server<'a>(Result<HealthyServer<'a>, String>);

struct HealthyServer<'a> {
    sources: &'a BeancountSources,
    _parser: &'a BeancountParser<'a>,
    parsed: Result<Parsed<'a>, Vec<parser::Error>>,
}

struct Parsed<'a> {
    pub directives: Vec<parser::Spanned<parser::Directive<'a>>>,
    pub options: parser::Options<'a>,
    pub plugins: Vec<parser::Plugin<'a>>,
    pub warnings: Vec<parser::Warning>,
}

impl<'a> HealthyServer<'a> {
    fn new(sources: &'a BeancountSources, parser: &'a BeancountParser<'a>) -> Self {
        let parsed = match parser.parse() {
            Ok(ParseSuccess {
                directives,
                options,
                plugins,
                warnings,
            }) => Ok(Parsed {
                directives,
                options,
                plugins,
                warnings,
            }),
            Err(ParseError { errors, .. }) => Err(errors),
        };

        Self {
            sources,
            _parser: parser,
            parsed,
        }
    }
}

impl<'a> Server<'a> {
    fn serve<R, W>(&self, r: R, w: &mut W)
    where
        R: Read,
        W: Write,
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

    fn dispatch<W>(&self, request: &str, w: &mut W)
    where
        W: Write,
    {
        use RequestMethod as Method;

        match serde_json::from_str::<Request>(request) {
            Err(e) => write_other_error(None, ERROR_PARSE, Cow::Owned(e.to_string()), w).unwrap(),

            Ok(Request {
                id,
                jsonrpc,
                method,
            }) => {
                if jsonrpc != JSONRPC_VERSION {
                    write_other_error(
                        id,
                        ERROR_INVALID_REQUEST,
                        Cow::Owned(format!("JSON-RPC protocol must be 2.0, found {}", jsonrpc)),
                        w,
                    )
                    .unwrap()
                } else {
                    match (&self.0, method) {
                        (Ok(healthy), Method::Status) => healthy.status(id, w).unwrap(),

                        (Ok(healthy), Method::ParserPlugins) => {
                            healthy.parser_plugins(id, w).unwrap()
                        }

                        (Ok(healthy), Method::ParserDirectives) => {
                            healthy.parser_directives(id, w).unwrap()
                        }

                        (Ok(healthy), Method::ParserFormatErrors(Params { params })) => healthy
                            .parser_format_report::<parser::ErrorKind, W>(id, &params, w)
                            .unwrap(),

                        (Ok(healthy), Method::ParserFormatWarnings(Params { params })) => healthy
                            .parser_format_report::<parser::WarningKind, W>(id, &params, w)
                            .unwrap(),

                        (Ok(healthy), Method::ParserResolveSpan(Params { params })) => {
                            healthy.parser_resolve_span(id, &params, w).unwrap()
                        }

                        (Ok(healthy), Method::Book(optional)) => {
                            healthy.book(id, optional.params.as_ref(), w).unwrap()
                        }

                        (Err(unhealthy), _) => write_other_error(
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
    fn status<W>(&self, id: Option<Id>, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        let response = ResultResponse::new(id, ResultData::Ok);

        write_response(&response, w)
    }
}

impl<'a> HealthyServer<'a> {
    fn parser_plugins<W>(&self, id: Option<Id>, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match &self.parsed {
            Ok(Parsed { plugins, .. }) => {
                let plugins = plugins
                    .iter()
                    .map(|plugin| plugin.into())
                    .collect::<Vec<_>>();
                let response = ResultResponse::new(id, ResultData::Plugins(&plugins));

                write_response(&response, w)
            }
            Err(errors) => {
                let reports = from_errors_or_warnings(errors);
                write_error_reports(None, reports, w)
            }
        }
    }

    fn parser_directives<W>(&self, id: Option<Id>, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match &self.parsed {
            Ok(Parsed { directives, .. }) => {
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
            }
            Err(errors) => {
                let reports = from_errors_or_warnings(errors);
                write_error_reports(None, reports, w)
            }
        }
    }

    fn parser_format_report<K, W>(
        &self,
        id: Option<Id>,
        reports: &[Report<'a>],
        w: &mut W,
    ) -> io::Result<()>
    where
        K: parser::ErrorOrWarningKind,
        W: Write,
    {
        let mut buf = Vec::new();
        let mut sep = false;

        for report in reports {
            if sep {
                buf.write_all(b"\n")?;
            }

            self.sources
                .write_report::<_, K, Report>(&mut buf, report)?;
            if let Some(annotation) = report.annotation.as_ref() {
                buf.write_fmt(core::format_args!("{}\n", annotation))?;
            }

            sep = true;
        }

        let response = ResultResponse::new(id, ResultData::Report(String::from_utf8_lossy(&buf)));
        write_response(&response, w)
    }

    fn parser_resolve_span<W>(&self, id: Option<Id>, span: &Span, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        let span = span.into();
        let spanned_source = self.sources.resolve_span(&span);
        let response = ResultResponse::new(id, ResultData::ResolvedSpan(spanned_source.into()));
        write_response(&response, w)
    }

    fn book<W>(
        &self,
        id: Option<Id>,
        param_directives: Option<&Vec<Directive>>,
        w: &mut W,
    ) -> io::Result<()>
    where
        W: Write,
    {
        match &self.parsed {
            Ok(Parsed {
                directives: parsed_directives,
                options,
                ..
            }) => {
                let raw_parsed_directives = if param_directives.is_none() {
                    Some(
                        parsed_directives
                            .iter()
                            .map(Into::<Directive>::into)
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                };
                let directives_to_book = match (param_directives, raw_parsed_directives.as_ref()) {
                    (Some(param_directives), _) => param_directives,
                    (None, Some(raw_parsed_directives)) => raw_parsed_directives,
                    _ => panic!("impossible"),
                };

                match booking::book(directives_to_book, options) {
                    Ok(booking::LoadSuccess {
                        directives,
                        warnings: _,
                    }) => {
                        // TODO warnings
                        let response = ResultResponse::new(id, ResultData::Booked(directives));
                        write_response(&response, w)
                    }

                    Err(LoadError { errors, .. }) => {
                        let reports = from_annotated_errors_or_warnings(&errors);
                        write_error_reports(None, reports, w)
                    }
                }
            }
            Err(errors) => {
                let reports = from_errors_or_warnings(errors);
                write_error_reports(id, reports, w)
            }
        }
    }
}

fn write_response<W>(response: &ResultResponse, w: &mut W) -> io::Result<()>
where
    W: Write,
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
                write_error(
                    response.id,
                    ERROR_INTERNAL,
                    Cow::Owned(e.to_string()),
                    None,
                    w,
                )
            }
        }
    }
}

fn write_error_reports<'a, W>(
    id: Option<Id<'a>>,
    reports: Vec<Report<'a>>,
    w: &mut W,
) -> io::Result<()>
where
    W: Write,
{
    write_error(
        id,
        ERROR_REPORT,
        Cow::Borrowed("Error reports"),
        Some(reports),
        w,
    )
}

fn write_other_error<'a, 'b, W>(
    id: Option<Id<'a>>,
    code: ErrorCode,
    message: Cow<'b, str>,
    w: &mut W,
) -> io::Result<()>
where
    W: Write,
{
    write_error(id, code, message, None, w)
}

fn write_error<'a, 'b, W>(
    id: Option<Id<'a>>,
    code: ErrorCode,
    message: Cow<'b, str>,
    data: Option<Vec<Report<'a>>>,
    w: &mut W,
) -> io::Result<()>
where
    W: Write,
{
    let response = ErrorResponse::new(id, code, message, data);
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
