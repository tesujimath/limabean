use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use super::types::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Request<'a> {
    pub(crate) jsonrpc: &'a str,
    pub(crate) id: &'a str,
    #[serde(borrow)]
    #[serde(flatten)]
    pub(crate) method: RequestMethod<'a>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(tag = "method")]
pub(crate) enum RequestMethod<'a> {
    #[serde(rename = "parser.directives.get")]
    ParserDirectivesGet(ParserDirectivesGet),
    #[serde(rename = "directives.put")]
    #[serde(borrow)]
    DirectivesPut(DirectivesPut<'a>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ParserDirectivesGet {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct DirectivesPut<'a> {
    #[serde(borrow)]
    directives: Vec<Directive<'a>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ResultResponse<'a, 'b> {
    pub(crate) jsonrpc: &'static str,
    pub(crate) id: &'a str,
    pub(crate) result: ResultData<'b>,
}

impl<'a, 'b> ResultResponse<'a, 'b> {
    pub(crate) fn new(id: &'a str, result: ResultData<'b>) -> Self {
        ResultResponse {
            jsonrpc: JSONRPC_VERSION,
            id,
            result,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub(crate) enum ResultData<'a> {
    #[serde(rename = "parser.directives.get")]
    #[serde(borrow)]
    ParserDirectives(Vec<Directive<'a>>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ErrorResponse<'a, 'b> {
    pub(crate) jsonrpc: &'static str,
    pub(crate) id: &'a str,
    pub(crate) error: ErrorData<'b>,
}

impl<'a, 'b> ErrorResponse<'a, 'b> {
    pub(crate) fn new(id: &'a str, code: ErrorCode, message: Cow<'b, str>) -> Self {
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

const JSONRPC_VERSION: &str = "2.0";

// https://www.jsonrpc.org/specification#error_object
pub(crate) type ErrorCode = i32;
pub(crate) const ERROR_BEANFILE_IO_ERROR: ErrorCode = 1;
pub(crate) const ERROR_PARSE: ErrorCode = -32700;
pub(crate) const ERROR_INTERNAL: ErrorCode = -32603;
