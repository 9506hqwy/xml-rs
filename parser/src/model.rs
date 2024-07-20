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
pub struct DeclarationAtt<'a> {
    pub name: QName<'a>,
    pub defs: Vec<DeclarationAttDef<'a>>,
}

impl<'a> From<(QName<'a>, Vec<DeclarationAttDef<'a>>)> for DeclarationAtt<'a> {
    fn from(value: (QName<'a>, Vec<DeclarationAttDef<'a>>)) -> Self {
        let (name, defs) = value;
        DeclarationAtt { name, defs }
    }
}
// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeclarationAttDef<'a> {
    pub name: DeclarationAttName<'a>,
    pub ty: DeclarationAttType<'a>,
    pub value: DeclarationAttDefault<'a>,
}

impl<'a>
    From<(
        DeclarationAttName<'a>,
        DeclarationAttType<'a>,
        DeclarationAttDefault<'a>,
    )> for DeclarationAttDef<'a>
{
    fn from(
        value: (
            DeclarationAttName<'a>,
            DeclarationAttType<'a>,
            DeclarationAttDefault<'a>,
        ),
    ) -> Self {
        let (name, ty, value) = value;
        DeclarationAttDef { name, ty, value }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum DeclarationAttDefault<'a> {
    #[default]
    Required,
    Implied,
    Value(Option<&'a str>, Vec<AttributeValue<'a>>),
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationAttName<'a> {
    Attr(QName<'a>),
    Namsspace(AttributeName<'a>),
}

impl<'a> Default for DeclarationAttName<'a> {
    fn default() -> Self {
        DeclarationAttName::Attr(QName::default())
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum DeclarationAttType<'a> {
    #[default]
    Cdata,
    Entities,
    Entity,
    Id,
    IdRef,
    IdRefs,
    NmToken,
    NmTokens,
    Notation(Vec<&'a str>),
    Enumeration(Vec<&'a str>),
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum DeclarationContent<'a> {
    #[default]
    Empty,
    Any,
    Mixed(Option<Vec<QName<'a>>>),
    Children(DeclarationContentItem<'a>),
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationContentItem<'a> {
    Name(QName<'a>, Option<&'a str>),
    Choice(Vec<DeclarationContentItem<'a>>, Option<&'a str>),
    Seq(Vec<DeclarationContentItem<'a>>, Option<&'a str>),
}

impl<'a> Default for DeclarationContentItem<'a> {
    fn default() -> Self {
        DeclarationContentItem::Name(QName::default(), None)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeclarationDoc<'a> {
    pub name: QName<'a>,
    pub external_id: Option<ExternalId<'a>>,
    pub internal_subset: Vec<InternalSubset<'a>>,
}

impl<'a>
    From<(
        QName<'a>,
        Option<ExternalId<'a>>,
        Option<Vec<InternalSubset<'a>>>,
    )> for DeclarationDoc<'a>
{
    fn from(
        value: (
            QName<'a>,
            Option<ExternalId<'a>>,
            Option<Vec<InternalSubset<'a>>>,
        ),
    ) -> Self {
        let (name, external_id, int_subsets) = value;
        let internal_subset = int_subsets.unwrap_or_default();
        DeclarationDoc {
            name,
            external_id,
            internal_subset,
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationEntity<'a> {
    GeneralEntity(DeclarationGeneralEntity<'a>),
    ParameterEntity(DeclarationParameterEntity<'a>),
}

impl<'a> Default for DeclarationEntity<'a> {
    fn default() -> Self {
        DeclarationEntity::from(DeclarationGeneralEntity::default())
    }
}

impl<'a> From<DeclarationGeneralEntity<'a>> for DeclarationEntity<'a> {
    fn from(value: DeclarationGeneralEntity<'a>) -> Self {
        DeclarationEntity::GeneralEntity(value)
    }
}

impl<'a> From<DeclarationParameterEntity<'a>> for DeclarationEntity<'a> {
    fn from(value: DeclarationParameterEntity<'a>) -> Self {
        DeclarationEntity::ParameterEntity(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeclarationElement<'a> {
    pub name: QName<'a>,
    pub content: DeclarationContent<'a>,
}

impl<'a> From<(QName<'a>, DeclarationContent<'a>)> for DeclarationElement<'a> {
    fn from(value: (QName<'a>, DeclarationContent<'a>)) -> Self {
        let (name, content) = value;
        DeclarationElement { name, content }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationEntityDef<'a> {
    EntityValue(Vec<EntityValue<'a>>),
    ExternalId(ExternalId<'a>, Option<&'a str>),
}

impl<'a> Default for DeclarationEntityDef<'a> {
    fn default() -> Self {
        DeclarationEntityDef::from(vec![])
    }
}

impl<'a> From<Vec<EntityValue<'a>>> for DeclarationEntityDef<'a> {
    fn from(value: Vec<EntityValue<'a>>) -> Self {
        DeclarationEntityDef::EntityValue(value)
    }
}

impl<'a> From<(ExternalId<'a>, Option<&'a str>)> for DeclarationEntityDef<'a> {
    fn from(value: (ExternalId<'a>, Option<&'a str>)) -> Self {
        let (external_id, ndata) = value;
        DeclarationEntityDef::ExternalId(external_id, ndata)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeclarationGeneralEntity<'a> {
    pub name: &'a str,
    pub def: DeclarationEntityDef<'a>,
}

impl<'a> From<(&'a str, DeclarationEntityDef<'a>)> for DeclarationGeneralEntity<'a> {
    fn from(value: (&'a str, DeclarationEntityDef<'a>)) -> Self {
        let (name, def) = value;
        DeclarationGeneralEntity { name, def }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationMarkup<'a> {
    Element(DeclarationElement<'a>),
    Attributes(DeclarationAtt<'a>),
    Entity(DeclarationEntity<'a>),
    Notation(DeclarationNotation<'a>),
    PI(PI<'a>),
    Commnect(Comment<'a>),
}

impl<'a> Default for DeclarationMarkup<'a> {
    fn default() -> Self {
        DeclarationMarkup::element(DeclarationElement::default())
    }
}

impl<'a> From<DeclarationEntity<'a>> for DeclarationMarkup<'a> {
    fn from(value: DeclarationEntity<'a>) -> Self {
        DeclarationMarkup::Entity(value)
    }
}

impl<'a> From<DeclarationNotation<'a>> for DeclarationMarkup<'a> {
    fn from(value: DeclarationNotation<'a>) -> Self {
        DeclarationMarkup::Notation(value)
    }
}

impl<'a> From<PI<'a>> for DeclarationMarkup<'a> {
    fn from(value: PI<'a>) -> Self {
        DeclarationMarkup::PI(value)
    }
}

impl<'a> From<Comment<'a>> for DeclarationMarkup<'a> {
    fn from(value: Comment<'a>) -> Self {
        DeclarationMarkup::Commnect(value)
    }
}

impl<'a> DeclarationMarkup<'a> {
    pub fn element(value: DeclarationElement<'a>) -> DeclarationMarkup<'a> {
        DeclarationMarkup::Element(value)
    }

    pub fn attributes(value: DeclarationAtt<'a>) -> DeclarationMarkup<'a> {
        DeclarationMarkup::Attributes(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeclarationNotation<'a> {
    pub name: &'a str,
    pub id: DeclarationNotationId<'a>,
}

impl<'a> From<(&'a str, DeclarationNotationId<'a>)> for DeclarationNotation<'a> {
    fn from(value: (&'a str, DeclarationNotationId<'a>)) -> Self {
        let (name, id) = value;
        DeclarationNotation { name, id }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationNotationId<'a> {
    ExternalId(ExternalId<'a>),
    PublicId(&'a str),
}

impl<'a> Default for DeclarationNotationId<'a> {
    fn default() -> Self {
        DeclarationNotationId::from(ExternalId::default())
    }
}

impl<'a> From<ExternalId<'a>> for DeclarationNotationId<'a> {
    fn from(value: ExternalId<'a>) -> Self {
        DeclarationNotationId::ExternalId(value)
    }
}

impl<'a> From<&'a str> for DeclarationNotationId<'a> {
    fn from(value: &'a str) -> Self {
        DeclarationNotationId::PublicId(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeclarationParameterEntity<'a> {
    pub name: &'a str,
    pub def: DeclarationPeDef<'a>,
}

impl<'a> From<(&'a str, DeclarationPeDef<'a>)> for DeclarationParameterEntity<'a> {
    fn from(value: (&'a str, DeclarationPeDef<'a>)) -> Self {
        let (name, def) = value;
        DeclarationParameterEntity { name, def }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationPeDef<'a> {
    EntityValue(Vec<EntityValue<'a>>),
    ExternalId(ExternalId<'a>),
}

impl<'a> Default for DeclarationPeDef<'a> {
    fn default() -> Self {
        DeclarationPeDef::from(vec![])
    }
}

impl<'a> From<Vec<EntityValue<'a>>> for DeclarationPeDef<'a> {
    fn from(value: Vec<EntityValue<'a>>) -> Self {
        DeclarationPeDef::EntityValue(value)
    }
}

impl<'a> From<ExternalId<'a>> for DeclarationPeDef<'a> {
    fn from(value: ExternalId<'a>) -> Self {
        DeclarationPeDef::ExternalId(value)
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
pub enum ExternalId<'a> {
    System(&'a str),
    Public(&'a str, &'a str),
}

impl<'a> Default for ExternalId<'a> {
    fn default() -> Self {
        ExternalId::System("")
    }
}

impl<'a> From<&'a str> for ExternalId<'a> {
    fn from(value: &'a str) -> Self {
        ExternalId::System(value)
    }
}

impl<'a> From<(&'a str, &'a str)> for ExternalId<'a> {
    fn from(value: (&'a str, &'a str)) -> Self {
        let (p, s) = value;
        ExternalId::Public(p, s)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum InternalSubset<'a> {
    Markup(DeclarationMarkup<'a>),
    ParameterEntityReference(&'a str),
}

impl<'a> Default for InternalSubset<'a> {
    fn default() -> Self {
        InternalSubset::from(DeclarationMarkup::default())
    }
}

impl<'a> From<DeclarationMarkup<'a>> for InternalSubset<'a> {
    fn from(value: DeclarationMarkup<'a>) -> Self {
        InternalSubset::Markup(value)
    }
}

impl<'a> From<&'a str> for InternalSubset<'a> {
    fn from(value: &'a str) -> Self {
        InternalSubset::ParameterEntityReference(value)
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
    pub declaration_doc: Option<DeclarationDoc<'a>>,
    pub tails: Vec<Misc<'a>>,
}

#[allow(clippy::type_complexity)]
impl<'a>
    From<(
        Option<DeclarationXml<'a>>,
        Vec<Misc<'a>>,
        Option<(DeclarationDoc<'a>, Vec<Misc<'a>>)>,
    )> for Prolog<'a>
{
    fn from(
        value: (
            Option<DeclarationXml<'a>>,
            Vec<Misc<'a>>,
            Option<(DeclarationDoc<'a>, Vec<Misc<'a>>)>,
        ),
    ) -> Self {
        let (xml_decl, heads, tail) = value;
        let declaration_doc = tail.as_ref().map(|t| t.0.clone());
        let tails = tail.map(|t| t.1).unwrap_or_default();
        Prolog {
            declaration_xml: xml_decl,
            heads,
            declaration_doc,
            tails,
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
