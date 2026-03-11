use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use super::types::raw::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Request<'a> {
    pub(crate) jsonrpc: &'a str,
    pub(crate) id: Option<Id<'a>>,
    #[serde(borrow)]
    #[serde(flatten)]
    pub(crate) method: RequestMethod<'a>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(tag = "method")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RequestMethod<'a> {
    Status,
    #[serde(rename = "parser.directives")]
    ParserDirectives,
    #[serde(borrow)]
    Book(Book<'a>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Book<'a> {
    #[serde(borrow)]
    #[serde(skip_serializing_if = "Option::is_none")]
    directives: Option<Vec<Directive<'a>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ResultResponse<'a, 'b> {
    pub(crate) jsonrpc: &'static str,
    pub(crate) id: Option<Id<'a>>,
    pub(crate) result: ResultData<'b>,
}

impl<'a, 'b> ResultResponse<'a, 'b> {
    pub(crate) fn new(id: Option<Id<'a>>, result: ResultData<'b>) -> Self {
        ResultResponse {
            jsonrpc: JSONRPC_VERSION,
            id,
            result,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResultData<'a> {
    Ok,
    #[serde(borrow)]
    RawDirectives(Vec<Directive<'a>>),
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
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
