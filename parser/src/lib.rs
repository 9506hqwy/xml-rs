pub mod model;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, digit1, hex_digit1, multispace0, multispace1};
use nom::combinator::{map, opt, recognize};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::{AsChar, IResult, InputTakeAtPosition};
use xml_nom::{helper, ncname, qname, xmlchar};

// -----------------------------------------------------------------------------------------------

/// prolog element Misc*
///
/// [\[1\] document](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-document)
pub fn document(input: &str) -> IResult<&str, model::Document<'_>> {
    map(tuple((prolog, element, many0(misc))), model::Document::from)(input)
}

/// Recognizes zero or more XML characters.
///
/// #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
///
/// [\[2\] Char](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Char)
fn multichar0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position_complete(|i| !xmlchar::is_char(i.as_char()))
}

/// Recognizes zero or more XML starting name characters.
///
/// ":" | \[A-Z] | "_" | \[a-z] | \[#xC0-#xD6] | \[#xD8-#xF6] | \[#xF8-#x2FF] | \[#x370-#x37D] |
/// \[#x37F-#x1FFF] | \[#x200C-#x200D] | \[#x2070-#x218F] | \[#x2C00-#x2FEF] | \[#x3001-#xD7FF] |
/// \[#xF900-#xFDCF] | \[#xFDF0-#xFFFD] | \[#x10000-#xEFFFF]
///
/// [\[4\] NameStartChar](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NameStartChar)
fn multinamestartchar0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position_complete(|i| !xmlchar::is_name_start_char(i.as_char()))
}

/// Recognizes zero or more XML name characters.
///
/// NameStartChar | "-" | "." | \[0-9] | #xB7 | \[#x0300-#x036F] | \[#x203F-#x2040]
///
/// [\[4a\] NameChar](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NameChar)
fn multinamechar0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position_complete(|i| !xmlchar::is_name_char(i.as_char()))
}

/// NameStartChar (NameChar)*
///
/// [\[5\] Name](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Name)
fn name(input: &str) -> IResult<&str, &str> {
    recognize(tuple((multinamestartchar0, multinamechar0)))(input)
}

/// (NameChar)+
///
/// [\[7\] Nmtoken](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Nmtoken)
fn nmtoken<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position1_complete(|i| !xmlchar::is_name_char(i.as_char()), ErrorKind::Fail)
}

/// '"' ([^%&"] | PEReference | Reference)* '"' | "'" ([^%&'] | PEReference | Reference)* "'"
///
/// [\[9\] EntityValue](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EntityValue)
fn entity_value(input: &str) -> IResult<&str, Vec<model::EntityValue>> {
    alt((
        delimited(
            tag("\""),
            many0(alt((
                map(xmlchar::char_except1("%&\""), model::EntityValue::text),
                map(pe_reference, model::EntityValue::pe_reference),
                map(reference, model::EntityValue::reference),
            ))),
            tag("\""),
        ),
        delimited(
            tag("'"),
            many0(alt((
                map(xmlchar::char_except1("%&'"), model::EntityValue::text),
                map(pe_reference, model::EntityValue::pe_reference),
                map(reference, model::EntityValue::reference),
            ))),
            tag("'"),
        ),
    ))(input)
}

/// '"' ([^<&"] | Reference)* '"' |  "'" ([^<&'] | Reference)* "'"
///
/// [\[10\] AttValue](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-AttValue)
fn att_value(input: &str) -> IResult<&str, Vec<model::AttributeValue<'_>>> {
    alt((
        delimited(
            tag("\""),
            many0(alt((
                map(xmlchar::char_except1("<&\""), model::AttributeValue::from),
                map(reference, model::AttributeValue::from),
            ))),
            tag("\""),
        ),
        delimited(
            tag("'"),
            many0(alt((
                map(xmlchar::char_except1("<&'"), model::AttributeValue::from),
                map(reference, model::AttributeValue::from),
            ))),
            tag("'"),
        ),
    ))(input)
}

/// ('"' [^"]* '"') | ("'" [^']* "'")
///
/// [\[11\] SystemLiteral](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-SystemLiteral)
fn system_literal(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(tag("\""), xmlchar::char_except0("\""), tag("\"")),
        delimited(tag("'"), xmlchar::char_except0("'"), tag("'")),
    ))(input)
}

/// '"' PubidChar* '"' | "'" (PubidChar - "'")* "'"
///
/// [\[12\] PubidLiteral](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PubidLiteral)
fn pubid_literal(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(tag("\""), multipubidchar0, tag("\"")),
        delimited(tag("'"), xmlchar::pubid_char_except0("'"), tag("'")),
    ))(input)
}

/// Recognizes zero or more public identifier characters.
///
/// #x20 | #xD | #xA | [a-zA-Z0-9] | [-'()+,./:=?;!*#@$_%]
///
/// [[13] PubidChar](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PubidChar)
fn multipubidchar0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position_complete(|i| !xmlchar::is_pubid_char(i.as_char()))
}

/// \[^<&]* - (\[^<&]* ']]>' \[^<&]*)
///
/// [\[14\] CharData](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-CharData)
fn char_data(input: &str) -> IResult<&str, &str> {
    helper::take_until(xmlchar::char_except0("<&"), "]]>")(input)
}

/// '\<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
///
/// [\[15\] Comment](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Comment)
fn comment(input: &str) -> IResult<&str, model::Comment<'_>> {
    map(
        delimited(
            tag("<!--"),
            recognize(many0(tuple((opt(tag("-")), xmlchar::char_except1("-"))))),
            tag("-->"),
        ),
        model::Comment::from,
    )(input)
}

/// '\<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'
///
/// [\[16\] PI](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PI)
fn pi(input: &str) -> IResult<&str, model::PI<'_>> {
    map(
        delimited(
            tag("<?"),
            tuple((
                pi_target,
                opt(preceded(multispace1, helper::take_until(multichar0, "?>"))),
            )),
            tag("?>"),
        ),
        model::PI::from,
    )(input)
}

/// Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))
///
/// [\[17\] PITarget](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PITarget)
fn pi_target(input: &str) -> IResult<&str, &str> {
    helper::take_except(name, "xml")(input)
}

