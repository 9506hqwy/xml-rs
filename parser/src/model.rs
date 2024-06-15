use xml_nom::model::QName;

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attribute<'a> {
    pub name: AttributeName<'a>,
    pub value: Vec<AttributeValue<'a>>,
}

impl<'a> From<(AttributeName<'a>, Vec<AttributeValue<'a>>)> for Attribute<'a> {
    fn from(value: (AttributeName<'a>, Vec<AttributeValue<'a>>)) -> Self {
        let (name, value) = value;
        Attribute { name, value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AttributeName<'a> {
    #[default]
    DefaultNamespace,
    Namespace(&'a str),
    QName(QName<'a>),
}

impl<'a> From<&'a str> for AttributeName<'a> {
    fn from(value: &'a str) -> Self {
        AttributeName::Namespace(value)
    }
}

impl<'a> From<QName<'a>> for AttributeName<'a> {
    fn from(value: QName<'a>) -> Self {
        AttributeName::QName(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue<'a> {
    Reference(Reference<'a>),
    Text(&'a str),
}

impl<'a> Default for AttributeValue<'a> {
    fn default() -> Self {
        AttributeValue::from("")
    }
}

impl<'a> From<Reference<'a>> for AttributeValue<'a> {
    fn from(value: Reference<'a>) -> Self {
        AttributeValue::Reference(value)
    }
}

impl<'a> From<&'a str> for AttributeValue<'a> {
    fn from(value: &'a str) -> Self {
        AttributeValue::Text(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CData<'a> {
    pub value: &'a str,
}

impl<'a> From<&'a str> for CData<'a> {
    fn from(value: &'a str) -> Self {
        CData { value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Comment<'a> {
    pub value: &'a str,
}

impl<'a> From<&'a str> for Comment<'a> {
    fn from(value: &'a str) -> Self {
        Comment { value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Content<'a> {
    pub head: Option<&'a str>,
    pub children: Vec<ContentCell<'a>>,
}

impl<'a> From<(Option<&'a str>, Vec<ContentCell<'a>>)> for Content<'a> {
    fn from(value: (Option<&'a str>, Vec<ContentCell<'a>>)) -> Self {
        let (head, children) = value;
        Content { head, children }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContentCell<'a> {
    pub child: Contents<'a>,
    pub tail: Option<&'a str>,
}

impl<'a> From<(Contents<'a>, Option<&'a str>)> for ContentCell<'a> {
    fn from(value: (Contents<'a>, Option<&'a str>)) -> Self {
        let (child, tail) = value;
        ContentCell { child, tail }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Contents<'a> {
    Element(Element<'a>),
    Reference(Reference<'a>),
    CData(CData<'a>),
    PI(PI<'a>),
    Comment(Comment<'a>),
}

impl<'a> Default for Contents<'a> {
    fn default() -> Self {
        Contents::from(Comment::default())
    }
}

impl<'a> From<Element<'a>> for Contents<'a> {
    fn from(value: Element<'a>) -> Self {
        Contents::Element(value)
    }
}

impl<'a> From<Reference<'a>> for Contents<'a> {
    fn from(value: Reference<'a>) -> Self {
        Contents::Reference(value)
    }
}

impl<'a> From<CData<'a>> for Contents<'a> {
    fn from(value: CData<'a>) -> Self {
        Contents::CData(value)
    }
}

impl<'a> From<PI<'a>> for Contents<'a> {
    fn from(value: PI<'a>) -> Self {
        Contents::PI(value)
    }
}

impl<'a> From<Comment<'a>> for Contents<'a> {
    fn from(value: Comment<'a>) -> Self {
        Contents::Comment(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeclarationXml<'a> {
    pub version: &'a str,
    pub encoding: Option<&'a str>,
    pub standalone: Option<bool>,
}

impl<'a> From<(&'a str, Option<&'a str>, Option<bool>)> for DeclarationXml<'a> {
    fn from(value: (&'a str, Option<&'a str>, Option<bool>)) -> Self {
        let (version, encoding, standalone) = value;
        DeclarationXml {
            version,
            encoding,
            standalone,
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document<'a> {
    pub prolog: Prolog<'a>,
    pub element: Element<'a>,
    pub miscs: Vec<Misc<'a>>,
}

impl<'a> From<(Prolog<'a>, Element<'a>, Vec<Misc<'a>>)> for Document<'a> {
    fn from(value: (Prolog<'a>, Element<'a>, Vec<Misc<'a>>)) -> Self {
        let (prolog, element, miscs) = value;
        Document {
            prolog,
            element,
            miscs,
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Element<'a> {
    pub name: QName<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub content: Option<Content<'a>>,
}

impl<'a> From<(QName<'a>, Vec<Attribute<'a>>)> for Element<'a> {
    fn from(value: (QName<'a>, Vec<Attribute<'a>>)) -> Self {
        let (name, attributes) = value;
        Element {
            name,
            attributes,
            content: None,
        }
    }
}

impl<'a> Element<'a> {
    pub fn set_content(mut self, content: Content<'a>) -> Self {
        self.content = Some(content);
        self
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum EntityValue<'a> {
    Text(&'a str),
    ParameterEntityReference(&'a str),
    Reference(Reference<'a>),
}

impl<'a> Default for EntityValue<'a> {
    fn default() -> Self {
        EntityValue::Text("")
    }
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

#[derive(Clone, Debug, PartialEq)]
pub enum Misc<'a> {
    Comment(Comment<'a>),
    PI(PI<'a>),
    Whitespace(&'a str),
}

impl<'a> Default for Misc<'a> {
    fn default() -> Self {
        Misc::from(Comment::default())
    }
}

impl<'a> From<Comment<'a>> for Misc<'a> {
    fn from(value: Comment<'a>) -> Self {
        Misc::Comment(value)
    }
}

impl<'a> From<PI<'a>> for Misc<'a> {
    fn from(value: PI<'a>) -> Self {
        Misc::PI(value)
    }
}

impl<'a> From<&'a str> for Misc<'a> {
    fn from(value: &'a str) -> Self {
        Misc::Whitespace(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PI<'a> {
    pub target: &'a str,
    pub value: Option<&'a str>,
}

impl<'a> From<(&'a str, Option<&'a str>)> for PI<'a> {
    fn from(value: (&'a str, Option<&'a str>)) -> Self {
        let (target, value) = value;
        PI { target, value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Prolog<'a> {
    pub declaration_xml: Option<DeclarationXml<'a>>,
    pub heads: Vec<Misc<'a>>,
    pub declaration_doc: Option<&'a str>,
    pub tails: Vec<Misc<'a>>,
}

#[allow(clippy::type_complexity)]
impl<'a>
    From<(
        Option<DeclarationXml<'a>>,
        Vec<Misc<'a>>,
        Option<(&'a str, Vec<Misc<'a>>)>,
    )> for Prolog<'a>
{
    fn from(
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

#[derive(Clone, Debug, PartialEq)]
pub enum Reference<'a> {
    Character(&'a str, u32),
    Entity(&'a str),
}

impl<'a> Default for Reference<'a> {
    fn default() -> Self {
        Reference::Character("", 0)
    }
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
