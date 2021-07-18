use cookie::{Cookie, CookieJar};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use std::{fmt::Display, fs::File, io::Read, path::Path};
use time::OffsetDateTime;

#[derive(Debug, Parser)]
#[grammar = "cookie.pest"]
pub struct CookieParser {}

#[derive(Debug)]
pub enum Error {
    ParseError(pest::error::Error<Rule>),
    IoError(std::io::Error),
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Self::ParseError(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(e) => write!(f, "ParseError: {}", e),
            Self::IoError(e) => write!(f, "IoError: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseError(e) => Some(e),
            Self::IoError(e) => Some(e),
        }
    }
}

#[inline]
pub fn open(path: impl AsRef<Path>) -> Result<CookieJar, Error> {
    parse_file(File::open(path)?)
}
pub fn parse_file(mut f: File) -> Result<CookieJar, Error> {
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    parse(&buf)
}
pub fn parse(s: &str) -> Result<CookieJar, Error> {
    let file = CookieParser::parse(Rule::file, s)?.next().unwrap();
    let mut jar = CookieJar::new();
    for c in file
        .into_inner()
        .filter(|r: &Pair<Rule>| !matches!(r.as_rule(), Rule::EOI))
    {
        let mut fileds: Pairs<Rule> = c.into_inner();
        let domain = fileds.next().unwrap().as_str();
        let _ = fileds.next();
        let path = fileds.next().unwrap().as_str();
        let secure = fileds.next().unwrap().as_str() == "TRUE";
        let expiration: i64 = fileds.next().unwrap().as_str().parse().unwrap();
        let name = fileds.next().unwrap().as_str();
        let value = fileds.next().unwrap().as_str();
        let cookie = Cookie::build(name, value)
            .domain(domain)
            .path(path)
            .secure(secure)
            .expires(match expiration {
                0 => None,
                exp => Some(OffsetDateTime::from_unix_timestamp(exp)),
            })
            .finish();
        jar.add(cookie.into_owned());
    }
    Ok(jar)
}