/// CDStart CData CDEnd
///
/// [\[18\] CDSect](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-CDSect)
fn cdsect(input: &str) -> IResult<&str, model::CData<'_>> {
    map(
        delimited(
            tag("<![CDATA["),                      // [19] CDStart
            helper::take_until(multichar0, "]]>"), // [20] CData
            tag("]]>"),                            // [21] CDEnd
        ),
        model::CData::from,
    )(input)
}

/// XMLDecl? Misc* (doctypedecl Misc*)?
///
/// [\[22\] prolog](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-prolog)
fn prolog(input: &str) -> IResult<&str, model::Prolog<'_>> {
    map(
        tuple((
            opt(xml_decl),
            many0(misc),
            opt(tuple((doctype_decl, many0(misc)))),
        )),
        model::Prolog::from,
    )(input)
}

/// '\<?xml' VersionInfo EncodingDecl? SDDecl? S? '?>'
///
/// [\[23\] XMLDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-XMLDecl)
fn xml_decl(input: &str) -> IResult<&str, model::DeclarationXml<'_>> {
    map(
        delimited(
            tag("<?xml"),
            tuple((version_info, opt(encoding_decl), opt(sd_decl))),
            tuple((multispace0, tag("?>"))),
        ),
        model::DeclarationXml::from,
    )(input)
}

/// S 'version' Eq ("'" VersionNum "'" | '"' VersionNum '"')
///
/// [\[24\] VersionInfo](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-VersionInfo)
fn version_info(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((multispace1, tag("version"), eq)),
        alt((
            delimited(tag("'"), version_num, tag("'")),
            delimited(tag("\""), version_num, tag("\"")),
        )),
    )(input)
}

/// S? '=' S?
///
/// [\[25\] Eq](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Eq)
fn eq(input: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag("="), multispace0)(input)
}

/// '1.' [0-9]+
///
/// [\[26\] VersionNum](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-VersionNum)
pub fn version_num(input: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("1."), digit1)))(input)
}

///  Comment | PI | S
///
/// [\[27\] Misc](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Misc)
fn misc(input: &str) -> IResult<&str, model::Misc<'_>> {
    alt((
        map(comment, model::Misc::from),
        map(pi, model::Misc::from),
        map(multispace1, model::Misc::from),
    ))(input)
}

/// '\<!DOCTYPE' S Name (S ExternalID)? S? ('[' intSubset ']' S?)? '>'
///
/// [\[28\] doctypedecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-doctypedecl)
///
/// [\[16\] doctypedecl](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-doctypedecl)
fn doctype_decl(input: &str) -> IResult<&str, model::DeclarationDoc<'_>> {
    map(
        tuple((
            preceded(tuple((tag("<!DOCTYPE"), multispace1)), qname),
            terminated(opt(preceded(multispace1, external_id)), multispace0),
            terminated(
                opt(delimited(
                    tag("["),
                    int_subset,
                    tuple((tag("]"), multispace0)),
                )),
                tag(">"),
            ),
        )),
        model::DeclarationDoc::from,
    )(input)
}

/// PEReference | S
///
/// [\[28a\] DeclSep](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-DeclSep)
fn decl_sep(input: &str) -> IResult<&str, &str> {
    alt((pe_reference, multispace1))(input)
}

/// (markupdecl | DeclSep)*
///
/// [\[28b\] intSubset](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-intSubset)
fn int_subset(input: &str) -> IResult<&str, Vec<model::InternalSubset<'_>>> {
    many0(alt((
        map(markup_decl, model::InternalSubset::from),
        map(decl_sep, model::InternalSubset::from),
    )))(input)
}

/// elementdecl | AttlistDecl | EntityDecl | NotationDecl | PI | Comment
///
/// [\[29\] markupdecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-markupdecl)
fn markup_decl(input: &str) -> IResult<&str, model::DeclarationMarkup<'_>> {
    alt((
        map(element_decl, model::DeclarationMarkup::element),
        map(attlist_decl, model::DeclarationMarkup::attributes),
        map(entity_decl, model::DeclarationMarkup::from),
        map(notation_decl, model::DeclarationMarkup::from),
        map(pi, model::DeclarationMarkup::from),
        map(comment, model::DeclarationMarkup::from),
    ))(input)
}

/// S 'standalone' Eq (("'" ('yes' | 'no') "'") | ('"' ('yes' | 'no') '"'))
///
/// [\[32\] SDDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-SDDecl)
fn sd_decl(input: &str) -> IResult<&str, bool> {
    map(
        preceded(
            tuple((multispace1, tag("standalone"), eq)),
            alt((
                delimited(tag("'"), tag("yes"), tag("'")),
                delimited(tag("\""), tag("yes"), tag("\"")),
                delimited(tag("'"), tag("no"), tag("'")),
                delimited(tag("\""), tag("no"), tag("\"")),
            )),
        ),
        |v| v == "yes",
    )(input)
}

/// EmptyElemTag | STag content ETag
///
/// [\[39\] element](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-element)
fn element(input: &str) -> IResult<&str, model::Element<'_>> {
    alt((
        empty_entity_tag,
        map(tuple((stag, content, etag)), |(s, c, _)| s.set_content(c)),
    ))(input)
}

/// '\<' Name (S Attribute)* S? '>'
///
/// [\[40\] STag](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-STag)
///
/// [\[12\] STag](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-STag)
fn stag(input: &str) -> IResult<&str, model::Element<'_>> {
    map(
        delimited(
            tag("<"),
            tuple((qname, many0(preceded(multispace1, attribute)))),
            tuple((multispace0, tag(">"))),
        ),
        model::Element::from,
    )(input)
}

/// Name Eq AttValue
///
/// [\[41\] Attribute](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Attribute)
///
/// [\[15\] Attribute](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-Attribute)
fn attribute(input: &str) -> IResult<&str, model::Attribute<'_>> {
    map(
        tuple((
            alt((ns_att_name, map(qname, model::AttributeName::from))),
            preceded(eq, att_value),
        )),
        model::Attribute::from,
    )(input)
}

/// '\</' Name S? '>'
///
/// [\[42\] ETag](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-ETag)
///
/// [\[13\] ETag](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-ETag)
fn etag(input: &str) -> IResult<&str, ()> {
    map(
        delimited(tag("</"), qname, tuple((multispace0, tag(">")))),
        |_| (),
    )(input)
}

/// CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
///
/// [\[43\] content](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-content)
fn content(input: &str) -> IResult<&str, model::Content<'_>> {
    map(
        tuple((
            opt(char_data),
            many0(tuple((
                alt((
                    map(element, model::Contents::from),
                    map(reference, model::Contents::from),
                    map(cdsect, model::Contents::from),
                    map(pi, model::Contents::from),
                    map(comment, model::Contents::from),
                )),
                opt(char_data),
            ))),
        )),
        |(head, children)| {
            model::Content::from((
                head,
                children.into_iter().map(model::ContentCell::from).collect(),
            ))
        },
    )(input)
}

/// '\<' Name (S Attribute)* S? '/>'
///
/// [\[44\] EmptyElemTag](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EmptyElemTag)
///
/// [\[14\] EmptyElemTag](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-EmptyElemTag)
fn empty_entity_tag(input: &str) -> IResult<&str, model::Element<'_>> {
    map(
        delimited(
            tag("<"),
            tuple((qname, many0(preceded(multispace1, attribute)))),
            tuple((multispace0, tag("/>"))),
        ),
        model::Element::from,
    )(input)
}

/// '\<!ELEMENT' S Name S contentspec S? '>'
///
/// [\[45\] elementdecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-elementdecl)
///
/// [\[17\] elementdecl](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-elementdecl)
fn element_decl(input: &str) -> IResult<&str, model::DeclarationElement<'_>> {
    map(
        delimited(
            tuple((tag("<!ELEMENT"), multispace1)),
            tuple((qname, preceded(multispace1, content_spec))),
            tuple((multispace0, tag(">"))),
        ),
        model::DeclarationElement::from,
    )(input)
}

/// 'EMPTY' | 'ANY' | Mixed | children
///
/// [\[46\] contentspec](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-contentspec)
fn content_spec(input: &str) -> IResult<&str, model::DeclarationContent<'_>> {
    alt((
        map(tag("EMPTY"), |_| model::DeclarationContent::Empty),
        map(tag("ANY"), |_| model::DeclarationContent::Any),
        map(mixed, model::DeclarationContent::Mixed),
        map(children, model::DeclarationContent::Children),
    ))(input)
}

/// (choice | seq) ('?' | '*' | '+')?
///
/// [\[47\] children](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-children)
fn children(input: &str) -> IResult<&str, model::DeclarationContentItem<'_>> {
    alt((
        map(
            tuple((seq, opt(alt((tag("?"), tag("*"), tag("+")))))),
            |(v, q)| model::DeclarationContentItem::Seq(v, q),
        ),
        map(
            tuple((choice, opt(alt((tag("?"), tag("*"), tag("+")))))),
            |(v, q)| model::DeclarationContentItem::Choice(v, q),
        ),
    ))(input)
}

/// (Name | choice | seq) ('?' | '*' | '+')?
///
/// [\[48\] cp](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-cp)
///
/// [\[18\] cp](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-cp)
fn cp(input: &str) -> IResult<&str, model::DeclarationContentItem<'_>> {
    alt((
        map(
            tuple((seq, opt(alt((tag("?"), tag("*"), tag("+")))))),
            |(v, q)| model::DeclarationContentItem::Seq(v, q),
        ),
        map(
            tuple((choice, opt(alt((tag("?"), tag("*"), tag("+")))))),
            |(v, q)| model::DeclarationContentItem::Choice(v, q),
        ),
        map(
            tuple((qname, opt(alt((tag("?"), tag("*"), tag("+")))))),
            |(v, q)| model::DeclarationContentItem::Name(v, q),
        ),
    ))(input)
}

/// '(' S? cp ( S? '|' S? cp )+ S? ')'
///
/// [\[49\] choice](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-choice)
fn choice(input: &str) -> IResult<&str, Vec<model::DeclarationContentItem<'_>>> {
    map(
        delimited(
            tuple((tag("("), multispace0)),
            tuple((
                cp,
                many1(preceded(tuple((multispace0, tag("|"), multispace0)), cp)),
            )),
            tuple((multispace0, tag(")"))),
        ),
        |(f, mut r)| {
            r.insert(0, f);
            r
        },
    )(input)
}

/// '(' S? cp ( S? ',' S? cp )* S? ')'
///
/// [\[50\] seq](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-seq)
fn seq(input: &str) -> IResult<&str, Vec<model::DeclarationContentItem<'_>>> {
    map(
        delimited(
            tuple((tag("("), multispace0)),
            tuple((
                cp,
                many0(preceded(tuple((multispace0, tag(","), multispace0)), cp)),
            )),
            tuple((multispace0, tag(")"))),
        ),
        |(f, mut r)| {
            r.insert(0, f);
            r
        },
    )(input)
}

/// '(' S? '#PCDATA' (S? '|' S? Name)* S? ')*' | '(' S? '#PCDATA' S? ')'
///
/// [\[51\] Mixed](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Mixed)
///
/// [\[19\] Mixed](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-Mixed)
fn mixed(input: &str) -> IResult<&str, Option<Vec<xml_nom::model::QName<'_>>>> {
    alt((
        map(
            delimited(
                tuple((tag("("), multispace0, tag("#PCDATA"))),
                many0(preceded(tuple((multispace0, tag("|"), multispace0)), qname)),
                tuple((multispace0, tag(")*"))),
            ),
            Some,
        ),
        map(
            tuple((tag("("), multispace0, tag("#PCDATA"), multispace0, tag(")"))),
            |_| None,
        ),
    ))(input)
}

/// '\<!ATTLIST' S Name AttDef* S? '>'
///
/// [\[52\] AttlistDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-AttlistDecl)
///
/// [\[20\] AttlistDecl](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-AttlistDecl)
fn attlist_decl(input: &str) -> IResult<&str, model::DeclarationAtt<'_>> {
    map(
        delimited(
            tuple((tag("<!ATTLIST"), multispace1)),
            tuple((qname, many0(att_def))),
            tuple((multispace0, tag(">"))),
        ),
        model::DeclarationAtt::from,
    )(input)
}

