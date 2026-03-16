use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::api::types::booked;

use super::types::{Report, raw::*};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Request<'a> {
    pub(crate) jsonrpc: &'a str,
    pub(crate) id: Option<Id<'a>>,
    #[serde(borrow)]
    #[serde(flatten)]
    pub(crate) method: RequestMethod<'a>,
}

// TODO RequestMethod should be generic and the actual methods moved out of this module,
// but the deserialize lifetimes are a bit tricksy
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "method")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RequestMethod<'a> {
    Status,
    #[serde(rename = "parser.directives")]
    ParserDirectives,
    #[serde(rename = "parser.format-report")]
    ParserFormatReport(Params<Vec<Report<'a>>>),
    #[serde(borrow)]
    Book(OptionalParams<Vec<Directive<'a>>>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Params<T> {
    pub(crate) params: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct OptionalParams<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ResultResponse<'i, 'a, 'b> {
    pub(crate) jsonrpc: &'static str,
    pub(crate) id: Option<Id<'i>>,
    pub(crate) result: ResultData<'a, 'b>,
}

impl<'i, 'a, 'b> ResultResponse<'i, 'a, 'b> {
    pub(crate) fn new(id: Option<Id<'i>>, result: ResultData<'a, 'b>) -> Self {
        ResultResponse {
            jsonrpc: JSONRPC_VERSION,
            id,
            result,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResultData<'a, 'b> {
    Ok,
    #[serde(borrow)]
    RawDirectives(Vec<Directive<'a>>),
    Report(Cow<'b, str>),
    // TODO also return warnings with booked
    Booked(Vec<booked::Directive<'a>>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ErrorResponse<'a, 'b> {
    pub(crate) jsonrpc: &'static str,
    pub(crate) id: Option<Id<'a>>,
    pub(crate) error: ErrorData<'b>,
}

impl<'a, 'b> ErrorResponse<'a, 'b> {
    pub(crate) fn new(id: Option<Id<'a>>, code: ErrorCode, message: Cow<'b, str>) -> Self {
        ErrorResponse {
            jsonrpc: JSONRPC_VERSION,
            id,
            error: ErrorData { code, message },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ErrorData<'a> {
    code: i32,
    message: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum Id<'a> {
    String(&'a str),
    Int(i32),
    Float(f64),
}

pub(crate) const JSONRPC_VERSION: &str = "2.0";

// https://www.jsonrpc.org/specification#error_object
pub(crate) type ErrorCode = i32;
pub(crate) const ERROR_BEANFILE_IO_ERROR: ErrorCode = 1;
pub(crate) const ERROR_PARSE: ErrorCode = -32700;
pub(crate) const ERROR_INVALID_REQUEST: ErrorCode = -32600;
pub(crate) const ERROR_INTERNAL: ErrorCode = -32603;
