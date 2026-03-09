use serde::{Deserialize, Serialize};

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