/// S Name S AttType S DefaultDecl
///
/// [\[53\] AttDef](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-AttDef)
///
/// [\[21\] AttDef](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-AttDef)
fn att_def(input: &str) -> IResult<&str, model::DeclarationAttDef<'_>> {
    map(
        tuple((
            preceded(
                multispace1,
                alt((
                    map(qname, model::DeclarationAttName::Attr),
                    map(ns_att_name, model::DeclarationAttName::Namsspace),
                )),
            ),
            preceded(multispace1, att_type),
            preceded(multispace1, default_decl),
        )),
        model::DeclarationAttDef::from,
    )(input)
}

/// StringType | TokenizedType | EnumeratedType
///
/// [\[54\] AttType](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-AttType)
fn att_type(input: &str) -> IResult<&str, model::DeclarationAttType<'_>> {
    alt((
        enumerated_type,
        map(tag("CDATA"), |_| model::DeclarationAttType::Cdata), // [55] StringType
        map(tag("IDREFS"), |_| model::DeclarationAttType::IdRefs), // [56] TokenizedType
        map(tag("IDREF"), |_| model::DeclarationAttType::IdRef), // [56] TokenizedType
        map(tag("ID"), |_| model::DeclarationAttType::Id),       // [56] TokenizedType
        map(tag("ENTITIES"), |_| model::DeclarationAttType::Entities), // [56] TokenizedType
        map(tag("ENTITY"), |_| model::DeclarationAttType::Entity), // [56] TokenizedType
        map(tag("NMTOKENS"), |_| model::DeclarationAttType::NmTokens), // [56] TokenizedType
        map(tag("NMTOKEN"), |_| model::DeclarationAttType::NmToken), // [56] TokenizedType
    ))(input)
}

/// NotationType | Enumeration
///
/// [\[57\] EnumeratedType](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EnumeratedType)
fn enumerated_type(input: &str) -> IResult<&str, model::DeclarationAttType<'_>> {
    alt((
        map(notation_type, model::DeclarationAttType::Notation),
        map(enumeration, model::DeclarationAttType::Enumeration),
    ))(input)
}

/// 'NOTATION' S '(' S? Name (S? '|' S? Name)* S? ')'
///
/// [\[58\] NotationType](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NotationType)
fn notation_type(input: &str) -> IResult<&str, Vec<&str>> {
    map(
        delimited(
            tuple((tag("NOTATION"), multispace1, tag("("), multispace0)),
            tuple((
                name,
                many0(preceded(tuple((multispace0, tag("|"), multispace0)), name)),
            )),
            tuple((multispace0, tag(")"))),
        ),
        |(f, mut r)| {
            r.insert(0, f);
            r
        },
    )(input)
}

/// '(' S? Nmtoken (S? '|' S? Nmtoken)* S? ')'
///
/// [\[59\] Enumeration](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Enumeration)
fn enumeration(input: &str) -> IResult<&str, Vec<&str>> {
    map(
        delimited(
            tuple((tag("("), multispace0)),
            tuple((
                nmtoken,
                many0(preceded(
                    tuple((multispace0, tag("|"), multispace0)),
                    nmtoken,
                )),
            )),
            tuple((multispace0, tag(")"))),
        ),
        |(f, mut r)| {
            r.insert(0, f);
            r
        },
    )(input)
}

/// '#REQUIRED' | '#IMPLIED' | (('#FIXED' S)? AttValue)
///
/// [\[60\] DefaultDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-DefaultDecl)
fn default_decl(input: &str) -> IResult<&str, model::DeclarationAttDefault<'_>> {
    alt((
        map(tag("#REQUIRED"), |_| model::DeclarationAttDefault::Required),
        map(tag("#IMPLIED"), |_| model::DeclarationAttDefault::Implied),
        map(
            tuple((opt(terminated(tag("#FIXED"), multispace1)), att_value)),
            |(f, a)| model::DeclarationAttDefault::Value(f, a),
        ),
    ))(input)
}

/// '&#' [0-9]+ ';' | '&#x' [0-9a-fA-F]+ ';'
///
/// [\[66\] CharRef](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-CharRef)
fn char_ref(input: &str) -> IResult<&str, model::Reference<'_>> {
    alt((
        map(
            delimited(tag("&#"), digit1, tag(";")),
            model::Reference::digit,
        ),
        map(
            delimited(tag("&#x"), hex_digit1, tag(";")),
            model::Reference::hex,
        ),
    ))(input)
}

/// EntityRef | CharRef
///
/// [\[67\] Reference](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Reference)
fn reference(input: &str) -> IResult<&str, model::Reference<'_>> {
    alt((entity_ref, char_ref))(input)
}

/// '&' Name ';'
///
/// [\[68\] EntityRef](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EntityRef)
fn entity_ref(input: &str) -> IResult<&str, model::Reference<'_>> {
    map(
        delimited(tag("&"), name, tag(";")),
        model::Reference::entity,
    )(input)
}

/// '%' Name ';'
///
/// [\[69\] PEReference](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PEReference)
fn pe_reference(input: &str) -> IResult<&str, &str> {
    delimited(tag("%"), name, tag(";"))(input)
}

/// GEDecl | PEDecl
///
/// [\[70\] EntityDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EntityDecl)
fn entity_decl(input: &str) -> IResult<&str, model::DeclarationEntity<'_>> {
    alt((
        map(ge_decl, model::DeclarationEntity::from),
        map(pe_decl, model::DeclarationEntity::from),
    ))(input)
}

/// '\<!ENTITY' S Name S EntityDef S? '>'
///
/// [\[71\] GEDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-GEDecl)
fn ge_decl(input: &str) -> IResult<&str, model::DeclarationGeneralEntity<'_>> {
    map(
        tuple((
            delimited(tuple((tag("<!ENTITY"), multispace1)), name, multispace1),
            terminated(entity_def, tuple((multispace0, tag(">")))),
        )),
        model::DeclarationGeneralEntity::from,
    )(input)
}

/// '\<!ENTITY' S '%' S Name S PEDef S? '>'
///
/// [\[72\] PEDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PEDecl)
fn pe_decl(input: &str) -> IResult<&str, model::DeclarationParameterEntity<'_>> {
    map(
        tuple((
            delimited(
                tuple((tag("<!ENTITY"), multispace1, tag("%"), multispace1)),
                name,
                multispace1,
            ),
            terminated(pe_def, tuple((multispace0, tag(">")))),
        )),
        model::DeclarationParameterEntity::from,
    )(input)
}

