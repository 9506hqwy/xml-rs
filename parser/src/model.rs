// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct Attribute<'a> {
    pub name: AttributeName<'a>,
    pub value: Vec<AttributeValue<'a>>,
}

impl<'a> Attribute<'a> {
    pub fn new(value: (AttributeName<'a>, Vec<AttributeValue<'a>>)) -> Self {
        let (name, value) = value;
        Attribute { name, value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum AttributeName<'a> {
    DefaultNamespace,
    Namespace(&'a str),
    QName(QName<'a>),
}

impl<'a> AttributeName<'a> {
    pub fn default() -> Self {
        AttributeName::DefaultNamespace
    }

    pub fn namespace(value: &'a str) -> Self {
        AttributeName::Namespace(value)
    }

    pub fn qname(value: QName<'a>) -> Self {
        AttributeName::QName(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum AttributeValue<'a> {
    Reference(Reference<'a>),
    Text(&'a str),
}

impl<'a> AttributeValue<'a> {
    pub fn reference(value: Reference<'a>) -> Self {
        AttributeValue::Reference(value)
    }

    pub fn text(value: &'a str) -> Self {
        AttributeValue::Text(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct CData<'a> {
    pub value: &'a str,
}

impl<'a> CData<'a> {
    pub fn new(value: &'a str) -> Self {
        CData { value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct Comment<'a> {
    pub value: &'a str,
}

impl<'a> Comment<'a> {
    pub fn new(value: &'a str) -> Self {
        Comment { value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct Content<'a> {
    pub head: Option<&'a str>,
    pub children: Vec<ContentCell<'a>>,
}

impl<'a> Content<'a> {
    pub fn new(head: Option<&'a str>, children: Vec<ContentCell<'a>>) -> Self {
        Content { head, children }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct ContentCell<'a> {
    pub child: Contents<'a>,
    pub tail: Option<&'a str>,
}

impl<'a> ContentCell<'a> {
    pub fn new(value: (Contents<'a>, Option<&'a str>)) -> Self {
        let (child, tail) = value;
        ContentCell { child, tail }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum Contents<'a> {
    Element(Element<'a>),
    Reference(Reference<'a>),
    CData(CData<'a>),
    PI(PI<'a>),
    Comment(Comment<'a>),
}

impl<'a> Contents<'a> {
    pub fn element(value: Element<'a>) -> Self {
        Contents::Element(value)
    }

    pub fn reference(value: Reference<'a>) -> Self {
        Contents::Reference(value)
    }

    pub fn cdata(value: CData<'a>) -> Self {
        Contents::CData(value)
    }

    pub fn pi(value: PI<'a>) -> Self {
        Contents::PI(value)
    }

    pub fn comment(value: Comment<'a>) -> Self {
        Contents::Comment(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct DeclarationXml<'a> {
    pub version: &'a str,
    pub encoding: Option<&'a str>,
    pub standalone: Option<bool>,
}

impl<'a> DeclarationXml<'a> {
    pub fn new(value: (&'a str, Option<&'a str>, Option<bool>)) -> Self {
        let (version, encoding, standalone) = value;
        DeclarationXml {
            version,
            encoding,
            standalone,
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct Document<'a> {
    pub prolog: Prolog<'a>,
    pub element: Element<'a>,
    pub miscs: Vec<Misc<'a>>,
}

impl<'a> Document<'a> {
    pub fn new(value: (Prolog<'a>, Element<'a>, Vec<Misc<'a>>)) -> Self {
        let (prolog, element, miscs) = value;
        Document {
            prolog,
            element,
            miscs,
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct Element<'a> {
    pub name: QName<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub content: Option<Content<'a>>,
}

impl<'a> Element<'a> {
    pub fn from(value: (QName<'a>, Vec<Attribute<'a>>)) -> Self {
        let (name, attributes) = value;
        Element {
            name,
            attributes,
            content: None,
        }
    }

    pub fn set_content(mut self, content: Content<'a>) -> Self {
        self.content = Some(content);
        self
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum EntityValue<'a> {
    Text(&'a str),
    ParameterEntityReference(&'a str),
    Reference(Reference<'a>),
}

impl<'a> EntityValue<'a> {
    pub fn text(value: &'a str) -> Self {
        EntityValue::Text(value)
    }

    pub fn pe_reference(value: &'a str) -> Self {
        EntityValue::ParameterEntityReference(value)
    }

    pub fn reference(value: Reference<'a>) -> Self {
        EntityValue::Reference(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum Misc<'a> {
    Comment(Comment<'a>),
    PI(PI<'a>),
    Whitespace(&'a str),
}

impl<'a> Misc<'a> {
    pub fn comment(value: Comment<'a>) -> Self {
        Misc::Comment(value)
    }

    pub fn pi(value: PI<'a>) -> Self {
        Misc::PI(value)
    }

    pub fn whitespace(value: &'a str) -> Self {
        Misc::Whitespace(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct PI<'a> {
    pub target: &'a str,
    pub value: Option<&'a str>,
}

impl<'a> PI<'a> {
    pub fn new(value: (&'a str, Option<&'a str>)) -> Self {
        let (target, value) = value;
        PI { target, value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct PrefixedName<'a> {
    pub prefix: &'a str,
    pub local_part: &'a str,
}

impl<'a> PrefixedName<'a> {
    pub fn new(value: (&'a str, &'a str)) -> Self {
        let (prefix, local_part) = value;
        PrefixedName { prefix, local_part }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct Prolog<'a> {
    pub declaration_xml: Option<DeclarationXml<'a>>,
    pub heads: Vec<Misc<'a>>,
    pub declaration_doc: Option<&'a str>,
    pub tails: Vec<Misc<'a>>,
}

#[allow(clippy::type_complexity)]
impl<'a> Prolog<'a> {
    pub fn new(
        value: (
            Option<DeclarationXml<'a>>,
            Vec<Misc<'a>>,
            Option<(&'a str, Vec<Misc<'a>>)>,
        ),
    ) -> Self {
        let (xml_decl, heads, tail) = value;
        Prolog {
            declaration_xml: xml_decl,
            heads,
            declaration_doc: tail.as_ref().map(|t| t.0),
            tails: tail.map(|t| t.1).unwrap_or_default(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum QName<'a> {
    Prefixed(PrefixedName<'a>),
    Unprefixed(&'a str),
}

impl<'a> QName<'a> {
    pub fn prefixed(value: PrefixedName<'a>) -> Self {
        QName::Prefixed(value)
    }

    pub fn unprefixed(value: &'a str) -> Self {
        QName::Unprefixed(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum Reference<'a> {
    Character(&'a str, u32),
    Entity(&'a str),
}

impl<'a> Reference<'a> {
    pub fn digit(value: &'a str) -> Self {
        Reference::Character(value, 10)
    }

    pub fn entity(value: &'a str) -> Self {
        Reference::Entity(value)
    }

    pub fn hex(value: &'a str) -> Self {
        Reference::Character(value, 16)
    }
}

// -----------------------------------------------------------------------------------------------
