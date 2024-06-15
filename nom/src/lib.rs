pub mod helper;
pub mod model;
pub mod xmlchar;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::{preceded, tuple};
use nom::IResult;

/// Name - (Char* ':' Char*)
///
/// [\[4\] NCName](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-NCName)
pub fn ncname(input: &str) -> IResult<&str, &str> {
    // FIXME: not name
    xmlchar::name_char_except1(":")(input)
}

/// PrefixedName | UnprefixedName
///
/// [\[7\] QName](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-QName)
pub fn qname(input: &str) -> IResult<&str, model::QName<'_>> {
    alt((
        map(prefixed_name, model::QName::from),
        map(ncname, model::QName::from),
    ))(input)
}

/// Prefix ':' LocalPart
///
/// [\[8\] PrefixedName](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-PrefixedName)
fn prefixed_name(input: &str) -> IResult<&str, model::PrefixedName> {
    map(
        tuple((ncname, preceded(tag(":"), ncname))),
        model::PrefixedName::from,
    )(input)
}