/// EntityValue | (ExternalID NDataDecl?)
///
/// [\[73\] EntityDef](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EntityDef)
fn entity_def(input: &str) -> IResult<&str, model::DeclarationEntityDef<'_>> {
    alt((
        map(entity_value, model::DeclarationEntityDef::from),
        map(
            tuple((external_id, opt(ndata_decl))),
            model::DeclarationEntityDef::from,
        ),
    ))(input)
}

/// EntityValue | ExternalID
///
/// [\[74\] PEDef](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PEDef)
fn pe_def(input: &str) -> IResult<&str, model::DeclarationPeDef<'_>> {
    alt((
        map(entity_value, model::DeclarationPeDef::from),
        map(external_id, model::DeclarationPeDef::from),
    ))(input)
}

/// 'SYSTEM' S SystemLiteral | 'PUBLIC' S PubidLiteral S SystemLiteral
///
/// [\[75\] ExternalID](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-ExternalID)
fn external_id(input: &str) -> IResult<&str, model::ExternalId<'_>> {
    alt((
        map(
            preceded(tuple((tag("SYSTEM"), multispace1)), system_literal),
            model::ExternalId::from,
        ),
        map(
            preceded(
                tuple((tag("PUBLIC"), multispace1)),
                tuple((pubid_literal, preceded(multispace1, system_literal))),
            ),
            model::ExternalId::from,
        ),
    ))(input)
}

/// S 'NDATA' S Name
///
/// [[76] NDataDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NDataDecl)
fn ndata_decl(input: &str) -> IResult<&str, &str> {
    preceded(tuple((multispace1, tag("NDATA"), multispace1)), name)(input)
}

/// S 'encoding' Eq ('"' EncName '"' | "'" EncName "'" )
///
/// [\[80\] EncodingDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EncodingDecl)
fn encoding_decl(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((multispace1, tag("encoding"), eq)),
        alt((
            delimited(tag("'"), enc_name, tag("'")),
            delimited(tag("\""), enc_name, tag("\"")),
        )),
    )(input)
}

/// \[A-Za-z] (\[A-Za-z0-9._] | '-')*
///
/// [\[81\] EncName](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-EncName)
fn enc_name(input: &str) -> IResult<&str, &str> {
    recognize(tuple((alpha1, xmlchar::enc_name0)))(input)
}

/// '\<!NOTATION' S Name S (ExternalID | PublicID) S? '>'
///
/// [\[82\] NotationDecl](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NotationDecl)
fn notation_decl(input: &str) -> IResult<&str, model::DeclarationNotation<'_>> {
    map(
        tuple((
            preceded(tuple((tag("<!NOTATION"), multispace1)), name),
            delimited(
                multispace1,
                alt((
                    map(external_id, model::DeclarationNotationId::from),
                    map(public_id, model::DeclarationNotationId::from),
                )),
                tuple((multispace0, tag(">"))),
            ),
        )),
        model::DeclarationNotation::from,
    )(input)
}

/// 'PUBLIC' S PubidLiteral
///
/// [\[83\] PublicID](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-PublicID)
fn public_id(input: &str) -> IResult<&str, &str> {
    preceded(tuple((tag("PUBLIC"), multispace1)), pubid_literal)(input)
}

// -----------------------------------------------------------------------------------------------

/// PrefixedAttName | DefaultAttName
///
/// [[1] NSAttName](https://www.w3.org/TR/2009/REC-xml-names-20091208/#NT-NSAttName)
fn ns_att_name(input: &str) -> IResult<&str, model::AttributeName<'_>> {
    alt((
        map(preceded(tag("xmlns:"), ncname), model::AttributeName::from), // [2] PrefixedAttName
        map(tag("xmlns"), |_| model::AttributeName::default()),           // [3] DefaultAttName
    ))(input)
}

// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use xml_nom::model::QName;

    #[test]
    fn test_document() {
        let (rest, ret) = document("<root>></root>").unwrap();
        assert_eq!("", rest);
        assert_eq!(QName::from("root"), ret.element.name);
    }

    #[test]
    fn test_entity_value() {
        let (rest, ret) = entity_value("\"aaa\"").unwrap();
        assert_eq!("", rest);
        assert_eq!(vec![model::EntityValue::text("aaa")], ret);

        let (rest, ret) = entity_value("\"%aaa;\"").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            vec![model::EntityValue::ParameterEntityReference("aaa")],
            ret
        );

        let (rest, ret) = entity_value("\"&aaa;\"").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            vec![model::EntityValue::Reference(model::Reference::Entity(
                "aaa"
            ))],
            ret
        );

        let (rest, ret) = entity_value("'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!(vec![model::EntityValue::text("aaa")], ret);

        let (rest, ret) = entity_value("'%aaa;'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            vec![model::EntityValue::ParameterEntityReference("aaa")],
            ret
        );

        let (rest, ret) = entity_value("'&aaa;'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            vec![model::EntityValue::Reference(model::Reference::Entity(
                "aaa"
            ))],
            ret
        );
    }

    #[test]
    fn test_att_value() {
        let (rest, ret) = att_value("\"aaa\"").unwrap();
        assert_eq!("", rest);
        assert_eq!(vec![model::AttributeValue::from("aaa")], ret);

        let (rest, ret) = att_value("\"&aaa;\"").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            vec![model::AttributeValue::Reference(model::Reference::Entity(
                "aaa"
            ))],
            ret
        );

        let (rest, ret) = att_value("'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!(vec![model::AttributeValue::from("aaa")], ret);

        let (rest, ret) = att_value("'&aaa;'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            vec![model::AttributeValue::Reference(model::Reference::Entity(
                "aaa"
            ))],
            ret
        );
    }

    #[test]
    fn test_system_literal() {
        let (rest, ret) = system_literal("\"aaa\"").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret);

        let (rest, ret) = system_literal("'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret);
    }

    #[test]
    fn test_pubid_literal() {
        let (rest, ret) = pubid_literal("\"aaa\"").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret);

        let (rest, ret) = pubid_literal("'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret);
    }

    #[test]
    fn test_char_data() {
        let (rest, ret) = char_data("").unwrap();
        assert_eq!("", rest);
        assert_eq!("", ret);

        let (rest, ret) = char_data("<").unwrap();
        assert_eq!("<", rest);
        assert_eq!("", ret);

        let (rest, ret) = char_data("a").unwrap();
        assert_eq!("", rest);
        assert_eq!("a", ret);

        let (rest, ret) = char_data("a]]").unwrap();
        assert_eq!("", rest);
        assert_eq!("a]]", ret);

        let (rest, ret) = char_data("a]]>b").unwrap();
        assert_eq!("]]>b", rest);
        assert_eq!("a", ret);
    }

    #[test]
    fn test_comment() {
        let (rest, ret) = comment("<!-- declarations for <head> & <body> -->").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Comment::from(" declarations for <head> & <body> "),
            ret
        );

        let (rest, ret) = comment("<!---->").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Comment::from(""), ret);

        let (rest, ret) = comment("<!---a-->").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Comment::from("-a"), ret);

        let _err = comment("<!----->").err().unwrap();
    }

    #[test]
    fn test_pi() {
        let (rest, ret) = pi("<?a?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PI::from(("a", None)), ret);

        let (rest, ret) = pi("<?a b?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PI::from(("a", Some("b"))), ret);

        let (rest, ret) = pi("<?a b> ?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::PI::from(("a", Some("b> "))), ret);
    }

    #[test]
    fn test_pi_target() {
        let (rest, ret) = pi_target("aaa").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret);

        let _err = pi_target("XML").err().unwrap();
    }

    #[test]
    fn test_cdsect() {
        let (rest, ret) = cdsect("<![CDATA[]]>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::CData::from(""), ret);

        let (rest, ret) = cdsect("<![CDATA[aaa]]>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::CData::from("aaa"), ret);
    }

    #[test]
    fn test_prolog() {
        let (rest, ret) = prolog("").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Prolog::from((None, vec![], None)), ret);

        let (rest, ret) = prolog("<?xml version='1.0'?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Prolog::from((
                Some(model::DeclarationXml::from(("1.0", None, None))),
                vec![],
                None
            )),
            ret
        );

        let (rest, ret) = prolog("<!-- aaa -->").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Prolog::from((
                None,
                vec![model::Misc::Comment(model::Comment::from(" aaa "))],
                None
            )),
            ret
        );

        let (rest, ret) = prolog("<!DOCTYPE aaa>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Prolog::from((
                None,
                vec![],
                Some((
                    model::DeclarationDoc::from((QName::from("aaa"), None, None)),
                    vec![]
                ))
            )),
            ret
        );

        let (rest, ret) = prolog("<!DOCTYPE aaa><!-- aaa -->").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Prolog::from((
                None,
                vec![],
                Some((
                    model::DeclarationDoc::from((QName::from("aaa"), None, None)),
                    vec![model::Misc::Comment(model::Comment::from(" aaa "))]
                ))
            )),
            ret
        );
    }

    #[test]
    fn test_xml_decl() {
        let (rest, ret) = xml_decl("<?xml version='1.0' ?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationXml::from(("1.0", None, None)), ret);

        let (rest, ret) = xml_decl("<?xml version='1.0' encoding='utf-8'?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationXml::from(("1.0", Some("utf-8"), None)),
            ret
        );

        let (rest, ret) = xml_decl("<?xml version='1.0' standalone='yes'?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationXml::from(("1.0", None, Some(true))), ret);

        let (rest, ret) = xml_decl("<?xml version='1.0' standalone='no'?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationXml::from(("1.0", None, Some(false))), ret);
    }

    #[test]
    fn test_version_info() {
        let (rest, ret) = version_info(" version='1.0'").unwrap();
        assert_eq!("", rest);
        assert_eq!("1.0", ret);

        let (rest, ret) = version_info(" version = \"1.1\"").unwrap();
        assert_eq!("", rest);
        assert_eq!("1.1", ret);
    }

    #[test]
    fn test_misc() {
        let (rest, ret) = misc("<!-- aaa -->").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Misc::from(model::Comment::from(" aaa ")), ret);

        let (rest, ret) = misc("<?aaa?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Misc::from(model::PI::from(("aaa", None))), ret);

        let (rest, ret) = misc(" ").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Misc::from(" "), ret);
    }

    #[test]
    fn test_doctype_decl() {
        let (rest, ret) = doctype_decl("<!DOCTYPE aaa>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationDoc::from((QName::from("aaa"), None, None)),
            ret
        );

        let (rest, ret) = doctype_decl("<!DOCTYPE aaa SYSTEM 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationDoc::from((
                QName::from("aaa"),
                Some(model::ExternalId::from("bbb")),
                None
            )),
            ret
        );

        let (rest, ret) = doctype_decl("<!DOCTYPE aaa [ <!ELEMENT aaa ANY > ]>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationDoc::from((
                QName::from("aaa"),
                None,
                Some(vec![
                    model::InternalSubset::from(" "),
                    model::InternalSubset::from(model::DeclarationMarkup::element(
                        model::DeclarationElement::from((
                            QName::from("aaa"),
                            model::DeclarationContent::Any,
                        ))
                    )),
                    model::InternalSubset::from(" "),
                ])
            )),
            ret
        );
    }

    #[test]
    fn test_int_subset() {
        let (rest, ret) = int_subset("<!ELEMENT aaa ANY >").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            vec![model::InternalSubset::from(
                model::DeclarationMarkup::element(model::DeclarationElement::from((
                    QName::from("aaa"),
                    model::DeclarationContent::Any,
                )))
            )],
            ret
        );

        let (rest, ret) = int_subset("%aaa;").unwrap();
        assert_eq!("", rest);
        assert_eq!(vec![model::InternalSubset::from("aaa")], ret);
    }

    #[test]
    fn test_markup_decl() {
        let (rest, ret) = markup_decl("<!ELEMENT aaa ANY >").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationMarkup::element(model::DeclarationElement::from((
                QName::from("aaa"),
                model::DeclarationContent::Any,
            ))),
            ret
        );

        let (rest, ret) = markup_decl("<!ATTLIST aaa>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationMarkup::attributes(model::DeclarationAtt::from((
                QName::from("aaa"),
                vec![]
            ))),
            ret
        );

        let (rest, ret) = markup_decl("<!ENTITY aaa 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationMarkup::from(model::DeclarationEntity::from(
                model::DeclarationGeneralEntity::from((
                    "aaa",
                    model::DeclarationEntityDef::from(vec![model::EntityValue::text("bbb")]),
                ))
            )),
            ret
        );

        let (rest, ret) = markup_decl("<!NOTATION aaa SYSTEM 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationMarkup::from(model::DeclarationNotation::from((
                "aaa",
                model::DeclarationNotationId::ExternalId(model::ExternalId::from("bbb"))
            ))),
            ret
        );

        let (rest, ret) = markup_decl("<?a?>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationMarkup::from(model::PI::from(("a", None))),
            ret
        );

        let (rest, ret) = markup_decl("<!---->").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationMarkup::from(model::Comment::from("")),
            ret
        );
    }

    #[test]
    fn test_sd_decl() {
        let (rest, ret) = sd_decl(" standalone='yes'").unwrap();
        assert_eq!("", rest);
        assert!(ret);

        let (rest, ret) = sd_decl(" standalone = \"yes\"").unwrap();
        assert_eq!("", rest);
        assert!(ret);

        let (rest, ret) = sd_decl(" standalone='no'").unwrap();
        assert_eq!("", rest);
        assert!(!ret);

        let (rest, ret) = sd_decl(" standalone = \"no\"").unwrap();
        assert_eq!("", rest);
        assert!(!ret);
    }

    #[test]
    fn test_element() {
        let (rest, ret) = element("<a/>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Element::from((QName::from("a"), vec![])), ret);

        let (rest, ret) = element("<a></a>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Element::from((QName::from("a"), vec![]))
                .set_content(model::Content::from((Some(""), vec![]))),
            ret
        );
    }

    #[test]
    fn test_stag() {
        let (rest, ret) = stag("<a>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Element::from((QName::from("a"), vec![])), ret);

        let (rest, ret) = stag("<a b='c'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Element::from((
                QName::from("a"),
                vec![model::Attribute::from((
                    model::AttributeName::QName(QName::Unprefixed("b")),
                    vec![model::AttributeValue::from("c")]
                ))]
            )),
            ret
        );
    }

    #[test]
    fn test_content() {
        let (rest, ret) = content("a").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::Content::from((Some("a"), vec![])), ret);

        let (rest, ret) = content("<a/>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Content::from((
                Some(""),
                vec![model::ContentCell::from((
                    model::Contents::from(model::Element::from((QName::from("a"), vec![]))),
                    Some("")
                )),]
            )),
            ret
        );

        let (rest, ret) = content("a<a/>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Content::from((
                Some("a"),
                vec![model::ContentCell::from((
                    model::Contents::from(model::Element::from((QName::from("a"), vec![]))),
                    Some("")
                )),]
            )),
            ret
        );

        let (rest, ret) = content("<a/>a").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Content::from((
                Some(""),
                vec![model::ContentCell::from((
                    model::Contents::from(model::Element::from((QName::from("a"), vec![]))),
                    Some("a"),
                )),]
            )),
            ret
        );

        let (rest, ret) = content("<a/><b/>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::Content::from((
                Some(""),
                vec![
                    model::ContentCell::from((
                        model::Contents::from(model::Element::from((QName::from("a"), vec![]))),
                        Some(""),
                    )),
                    model::ContentCell::from((
                        model::Contents::from(model::Element::from((QName::from("b"), vec![]))),
                        Some(""),
                    )),
                ]
            )),
            ret
        );
    }

    #[test]
    fn test_element_decl() {
        let (rest, ret) = element_decl("<!ELEMENT aaa EMPTY>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationElement::from((QName::from("aaa"), model::DeclarationContent::Empty)),
            ret
        );
    }

    #[test]
    fn test_content_spec() {
        let (rest, ret) = content_spec("EMPTY").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationContent::Empty, ret);

        let (rest, ret) = content_spec("ANY").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationContent::Any, ret);

        let (rest, ret) = content_spec("(#PCDATA)").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationContent::Mixed(None), ret);

        let (rest, ret) = content_spec("(bbb)").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContent::Children(model::DeclarationContentItem::Seq(
                vec![model::DeclarationContentItem::Name(
                    QName::from("bbb"),
                    None
                )],
                None
            )),
            ret
        );
    }

    #[test]
    fn test_children() {
        let (rest, ret) = children("(a)").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Seq(
                vec![model::DeclarationContentItem::Name(QName::from("a"), None),],
                None
            ),
            ret
        );

        let (rest, ret) = children("(a | b)?").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Choice(
                vec![
                    model::DeclarationContentItem::Name(QName::from("a"), None),
                    model::DeclarationContentItem::Name(QName::from("b"), None),
                ],
                Some("?")
            ),
            ret
        );

        let (rest, ret) = children("(a , b)*").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Seq(
                vec![
                    model::DeclarationContentItem::Name(QName::from("a"), None),
                    model::DeclarationContentItem::Name(QName::from("b"), None),
                ],
                Some("*")
            ),
            ret
        );

        let (rest, ret) = children("(a | b | c)+").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Choice(
                vec![
                    model::DeclarationContentItem::Name(QName::from("a"), None),
                    model::DeclarationContentItem::Name(QName::from("b"), None),
                    model::DeclarationContentItem::Name(QName::from("c"), None),
                ],
                Some("+")
            ),
            ret
        );
    }

    #[test]
    fn test_cp() {
        let (rest, ret) = cp("a").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Name(QName::from("a"), None),
            ret
        );

        let (rest, ret) = cp("(a | b)?").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Choice(
                vec![
                    model::DeclarationContentItem::Name(QName::from("a"), None),
                    model::DeclarationContentItem::Name(QName::from("b"), None),
                ],
                Some("?")
            ),
            ret
        );

        let (rest, ret) = cp("(a , b)*").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Seq(
                vec![
                    model::DeclarationContentItem::Name(QName::from("a"), None),
                    model::DeclarationContentItem::Name(QName::from("b"), None),
                ],
                Some("*")
            ),
            ret
        );

        let (rest, ret) = cp("a+").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationContentItem::Name(QName::from("a"), Some("+")),
            ret
        );
    }

    #[test]
    fn test_mixed() {
        let (rest, ret) = mixed("(#PCDATA)").unwrap();
        assert_eq!("", rest);
        assert_eq!(None, ret);

        let (rest, ret) = mixed("(#PCDATA)*").unwrap();
        assert_eq!("", rest);
        assert_eq!(Some(vec![]), ret);

        let (rest, ret) = mixed("(#PCDATA | a)*").unwrap();
        assert_eq!("", rest);
        assert_eq!(Some(vec![QName::from("a")]), ret);

        let (rest, ret) = mixed("(#PCDATA | a | b)*").unwrap();
        assert_eq!("", rest);
        assert_eq!(Some(vec![QName::from("a"), QName::from("b")]), ret);
    }

    #[test]
    fn test_attlist_decl() {
        let (rest, ret) = attlist_decl("<!ATTLIST a>").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAtt::from((QName::from("a"), vec![])), ret);

        let (rest, ret) = attlist_decl("<!ATTLIST a b ID #REQUIRED>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationAtt::from((
                QName::from("a"),
                vec![model::DeclarationAttDef::from((
                    model::DeclarationAttName::Attr(QName::from("b")),
                    model::DeclarationAttType::Id,
                    model::DeclarationAttDefault::Required
                )),]
            )),
            ret
        );
    }

    #[test]
    fn test_att_type() {
        let (rest, ret) = att_type("NOTATION (a)").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAttType::Notation(vec!["a"]), ret);

        let (rest, ret) = att_type("NOTATION (a | b)").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAttType::Notation(vec!["a", "b"]), ret);

        let (rest, ret) = att_type("(a)").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAttType::Enumeration(vec!["a"]), ret);

        let (rest, ret) = att_type("(a | b)").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAttType::Enumeration(vec!["a", "b"]), ret);

        let (rest, ret) = att_type("CDATA").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAttType::Cdata, ret);
    }

    #[test]
    fn test_default_decl() {
        let (rest, ret) = default_decl("#REQUIRED").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAttDefault::Required, ret);

        let (rest, ret) = default_decl("#IMPLIED").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::DeclarationAttDefault::Implied, ret);

        let (rest, ret) = default_decl("'a'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationAttDefault::Value(None, vec![model::AttributeValue::from("a")]),
            ret
        );

        let (rest, ret) = default_decl("#FIXED 'a'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationAttDefault::Value(
                Some("#FIXED"),
                vec![model::AttributeValue::from("a")]
            ),
            ret
        );
    }

    #[test]
    fn test_entity_decl() {
        let (rest, ret) = entity_decl("<!ENTITY aaa 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationEntity::from(model::DeclarationGeneralEntity::from((
                "aaa",
                model::DeclarationEntityDef::from(vec![model::EntityValue::text("bbb")]),
            ))),
            ret
        );

        let (rest, ret) = entity_decl("<!ENTITY % aaa 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationEntity::from(model::DeclarationParameterEntity::from((
                "aaa",
                model::DeclarationPeDef::from(vec![model::EntityValue::text("bbb")]),
            )),),
            ret
        );
    }

    #[test]
    fn test_ge_decl() {
        let (rest, ret) = ge_decl("<!ENTITY aaa 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationGeneralEntity::from((
                "aaa",
                model::DeclarationEntityDef::from(vec![model::EntityValue::text("bbb")]),
            )),
            ret
        );
    }

    #[test]
    fn test_pe_decl() {
        let (rest, ret) = pe_decl("<!ENTITY % aaa 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationParameterEntity::from((
                "aaa",
                model::DeclarationPeDef::from(vec![model::EntityValue::text("bbb")]),
            )),
            ret
        );
    }

    #[test]
    fn test_entity_def() {
        let (rest, ret) = entity_def("'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationEntityDef::from(vec![model::EntityValue::text("aaa")]),
            ret
        );

        let (rest, ret) = entity_def("SYSTEM 'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationEntityDef::from((model::ExternalId::from("aaa"), None)),
            ret
        );

        let (rest, ret) = entity_def("SYSTEM 'aaa' NDATA bbb").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationEntityDef::from((model::ExternalId::from("aaa"), Some("bbb"))),
            ret
        );
    }

    #[test]
    fn test_pe_def() {
        let (rest, ret) = pe_def("'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationPeDef::from(vec![model::EntityValue::text("aaa")]),
            ret
        );

        let (rest, ret) = pe_def("SYSTEM 'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!(
            model::DeclarationPeDef::from(model::ExternalId::from("aaa")),
            ret
        );
    }

    #[test]
    fn test_external_id() {
        let (rest, ret) = external_id("SYSTEM 'aaa'").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::ExternalId::from("aaa"), ret);

        let (rest, ret) = external_id("PUBLIC 'aaa' 'bbb'").unwrap();
        assert_eq!("", rest);
        assert_eq!(model::ExternalId::from(("aaa", "bbb")), ret);
    }

    #[test]
    fn test_encoding_decl() {
        let (rest, ret) = encoding_decl(" encoding='utf-8'").unwrap();
        assert_eq!("", rest);
        assert_eq!("utf-8", ret);

        let (rest, ret) = encoding_decl(" encoding = \"sjis\"").unwrap();
        assert_eq!("", rest);
        assert_eq!("sjis", ret);
    }

    #[test]
    fn test_enc_name() {
        let (rest, ret) = enc_name("utf-8").unwrap();
        assert_eq!("", rest);
        assert_eq!("utf-8", ret);
    }

    #[test]
    fn test_notation_decl() {
        let (rest, ret) = notation_decl("<!NOTATION aaa SYSTEM 'bbb'>").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret.name);
        assert_eq!(
            model::DeclarationNotationId::ExternalId(model::ExternalId::from("bbb")),
            ret.id
        );

        let (rest, ret) = notation_decl("<!NOTATION aaa PUBLIC 'bbb' 'ccc'>").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret.name);
        assert_eq!(
            model::DeclarationNotationId::ExternalId(model::ExternalId::from(("bbb", "ccc"))),
            ret.id
        );

        let (rest, ret) = notation_decl("<!NOTATION aaa PUBLIC 'ccc'>").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret.name);
        assert_eq!(model::DeclarationNotationId::PublicId("ccc"), ret.id);
    }
}

// -----------------------------------------------------------------------------------------------
