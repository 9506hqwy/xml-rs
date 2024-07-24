pub mod error;

use std::cell::RefCell;
use std::iter::Iterator;
use std::ops::Deref;
use std::rc::Rc;
use xml_parser::model as parser;

// TODO: Base URI is always empty string.

// -----------------------------------------------------------------------------------------------

pub type XmlNode<T> = Rc<RefCell<T>>;

// -----------------------------------------------------------------------------------------------

pub trait HasQName {
    fn local_name(&self) -> &str;

    fn prefix(&self) -> Option<&str>;

    fn qname(&self) -> xml_nom::model::QName<'_> {
        if let Some(prefix) = self.prefix() {
            xml_nom::model::QName::Prefixed(xml_nom::model::PrefixedName {
                prefix,
                local_part: self.local_name(),
            })
        } else {
            xml_nom::model::QName::Unprefixed(self.local_name())
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub trait Attribute: HasQName {
    fn namespace_name(&self) -> error::Result<Option<NamespaceUri>>;

    fn normalized_value(&self) -> error::Result<String>;

    fn specified(&self) -> bool;

    fn attribute_type(&self) -> Value<Option<XmlDeclarationAttType>>;

    fn references(&self) -> Value<Option<OrderedList<XmlItem>>>;

    fn owner_element(&self) -> error::Result<XmlNode<XmlElement>>;
}

// -----------------------------------------------------------------------------------------------

pub trait Character {
    fn character_code(&self) -> &str;

    fn element_content_whitespace(&self) -> Value<Option<bool>>;

    fn parent(&self) -> error::Result<XmlNode<XmlElement>>;
}

// -----------------------------------------------------------------------------------------------

pub trait Comment {
    fn comment(&self) -> &str;

    fn parent(&self) -> error::Result<XmlItem>;
}

// -----------------------------------------------------------------------------------------------

pub trait Document {
    fn children(&self) -> OrderedList<XmlItem>;

    fn document_element(&self) -> error::Result<XmlNode<XmlElement>>;

    fn notations(&self) -> Option<UnorderedSet<XmlNode<XmlNotation>>>;

    fn unparsed_entities(&self) -> UnorderedSet<XmlNode<XmlUnparsedEntity>>;

    fn base_uri(&self) -> &str;

    fn character_encoding_scheme(&self) -> &str;

    fn standalone(&self) -> Option<bool>;

    fn version(&self) -> Option<&str>;

    fn all_declarations_processed(&self) -> bool;
}

// -----------------------------------------------------------------------------------------------

pub trait DocumentTypeDeclaration {
    fn system_identifier(&self) -> Option<&str>;

    fn public_identifier(&self) -> Option<&str>;

    fn children(&self) -> OrderedList<XmlNode<XmlProcessingInstruction>>;

    fn parent(&self) -> XmlNode<XmlDocument>;
}

// -----------------------------------------------------------------------------------------------

pub trait Element: HasQName {
    fn namespace_name(&self) -> error::Result<Option<NamespaceUri>>;

    fn children(&self) -> OrderedList<XmlItem>;

    fn attributes(&self) -> UnorderedSet<XmlNode<XmlAttribute>>;

    fn namespace_attributes(&self) -> UnorderedSet<XmlNode<XmlAttribute>>;

    fn in_scope_namespace(&self) -> error::Result<UnorderedSet<XmlNode<XmlNamespace>>>;

    fn base_uri(&self) -> &str;

    fn parent(&self) -> error::Result<XmlItem>;
}

// -----------------------------------------------------------------------------------------------

pub trait Namespace {
    fn prefix(&self) -> Option<&str>;

    fn namespace_name(&self) -> &str;
}

// -----------------------------------------------------------------------------------------------

pub trait Notation {
    fn name(&self) -> &str;

    fn system_identifier(&self) -> Option<&str>;

    fn public_identifier(&self) -> Option<&str>;

    fn declaration_base_uri(&self) -> &str;
}

// -----------------------------------------------------------------------------------------------

pub trait ProcessingInstruction {
    fn target(&self) -> &str;

    fn content(&self) -> &str;

    fn base_uri(&self) -> &str;

    fn notation(&self) -> Value<Option<XmlNode<XmlNotation>>>;

    fn parent(&self) -> XmlItem;
}

// -----------------------------------------------------------------------------------------------

pub trait UnexpandedEntityReference {
    fn name(&self) -> &str;

    fn system_identifier(&self) -> Value<Option<&str>>;

    fn public_identifier(&self) -> Value<Option<&str>>;

    fn declaration_base_uri(&self) -> &str;

    fn parent(&self) -> XmlNode<XmlElement>;
}

// -----------------------------------------------------------------------------------------------

pub trait UnparsedEntity {
    fn name(&self) -> &str;

    fn system_identifier(&self) -> &str;

    fn public_identifier(&self) -> Option<&str>;

    fn declaration_base_uri(&self) -> &str;

    fn notation_name(&self) -> &str;

    fn notation(&self) -> Value<Option<XmlNode<XmlNotation>>>;
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlAttribute {
    local_name: String,
    prefix: Option<String>,
    values: Vec<XmlAttributeValue>,
    parent: Option<XmlNode<XmlElement>>,
    owner: XmlNode<XmlDocument>,
}

impl HasQName for XmlAttribute {
    fn local_name(&self) -> &str {
        self.local_name.as_str()
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

impl Attribute for XmlAttribute {
    fn namespace_name(&self) -> error::Result<Option<NamespaceUri>> {
        if self.namespace() {
            return Ok(Some(NamespaceUri::from("http://www.w3.org/2000/xmlns/")));
        }

        if let Some(prefix) = self.prefix.as_deref() {
            self.parent
                .as_ref()
                .ok_or(error::Error::IsolatedNode)?
                .borrow()
                .find_nameapce_uri(prefix)
        } else {
            Ok(None)
        }
    }

    fn normalized_value(&self) -> error::Result<String> {
        let mut normalized = String::new();

        for value in self.values.as_slice() {
            match value {
                XmlAttributeValue::Reference(v) => {
                    let v = str_from_reference(v, &self.owner)?;
                    normalized.push_str(normalize_ws(v.as_str()).as_str());
                }
                XmlAttributeValue::Text(ref v) => {
                    normalized.push_str(normalize_ws(v).as_str());
                }
            }
        }

        // TODO: attr type
        Ok(normalized)
    }

    fn specified(&self) -> bool {
        // TODO:
        true
    }

    fn attribute_type(&self) -> Value<Option<XmlDeclarationAttType>> {
        Value::V(self.find_delcaration_type())
    }

    fn references(&self) -> Value<Option<OrderedList<XmlItem>>> {
        // TODO:
        Value::V(None)
    }

    fn owner_element(&self) -> error::Result<XmlNode<XmlElement>> {
        self.parent.clone().ok_or(error::Error::IsolatedNode)
    }
}

impl PartialEq<XmlAttribute> for XmlAttribute {
    fn eq(&self, other: &XmlAttribute) -> bool {
        self.local_name == other.local_name
            && self.prefix == other.prefix
            && self.values == other.values
    }
}

impl XmlAttribute {
    pub fn new(
        value: &parser::Attribute,
        parent: XmlNode<XmlElement>,
        owner: XmlNode<XmlDocument>,
    ) -> XmlNode<Self> {
        let (local_name, prefix) = attribute_name(&value.name);
        let attribute = node(XmlAttribute {
            local_name,
            prefix,
            values: vec![],
            parent: Some(parent),
            owner,
        });

        for value in value.value.as_slice() {
            match value {
                parser::AttributeValue::Reference(v) => {
                    let value = XmlAttributeValue::Reference(XmlReference::new(v));
                    attribute.borrow_mut().push_value(value);
                }
                parser::AttributeValue::Text(v) => {
                    let value = XmlAttributeValue::Text(v.to_string());
                    attribute.borrow_mut().push_value(value);
                }
            }
        }

        attribute
    }

    fn find_delcaration_type(&self) -> Option<XmlDeclarationAttType> {
        let element = self.parent.as_ref()?;
        let doc = self.owner.borrow();
        let prolog = doc.prolog.borrow();
        let declaration = prolog.declaration()?.borrow();
        let elem = declaration
            .attributes()
            .iter()
            .find(|v| equal_qname(v.qname(), element.borrow().qname()))?;
        let def = elem
            .atts
            .iter()
            .find(|v| equal_qname(v.qname(), self.qname()))?;
        Some(def.ty.clone())
    }

    fn namespace(&self) -> bool {
        self.prefix().map(|p| p == "xmlns").unwrap_or_default() || self.local_name() == "xmlns"
    }

    fn push_value(&mut self, value: XmlAttributeValue) {
        self.values.push(value);
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum XmlAttributeValue {
    Text(String),
    Reference(XmlReference),
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlCData {
    data: String,
    parent: Option<XmlNode<XmlElement>>,
    owner: XmlNode<XmlDocument>,
}

impl Character for XmlCData {
    fn character_code(&self) -> &str {
        self.data.as_str()
    }

    fn element_content_whitespace(&self) -> Value<Option<bool>> {
        // TODO
        Value::V(None)
    }

    fn parent(&self) -> error::Result<XmlNode<XmlElement>> {
        self.parent.clone().ok_or(error::Error::IsolatedNode)
    }
}

impl PartialEq<XmlCData> for XmlCData {
    fn eq(&self, other: &XmlCData) -> bool {
        self.data == other.data
    }
}

impl XmlCData {
    pub fn new(
        value: &str,
        parent: XmlNode<XmlElement>,
        owner: XmlNode<XmlDocument>,
    ) -> XmlNode<Self> {
        let data = value.to_string();
        node(XmlCData {
            data,
            parent: Some(parent),
            owner,
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlCharReference {
    text: String,
    num: String,
    radix: u32,
    parent: Option<XmlNode<XmlElement>>,
    owner: XmlNode<XmlDocument>,
}

impl Character for XmlCharReference {
    fn character_code(&self) -> &str {
        self.text.as_str()
    }

    fn element_content_whitespace(&self) -> Value<Option<bool>> {
        // TODO
        Value::V(None)
    }

    fn parent(&self) -> error::Result<XmlNode<XmlElement>> {
        self.parent.clone().ok_or(error::Error::IsolatedNode)
    }
}

impl PartialEq<XmlCharReference> for XmlCharReference {
    fn eq(&self, other: &XmlCharReference) -> bool {
        self.text == other.text
    }
}

impl XmlCharReference {
    pub fn new(
        num: &str,
        radix: u32,
        parent: XmlNode<XmlElement>,
        owner: XmlNode<XmlDocument>,
    ) -> error::Result<XmlNode<Self>> {
        let num = num.to_string();
        let text = match radix {
            10 => char_from_char10(num.as_str()),
            16 => char_from_char16(num.as_str()),
            _ => unreachable!(),
        }?
        .to_string();
        Ok(node(XmlCharReference {
            text,
            num,
            radix,
            parent: Some(parent),
            owner,
        }))
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlComment {
    comment: String,
    parent: Option<XmlItem>,
    owner: XmlNode<XmlDocument>,
}

impl Comment for XmlComment {
    fn comment(&self) -> &str {
        self.comment.as_str()
    }

    fn parent(&self) -> error::Result<XmlItem> {
        self.parent.clone().ok_or(error::Error::IsolatedNode)
    }
}

impl PartialEq<XmlComment> for XmlComment {
    fn eq(&self, other: &XmlComment) -> bool {
        self.comment == other.comment
    }
}

impl XmlComment {
    pub fn new(comment: &str, parent: XmlItem, owner: XmlNode<XmlDocument>) -> XmlNode<Self> {
        let comment = comment.to_string();
        node(XmlComment {
            comment,
            parent: Some(parent),
            owner,
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlDeclarationAttDef {
    local_name: String,
    prefix: Option<String>,
    ty: XmlDeclarationAttType,
}

impl HasQName for XmlDeclarationAttDef {
    fn local_name(&self) -> &str {
        self.local_name.as_str()
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

impl From<&parser::DeclarationAttDef<'_>> for XmlDeclarationAttDef {
    fn from(value: &parser::DeclarationAttDef<'_>) -> Self {
        let (local_name, prefix) = match &value.name {
            parser::DeclarationAttName::Attr(v) => qname(v),
            parser::DeclarationAttName::Namsspace(v) => attribute_name(v),
        };

        let ty = XmlDeclarationAttType::from(&value.ty);

        XmlDeclarationAttDef {
            local_name,
            prefix,
            ty,
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlDeclarationAttList {
    local_name: String,
    prefix: Option<String>,
    atts: Vec<XmlDeclarationAttDef>,
    owner: XmlNode<XmlDocument>,
}

impl HasQName for XmlDeclarationAttList {
    fn local_name(&self) -> &str {
        self.local_name.as_str()
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

impl PartialEq<XmlDeclarationAttList> for XmlDeclarationAttList {
    fn eq(&self, other: &XmlDeclarationAttList) -> bool {
        self.local_name == other.local_name
            && self.prefix == other.prefix
            && self.atts == other.atts
    }
}

impl XmlDeclarationAttList {
    pub fn new(value: &parser::DeclarationAtt<'_>, owner: XmlNode<XmlDocument>) -> Self {
        let (local_name, prefix) = qname(&value.name);

        let atts = value.defs.iter().map(XmlDeclarationAttDef::from).collect();

        XmlDeclarationAttList {
            local_name,
            prefix,
            atts,
            owner,
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum XmlDeclarationAttType {
    Cdata,
    Entities,
    Entity,
    Id,
    IdRef,
    IdRefs,
    NmToken,
    NmTokens,
    Notation(Vec<String>),
    Enumeration(Vec<String>),
}

impl From<&parser::DeclarationAttType<'_>> for XmlDeclarationAttType {
    fn from(value: &parser::DeclarationAttType<'_>) -> Self {
        match value {
            parser::DeclarationAttType::Cdata => XmlDeclarationAttType::Cdata,
            parser::DeclarationAttType::Entities => XmlDeclarationAttType::Entities,
            parser::DeclarationAttType::Entity => XmlDeclarationAttType::Entity,
            parser::DeclarationAttType::Id => XmlDeclarationAttType::Id,
            parser::DeclarationAttType::IdRef => XmlDeclarationAttType::IdRef,
            parser::DeclarationAttType::IdRefs => XmlDeclarationAttType::IdRefs,
            parser::DeclarationAttType::NmToken => XmlDeclarationAttType::NmToken,
            parser::DeclarationAttType::NmTokens => XmlDeclarationAttType::NmTokens,
            parser::DeclarationAttType::Notation(v) => {
                XmlDeclarationAttType::Notation(v.iter().map(|i| i.to_string()).collect())
            }
            parser::DeclarationAttType::Enumeration(v) => {
                XmlDeclarationAttType::Enumeration(v.iter().map(|i| i.to_string()).collect())
            }
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlDocument {
    prolog: XmlNode<XmlDocumentProlog>,
    root: Option<XmlNode<XmlElement>>,
    epilog: XmlNode<XmlDocumentEpilog>,
    base_uri: String,
    encoding: String,
    standalone: Option<bool>,
    version: Option<String>,
    all_declarations_processed: bool,
}

impl Document for XmlDocument {
    fn children(&self) -> OrderedList<XmlItem> {
        let mut items = vec![];

        for v in self.prolog.borrow().head.as_slice() {
            items.push(v.clone());
        }

        if let Some(v) = &self.prolog.borrow().declaration {
            items.push(XmlItem::DocumentType(v.clone()));
        }

        for v in self.prolog.borrow().tail.as_slice() {
            items.push(v.clone());
        }

        if let Some(v) = &self.root {
            items.push(XmlItem::Element(v.clone()));
        }

        for v in self.epilog.borrow().head.as_slice() {
            items.push(v.clone());
        }

        OrderedList::new(items)
    }

    fn document_element(&self) -> error::Result<XmlNode<XmlElement>> {
        self.root
            .clone()
            .ok_or(error::Error::NotFoundDoumentElement)
    }

    fn notations(&self) -> Option<UnorderedSet<XmlNode<XmlNotation>>> {
        let items = self
            .prolog
            .borrow()
            .declaration
            .as_ref()
            .map(|v| v.borrow().notations())
            .unwrap_or_default();
        for item in items.as_slice() {
            let name = item.borrow().name().to_string();
            if items.iter().filter(|v| v.borrow().name() == name).count() > 1 {
                return None;
            }
        }
        Some(UnorderedSet::new(items))
    }

    fn unparsed_entities(&self) -> UnorderedSet<XmlNode<XmlUnparsedEntity>> {
        let items = self
            .prolog
            .borrow()
            .declaration
            .as_ref()
            .map(|v| v.borrow().unparsed_entities())
            .unwrap_or_default();
        UnorderedSet::new(items)
    }

    fn base_uri(&self) -> &str {
        self.base_uri.as_str()
    }

    fn character_encoding_scheme(&self) -> &str {
        self.encoding.as_str()
    }

    fn standalone(&self) -> Option<bool> {
        self.standalone
    }

    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    fn all_declarations_processed(&self) -> bool {
        self.all_declarations_processed
    }
}

impl PartialEq<XmlDocument> for XmlDocument {
    fn eq(&self, other: &XmlDocument) -> bool {
        self.prolog == other.prolog
            && self.root == other.root
            && self.epilog == other.epilog
            && self.encoding == other.encoding
            && self.standalone == other.standalone
            && self.version == other.version
    }
}

impl XmlDocument {
    pub fn new(value: &parser::Document<'_>) -> error::Result<XmlNode<Self>> {
        let document = node(XmlDocument {
            prolog: node(XmlDocumentProlog::default()),
            root: None,
            epilog: node(XmlDocumentEpilog::default()),
            base_uri: String::new(),
            encoding: xml_encoding(value),
            standalone: xml_standalone(value),
            version: xml_version(value),
            all_declarations_processed: true,
        });

        let prolog = XmlDocumentProlog::new(&value.prolog, document.clone());
        document.borrow_mut().set_prolog(prolog);

        let element = XmlElement::new(&value.element, document.clone().into(), document.clone());
        document.borrow_mut().set_root(element?);

        let epilog = XmlDocumentEpilog::new(value, document.clone());
        document.borrow_mut().set_epilog(epilog);

        Ok(document)
    }

    pub fn prolog(&self) -> XmlNode<XmlDocumentProlog> {
        self.prolog.clone()
    }

    fn set_epilog(&mut self, epilog: XmlNode<XmlDocumentEpilog>) {
        self.epilog = epilog;
    }

    fn set_prolog(&mut self, prolog: XmlNode<XmlDocumentProlog>) {
        self.prolog = prolog;
    }

    fn set_root(&mut self, root: XmlNode<XmlElement>) {
        self.root = Some(root);
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct XmlDocumentEpilog {
    head: Vec<XmlItem>,
}

impl XmlDocumentEpilog {
    pub fn new(value: &parser::Document, owner: XmlNode<XmlDocument>) -> XmlNode<Self> {
        let mut head = vec![];

        for h in value.miscs.as_slice() {
            match h {
                parser::Misc::Comment(c) => {
                    let c = XmlComment::new(c.value, owner.clone().into(), owner.clone());
                    head.push(c.into());
                }
                parser::Misc::PI(p) => {
                    let p = XmlProcessingInstruction::new(p, owner.clone().into(), owner.clone());
                    head.push(p.into());
                }
                parser::Misc::Whitespace(_) => {}
            }
        }

        node(XmlDocumentEpilog { head })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct XmlDocumentProlog {
    head: Vec<XmlItem>,
    declaration: Option<XmlNode<XmlDocumentTypeDeclaration>>,
    tail: Vec<XmlItem>,
}

impl XmlDocumentProlog {
    pub fn new(value: &parser::Prolog, owner: XmlNode<XmlDocument>) -> XmlNode<Self> {
        let mut head = vec![];
        let mut declaration = None;
        let mut tail = vec![];

        for h in value.heads.as_slice() {
            match h {
                parser::Misc::Comment(c) => {
                    let c = XmlComment::new(c.value, owner.clone().into(), owner.clone());
                    head.push(c.into());
                }
                parser::Misc::PI(p) => {
                    let p = XmlProcessingInstruction::new(p, owner.clone().into(), owner.clone());
                    head.push(p.into());
                }
                parser::Misc::Whitespace(_) => {}
            }
        }

        if let Some(d) = value.declaration_doc.as_ref() {
            declaration = Some(XmlDocumentTypeDeclaration::new(d, owner.clone()));
        }

        for t in value.tails.as_slice() {
            match t {
                parser::Misc::Comment(c) => {
                    let c = XmlComment::new(c.value, owner.clone().into(), owner.clone());
                    tail.push(c.into());
                }
                parser::Misc::PI(p) => {
                    let p = XmlProcessingInstruction::new(p, owner.clone().into(), owner.clone());
                    tail.push(p.into());
                }
                parser::Misc::Whitespace(_) => {}
            }
        }

        node(XmlDocumentProlog {
            head,
            declaration,
            tail,
        })
    }

    pub fn declaration(&self) -> Option<&XmlNode<XmlDocumentTypeDeclaration>> {
        self.declaration.as_ref()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlDocumentTypeDeclaration {
    local_name: String,
    prefix: Option<String>,
    system_identifier: Option<String>,
    public_identifier: Option<String>,
    attributes: Vec<XmlDeclarationAttList>,
    entities: Vec<XmlNode<XmlEntity>>,
    pis: Vec<XmlNode<XmlProcessingInstruction>>,
    notations: Vec<XmlNode<XmlNotation>>,
    parent: XmlNode<XmlDocument>,
}

impl HasQName for XmlDocumentTypeDeclaration {
    fn local_name(&self) -> &str {
        self.local_name.as_str()
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

impl DocumentTypeDeclaration for XmlDocumentTypeDeclaration {
    fn system_identifier(&self) -> Option<&str> {
        self.system_identifier.as_deref()
    }

    fn public_identifier(&self) -> Option<&str> {
        self.public_identifier.as_deref()
    }

    fn children(&self) -> OrderedList<XmlNode<XmlProcessingInstruction>> {
        OrderedList::new(self.pis.clone())
    }

    fn parent(&self) -> XmlNode<XmlDocument> {
        self.parent.clone()
    }
}

impl PartialEq<XmlDocumentTypeDeclaration> for XmlDocumentTypeDeclaration {
    fn eq(&self, other: &XmlDocumentTypeDeclaration) -> bool {
        self.local_name == other.local_name
            && self.prefix == other.prefix
            && self.system_identifier == other.system_identifier
            && self.public_identifier == other.public_identifier
            && self.attributes == other.attributes
            && self.entities == other.entities
            && self.pis == other.pis
            && self.notations == other.notations
    }
}

impl XmlDocumentTypeDeclaration {
    pub fn new(value: &parser::DeclarationDoc<'_>, parent: XmlNode<XmlDocument>) -> XmlNode<Self> {
        let (local_name, prefix) = qname(&value.name);

        let (system_identifier, public_identifier) = match value.external_id.as_ref() {
            Some(id) => {
                let (s, p) = external_id(id);
                (Some(s), p)
            }
            _ => (None, None),
        };

        let declaration = node(XmlDocumentTypeDeclaration {
            local_name,
            prefix,
            system_identifier,
            public_identifier,
            attributes: vec![],
            entities: vec![],
            pis: vec![],
            notations: vec![],
            parent: parent.clone(),
        });

        for subset in &value.internal_subset {
            match subset {
                parser::InternalSubset::Markup(v) => match v {
                    parser::DeclarationMarkup::Attributes(v) => {
                        let attribute = XmlDeclarationAttList::new(v, parent.clone());
                        declaration.borrow_mut().push_attribute(attribute);
                    }
                    parser::DeclarationMarkup::Commnect(_) => {
                        // drop
                    }
                    parser::DeclarationMarkup::Element(_) => {
                        // drop
                    }
                    parser::DeclarationMarkup::Entity(v) => match v {
                        parser::DeclarationEntity::GeneralEntity(v) => {
                            let entity = XmlEntity::new(v, parent.clone());
                            declaration.borrow_mut().push_entity(entity);
                        }
                        parser::DeclarationEntity::ParameterEntity(_) => {
                            unimplemented!("Not support parameter entity reference.")
                        }
                    },
                    parser::DeclarationMarkup::Notation(v) => {
                        let notation =
                            XmlNotation::new(v, declaration.clone().into(), parent.clone());
                        declaration.borrow_mut().push_notation(notation);
                    }
                    parser::DeclarationMarkup::PI(v) => {
                        let pi = XmlProcessingInstruction::new(
                            v,
                            declaration.clone().into(),
                            parent.clone(),
                        );
                        declaration.borrow_mut().push_pi(pi);
                    }
                },
                parser::InternalSubset::ParameterEntityReference(_) => {
                    unimplemented!("Not support parameter entity reference.")
                }
            }
        }

        declaration
    }

    pub fn attributes(&self) -> &[XmlDeclarationAttList] {
        self.attributes.as_slice()
    }

    pub fn entities(&self) -> &[XmlNode<XmlEntity>] {
        self.entities.as_slice()
    }

    pub fn notations(&self) -> Vec<XmlNode<XmlNotation>> {
        self.notations.clone()
    }

    pub fn unparsed_entities(&self) -> Vec<XmlNode<XmlUnparsedEntity>> {
        self.entities
            .iter()
            .filter(|v| v.borrow().notation_name.is_some())
            .map(|v| XmlUnparsedEntity::new(v.clone()))
            .collect()
    }

    fn push_attribute(&mut self, attribute: XmlDeclarationAttList) {
        self.attributes.push(attribute);
    }

    fn push_entity(&mut self, entity: XmlNode<XmlEntity>) {
        self.entities.push(entity);
    }

    fn push_notation(&mut self, notation: XmlNode<XmlNotation>) {
        self.notations.push(notation);
    }

    fn push_pi(&mut self, pi: XmlNode<XmlProcessingInstruction>) {
        self.pis.push(pi);
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlElement {
    local_name: String,
    prefix: Option<String>,
    children: Vec<XmlItem>,
    attributes: Vec<XmlNode<XmlAttribute>>,
    base_uri: String,
    parent: Option<XmlItem>,
}

impl HasQName for XmlElement {
    fn local_name(&self) -> &str {
        self.local_name.as_str()
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

impl Element for XmlElement {
    fn namespace_name(&self) -> error::Result<Option<NamespaceUri>> {
        let prefix = self.prefix().unwrap_or("xmlns");
        self.find_nameapce_uri(prefix)
    }

    fn children(&self) -> OrderedList<XmlItem> {
        OrderedList::new(self.children.clone())
    }

    fn attributes(&self) -> UnorderedSet<XmlNode<XmlAttribute>> {
        let items = self
            .attributes
            .iter()
            .filter(|v| !v.borrow().namespace())
            .cloned()
            .collect();
        UnorderedSet::new(items)
    }

    fn namespace_attributes(&self) -> UnorderedSet<XmlNode<XmlAttribute>> {
        let items = self
            .attributes
            .iter()
            .filter(|v| v.borrow().namespace())
            .cloned()
            .collect();
        UnorderedSet::new(items)
    }

    fn in_scope_namespace(&self) -> error::Result<UnorderedSet<XmlNode<XmlNamespace>>> {
        let mut items = vec![];

        for attr in self.namespace_attributes().iter() {
            let namespace_name = attr.borrow().normalized_value()?;
            if !namespace_name.is_empty() {
                if attr.borrow().local_name() == "xmlns" {
                    items.push(node(XmlNamespace {
                        prefix: None,
                        namespace_name,
                    }));
                } else {
                    items.push(node(XmlNamespace {
                        prefix: Some(attr.borrow().local_name().to_string()),
                        namespace_name,
                    }));
                }
            }
        }

        if let Some(parent) = self.parent.as_ref() {
            if let Some(parent) = parent.as_element() {
                for ns in parent.borrow().in_scope_namespace()?.iter() {
                    let exist = items.iter().any(|v| *v == ns);
                    if !exist {
                        items.push(ns);
                    }
                }
            } else if parent.as_document().is_some() {
                let implicity = node(XmlNamespace {
                    prefix: Some("xml".to_string()),
                    namespace_name: "http://www.w3.org/XML/1998/namespace".to_string(),
                });
                let exist = items.iter().any(|v| *v == implicity);
                if !exist {
                    items.push(implicity);
                }
            }
        }

        Ok(UnorderedSet::new(items))
    }

    fn base_uri(&self) -> &str {
        self.base_uri.as_str()
    }

    fn parent(&self) -> error::Result<XmlItem> {
        self.parent.clone().ok_or(error::Error::IsolatedNode)
    }
}

impl PartialEq<XmlElement> for XmlElement {
    fn eq(&self, other: &XmlElement) -> bool {
        self.local_name == other.local_name
            && self.prefix == other.prefix
            && self.children == other.children
            && self.attributes == other.attributes
    }
}

impl XmlElement {
    pub fn new(
        value: &parser::Element<'_>,
        parent: XmlItem,
        owner: XmlNode<XmlDocument>,
    ) -> error::Result<XmlNode<Self>> {
        let (local_name, prefix) = qname(&value.name);

        let element = node(XmlElement {
            local_name,
            prefix,
            children: vec![],
            attributes: vec![],
            base_uri: String::new(),
            parent: Some(parent),
        });

        if let Some(content) = &value.content {
            if let Some(head) = content.head {
                if !head.is_empty() {
                    let text = XmlText::new(head, element.clone(), owner.clone());
                    element.borrow_mut().push_child(text.into());
                }
            }

            for cell in content.children.as_slice() {
                match &cell.child {
                    parser::Contents::Element(v) => {
                        let child = XmlElement::new(v, element.clone().into(), owner.clone())?;
                        element.borrow_mut().push_child(child.into());
                    }
                    parser::Contents::Reference(v) => match v {
                        parser::Reference::Character(ch, radix) => {
                            let reference =
                                XmlCharReference::new(ch, *radix, element.clone(), owner.clone())?;
                            element.borrow_mut().push_child(reference.into());
                        }
                        parser::Reference::Entity(v) => {
                            let entity = entity(&owner, v)?;
                            let entity = XmlUnexpandedEntityReference::new(
                                entity,
                                element.clone(),
                                owner.clone(),
                            );
                            element.borrow_mut().push_child(entity.into());
                        }
                    },
                    parser::Contents::CData(v) => {
                        let cdata = XmlCData::new(v.value, element.clone(), owner.clone());
                        element.borrow_mut().push_child(cdata.into());
                    }
                    parser::Contents::PI(v) => {
                        let pi =
                            XmlProcessingInstruction::new(v, element.clone().into(), owner.clone());
                        element.borrow_mut().push_child(pi.into());
                    }
                    parser::Contents::Comment(v) => {
                        let comment =
                            XmlComment::new(v.value, element.clone().into(), owner.clone());
                        element.borrow_mut().push_child(comment.into());
                    }
                }

                if let Some(tail) = cell.tail {
                    if !tail.is_empty() {
                        let text = XmlText::new(tail, element.clone(), owner.clone());
                        element.borrow_mut().push_child(text.into());
                    }
                }
            }
        }

        for attribute in value.attributes.as_slice() {
            let attr = XmlAttribute::new(attribute, element.clone(), owner.clone());
            element.borrow_mut().push_attribute(attr);
        }

        Ok(element)
    }

    pub fn set_local_name(&mut self, local_name: &str) {
        self.local_name = local_name.to_string();
    }

    fn find_nameapce_uri(&self, prefix: &str) -> error::Result<Option<NamespaceUri>> {
        for namespace in self.namespace_attributes().iter() {
            if prefix == namespace.borrow().local_name() {
                return Ok(Some(NamespaceUri::try_from(&namespace)?));
            }
        }

        for namespace in self.in_scope_namespace()?.iter() {
            if prefix == namespace.borrow().prefix().unwrap_or_default() {
                return Ok(Some(NamespaceUri::from(&namespace)));
            }
        }

        Ok(None)
    }

    fn push_attribute(&mut self, attr: XmlNode<XmlAttribute>) {
        self.attributes.push(attr);
    }

    fn push_child(&mut self, child: XmlItem) {
        self.children.push(child);
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlEntity {
    name: String,
    values: Option<Vec<XmlEntityValue>>,
    system_identifier: Option<String>,
    public_identifier: Option<String>,
    notation_name: Option<String>,
    owner: XmlNode<XmlDocument>,
}

impl From<(&str, &str, XmlNode<XmlDocument>)> for XmlEntity {
    fn from(value: (&str, &str, XmlNode<XmlDocument>)) -> Self {
        let (name, value, owner) = value;
        XmlEntity {
            name: name.to_string(),
            values: Some(vec![XmlEntityValue::Text(value.to_string())]),
            system_identifier: None,
            public_identifier: None,
            notation_name: None,
            owner,
        }
    }
}

impl PartialEq<XmlEntity> for XmlEntity {
    fn eq(&self, other: &XmlEntity) -> bool {
        self.name == other.name
            && self.values == other.values
            && self.system_identifier == other.system_identifier
            && self.public_identifier == other.public_identifier
            && self.notation_name == other.notation_name
    }
}

impl XmlEntity {
    pub fn new(
        value: &parser::DeclarationGeneralEntity,
        owner: XmlNode<XmlDocument>,
    ) -> XmlNode<Self> {
        let name = value.name.to_string();

        let (values, system_identifier, public_identifier, notation_name) = match &value.def {
            parser::DeclarationEntityDef::EntityValue(v) => {
                let values = v.iter().map(XmlEntityValue::new).collect();
                (Some(values), None, None, None)
            }
            parser::DeclarationEntityDef::ExternalId(v, n) => {
                let (s, p) = external_id(v);
                let n = n.map(|n| n.to_string());
                (None, Some(s), p, n)
            }
        };

        node(XmlEntity {
            name,
            values,
            system_identifier,
            public_identifier,
            notation_name,
            owner,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn values(&self) -> Option<&[XmlEntityValue]> {
        self.values.as_deref()
    }

    pub fn system_identifier(&self) -> Option<&str> {
        self.system_identifier.as_deref()
    }

    pub fn public_identifier(&self) -> Option<&str> {
        self.public_identifier.as_deref()
    }

    pub fn notation_name(&self) -> Option<&str> {
        self.notation_name.as_deref()
    }

    pub fn owner(&self) -> XmlNode<XmlDocument> {
        self.owner.clone()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum XmlEntityValue {
    Text(String),
    Parameter(String),
    Reference(XmlReference),
}

impl XmlEntityValue {
    pub fn new(value: &parser::EntityValue<'_>) -> Self {
        match value {
            parser::EntityValue::ParameterEntityReference(v) => {
                XmlEntityValue::Parameter(v.to_string())
            }
            parser::EntityValue::Reference(v) => XmlEntityValue::Reference(XmlReference::new(v)),
            parser::EntityValue::Text(v) => XmlEntityValue::Text(v.to_string()),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum XmlItem {
    Attribute(XmlNode<XmlAttribute>),
    CData(XmlNode<XmlCData>),
    CharReference(XmlNode<XmlCharReference>),
    Comment(XmlNode<XmlComment>),
    Document(XmlNode<XmlDocument>),
    DocumentType(XmlNode<XmlDocumentTypeDeclaration>),
    Element(XmlNode<XmlElement>),
    Namespace(XmlNode<XmlNamespace>),
    Notation(XmlNode<XmlNotation>),
    PI(XmlNode<XmlProcessingInstruction>),
    Text(XmlNode<XmlText>),
    Unexpanded(XmlNode<XmlUnexpandedEntityReference>),
    Unparsed(XmlNode<XmlUnparsedEntity>),
}

impl From<XmlNode<XmlAttribute>> for XmlItem {
    fn from(value: XmlNode<XmlAttribute>) -> Self {
        XmlItem::Attribute(value)
    }
}

impl From<XmlNode<XmlCData>> for XmlItem {
    fn from(value: XmlNode<XmlCData>) -> Self {
        XmlItem::CData(value)
    }
}

impl From<XmlNode<XmlCharReference>> for XmlItem {
    fn from(value: XmlNode<XmlCharReference>) -> Self {
        XmlItem::CharReference(value)
    }
}

impl From<XmlNode<XmlComment>> for XmlItem {
    fn from(value: XmlNode<XmlComment>) -> Self {
        XmlItem::Comment(value)
    }
}

impl From<XmlNode<XmlDocument>> for XmlItem {
    fn from(value: XmlNode<XmlDocument>) -> Self {
        XmlItem::Document(value)
    }
}

impl From<XmlNode<XmlDocumentTypeDeclaration>> for XmlItem {
    fn from(value: XmlNode<XmlDocumentTypeDeclaration>) -> Self {
        XmlItem::DocumentType(value)
    }
}

impl From<XmlNode<XmlElement>> for XmlItem {
    fn from(value: XmlNode<XmlElement>) -> Self {
        XmlItem::Element(value)
    }
}

impl From<XmlNode<XmlNamespace>> for XmlItem {
    fn from(value: XmlNode<XmlNamespace>) -> Self {
        XmlItem::Namespace(value)
    }
}

impl From<XmlNode<XmlNotation>> for XmlItem {
    fn from(value: XmlNode<XmlNotation>) -> Self {
        XmlItem::Notation(value)
    }
}

impl From<XmlNode<XmlProcessingInstruction>> for XmlItem {
    fn from(value: XmlNode<XmlProcessingInstruction>) -> Self {
        XmlItem::PI(value)
    }
}

impl From<XmlNode<XmlText>> for XmlItem {
    fn from(value: XmlNode<XmlText>) -> Self {
        XmlItem::Text(value)
    }
}

impl From<XmlNode<XmlUnexpandedEntityReference>> for XmlItem {
    fn from(value: XmlNode<XmlUnexpandedEntityReference>) -> Self {
        XmlItem::Unexpanded(value)
    }
}

impl From<XmlNode<XmlUnparsedEntity>> for XmlItem {
    fn from(value: XmlNode<XmlUnparsedEntity>) -> Self {
        XmlItem::Unparsed(value)
    }
}

impl From<XmlAttribute> for XmlItem {
    fn from(value: XmlAttribute) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlCData> for XmlItem {
    fn from(value: XmlCData) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlCharReference> for XmlItem {
    fn from(value: XmlCharReference) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlComment> for XmlItem {
    fn from(value: XmlComment) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlDocument> for XmlItem {
    fn from(value: XmlDocument) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlDocumentTypeDeclaration> for XmlItem {
    fn from(value: XmlDocumentTypeDeclaration) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlElement> for XmlItem {
    fn from(value: XmlElement) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlNamespace> for XmlItem {
    fn from(value: XmlNamespace) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlNotation> for XmlItem {
    fn from(value: XmlNotation) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlProcessingInstruction> for XmlItem {
    fn from(value: XmlProcessingInstruction) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlText> for XmlItem {
    fn from(value: XmlText) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlUnexpandedEntityReference> for XmlItem {
    fn from(value: XmlUnexpandedEntityReference) -> Self {
        XmlItem::from(node(value))
    }
}

impl From<XmlUnparsedEntity> for XmlItem {
    fn from(value: XmlUnparsedEntity) -> Self {
        XmlItem::from(node(value))
    }
}

impl XmlItem {
    pub fn as_attribute(&self) -> Option<XmlNode<XmlAttribute>> {
        if let XmlItem::Attribute(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_cdata(&self) -> Option<XmlNode<XmlCData>> {
        if let XmlItem::CData(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_char_reference(&self) -> Option<XmlNode<XmlCharReference>> {
        if let XmlItem::CharReference(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_comment(&self) -> Option<XmlNode<XmlComment>> {
        if let XmlItem::Comment(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_document(&self) -> Option<XmlNode<XmlDocument>> {
        if let XmlItem::Document(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_document_type(&self) -> Option<XmlNode<XmlDocumentTypeDeclaration>> {
        if let XmlItem::DocumentType(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_element(&self) -> Option<XmlNode<XmlElement>> {
        if let XmlItem::Element(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_namespace(&self) -> Option<XmlNode<XmlNamespace>> {
        if let XmlItem::Namespace(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_notation(&self) -> Option<XmlNode<XmlNotation>> {
        if let XmlItem::Notation(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_pi(&self) -> Option<XmlNode<XmlProcessingInstruction>> {
        if let XmlItem::PI(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<XmlNode<XmlText>> {
        if let XmlItem::Text(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_unexpanded(&self) -> Option<XmlNode<XmlUnexpandedEntityReference>> {
        if let XmlItem::Unexpanded(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_unparsed(&self) -> Option<XmlNode<XmlUnparsedEntity>> {
        if let XmlItem::Unparsed(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlNamespace {
    prefix: Option<String>,
    namespace_name: String,
}

impl Namespace for XmlNamespace {
    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    fn namespace_name(&self) -> &str {
        self.namespace_name.as_str()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlNotation {
    name: String,
    system_identifier: Option<String>,
    public_identifier: Option<String>,
    declaration_base_uri: String,
    parent: XmlItem,
    owner: XmlNode<XmlDocument>,
}

impl Notation for XmlNotation {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn system_identifier(&self) -> Option<&str> {
        self.system_identifier.as_deref()
    }

    fn public_identifier(&self) -> Option<&str> {
        self.public_identifier.as_deref()
    }

    fn declaration_base_uri(&self) -> &str {
        self.declaration_base_uri.as_str()
    }
}

impl PartialEq<XmlNotation> for XmlNotation {
    fn eq(&self, other: &XmlNotation) -> bool {
        self.name == other.name
            && self.system_identifier == other.system_identifier
            && self.public_identifier == other.public_identifier
    }
}

impl XmlNotation {
    pub fn new(
        value: &parser::DeclarationNotation,
        parent: XmlItem,
        owner: XmlNode<XmlDocument>,
    ) -> XmlNode<Self> {
        let name = value.name.to_string();

        let (system_identifier, public_identifier) = match &value.id {
            parser::DeclarationNotationId::ExternalId(id) => {
                let (s, p) = external_id(id);
                (Some(s), p)
            }
            parser::DeclarationNotationId::PublicId(p) => (None, Some(p.to_string())),
        };

        let declaration_base_uri = String::new();

        node(XmlNotation {
            name,
            system_identifier,
            public_identifier,
            declaration_base_uri,
            parent,
            owner,
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlProcessingInstruction {
    target: String,
    content: Option<String>,
    base_uri: String,
    parent: XmlItem,
    owner: XmlNode<XmlDocument>,
}

impl ProcessingInstruction for XmlProcessingInstruction {
    fn target(&self) -> &str {
        self.target.as_str()
    }

    fn content(&self) -> &str {
        self.content.as_deref().unwrap_or_default()
    }

    fn base_uri(&self) -> &str {
        self.base_uri.as_str()
    }

    fn notation(&self) -> Value<Option<XmlNode<XmlNotation>>> {
        notation(&self.owner, self.target())
    }

    fn parent(&self) -> XmlItem {
        self.parent.clone()
    }
}

impl PartialEq<XmlProcessingInstruction> for XmlProcessingInstruction {
    fn eq(&self, other: &XmlProcessingInstruction) -> bool {
        self.target == other.target && self.content == other.content
    }
}

impl XmlProcessingInstruction {
    pub fn new(
        value: &parser::PI<'_>,
        parent: XmlItem,
        owner: XmlNode<XmlDocument>,
    ) -> XmlNode<Self> {
        let target = value.target.to_string();

        let content = value.value.map(|v| v.to_string());

        let base_uri = String::new();

        node(XmlProcessingInstruction {
            target,
            content,
            base_uri,
            parent,
            owner,
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum XmlReference {
    Character(String, u32),
    Entity(String),
}

impl XmlReference {
    pub fn new(value: &parser::Reference) -> Self {
        match value {
            parser::Reference::Character(v, r) => XmlReference::Character(v.to_string(), *r),
            parser::Reference::Entity(v) => XmlReference::Entity(v.to_string()),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlText {
    text: String,
    parent: Option<XmlNode<XmlElement>>,
    owner: XmlNode<XmlDocument>,
}

impl Character for XmlText {
    fn character_code(&self) -> &str {
        self.text.as_str()
    }

    fn element_content_whitespace(&self) -> Value<Option<bool>> {
        // TODO
        Value::V(None)
    }

    fn parent(&self) -> error::Result<XmlNode<XmlElement>> {
        self.parent.clone().ok_or(error::Error::IsolatedNode)
    }
}

impl PartialEq<XmlText> for XmlText {
    fn eq(&self, other: &XmlText) -> bool {
        self.text == other.text
    }
}

impl XmlText {
    pub fn new(
        value: &str,
        parent: XmlNode<XmlElement>,
        owner: XmlNode<XmlDocument>,
    ) -> XmlNode<Self> {
        let text = value.to_string();
        node(XmlText {
            text,
            parent: Some(parent),
            owner,
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlUnexpandedEntityReference {
    entity: XmlNode<XmlEntity>,
    name: String,
    system_identifier: Option<String>,
    public_identifier: Option<String>,
    declaration_base_uri: String,
    parent: XmlNode<XmlElement>,
    owner: XmlNode<XmlDocument>,
}

impl UnexpandedEntityReference for XmlUnexpandedEntityReference {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn system_identifier(&self) -> Value<Option<&str>> {
        Value::V(self.system_identifier.as_deref())
    }

    fn public_identifier(&self) -> Value<Option<&str>> {
        Value::V(self.public_identifier.as_deref())
    }

    fn declaration_base_uri(&self) -> &str {
        self.declaration_base_uri.as_str()
    }

    fn parent(&self) -> XmlNode<XmlElement> {
        self.parent.clone()
    }
}

impl PartialEq<XmlUnexpandedEntityReference> for XmlUnexpandedEntityReference {
    fn eq(&self, other: &XmlUnexpandedEntityReference) -> bool {
        self.entity == other.entity
    }
}

impl XmlUnexpandedEntityReference {
    pub fn new(
        entity: XmlNode<XmlEntity>,
        parent: XmlNode<XmlElement>,
        owner: XmlNode<XmlDocument>,
    ) -> XmlNode<Self> {
        let name = entity.borrow().name().to_string();

        let system_identifier = entity.borrow().system_identifier().map(|v| v.to_string());

        let public_identifier = entity.borrow().public_identifier().map(|v| v.to_string());

        let declaration_base_uri = String::new();

        node(XmlUnexpandedEntityReference {
            entity,
            name,
            system_identifier,
            public_identifier,
            declaration_base_uri,
            parent,
            owner,
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlUnparsedEntity {
    entity: XmlNode<XmlEntity>,
    name: String,
    system_identifier: String,
    public_identifier: Option<String>,
    declaration_base_uri: String,
    notation_name: String,
}

impl UnparsedEntity for XmlUnparsedEntity {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn system_identifier(&self) -> &str {
        self.system_identifier.as_str()
    }

    fn public_identifier(&self) -> Option<&str> {
        self.public_identifier.as_deref()
    }

    fn declaration_base_uri(&self) -> &str {
        self.declaration_base_uri.as_str()
    }

    fn notation_name(&self) -> &str {
        self.notation_name.as_str()
    }

    fn notation(&self) -> Value<Option<XmlNode<XmlNotation>>> {
        notation(&self.entity.borrow().owner(), self.notation_name())
    }
}

impl XmlUnparsedEntity {
    pub fn new(entity: XmlNode<XmlEntity>) -> XmlNode<Self> {
        let name = entity.borrow().name().to_string();

        let system_identifier = entity.borrow().system_identifier().unwrap().to_string();

        let public_identifier = entity.borrow().public_identifier().map(|v| v.to_string());

        let declaration_base_uri = String::new();

        let notation_name = entity.borrow().notation_name().unwrap().to_string();

        node(XmlUnparsedEntity {
            entity,
            name,
            system_identifier,
            public_identifier,
            declaration_base_uri,
            notation_name,
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct NamespaceUri {
    value: String,
}

impl Deref for NamespaceUri {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.value.as_str()
    }
}

impl From<&str> for NamespaceUri {
    fn from(value: &str) -> Self {
        NamespaceUri {
            value: value.to_string(),
        }
    }
}

impl From<&XmlNode<XmlNamespace>> for NamespaceUri {
    fn from(value: &XmlNode<XmlNamespace>) -> Self {
        let value = value.borrow().namespace_name().to_string();
        NamespaceUri { value }
    }
}

impl TryFrom<&XmlNode<XmlAttribute>> for NamespaceUri {
    type Error = error::Error;

    fn try_from(value: &XmlNode<XmlAttribute>) -> Result<Self, Self::Error> {
        let value = value.borrow().normalized_value()?.to_string();
        Ok(NamespaceUri { value })
    }
}

impl NamespaceUri {
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct OrderedList<T>
where
    T: Clone,
{
    items: Vec<T>,
}

impl<T> OrderedList<T>
where
    T: Clone,
{
    pub fn new(items: Vec<T>) -> Self {
        OrderedList { items }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn iter(&self) -> OrderedListIter<'_, T> {
        OrderedListIter {
            items: self.items.as_slice(),
            index: 0,
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub struct OrderedListIter<'a, T>
where
    T: Clone,
{
    items: &'a [T],
    index: usize,
}

impl<'a, T> Iterator for OrderedListIter<'a, T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.get(self.index);
        self.index += 1;
        item.cloned()
    }
}

impl<'a, T> OrderedListIter<'a, T>
where
    T: Clone,
{
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct UnorderedSet<T>
where
    T: Clone,
{
    items: Vec<T>,
}

impl<T> UnorderedSet<T>
where
    T: Clone,
{
    pub fn new(items: Vec<T>) -> Self {
        UnorderedSet { items }
    }

    pub fn iter(&self) -> UnorderedSetIter<'_, T> {
        UnorderedSetIter {
            items: self.items.as_slice(),
            index: 0,
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub struct UnorderedSetIter<'a, T>
where
    T: Clone,
{
    items: &'a [T],
    index: usize,
}

impl<'a, T> Iterator for UnorderedSetIter<'a, T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.get(self.index);
        self.index += 1;
        item.cloned()
    }
}

impl<'a, T> UnorderedSetIter<'a, T>
where
    T: Clone,
{
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Value<T>
where
    T: Clone,
{
    Unknown,
    V(T),
}

// -----------------------------------------------------------------------------------------------

fn attribute_name(name: &parser::AttributeName) -> (String, Option<String>) {
    match name {
        parser::AttributeName::DefaultNamespace => ("xmlns".to_string(), None),
        parser::AttributeName::Namespace(v) => (v.to_string(), Some("xmlns".to_string())),
        parser::AttributeName::QName(v) => qname(v),
    }
}

fn char_from_char10(value: &str) -> error::Result<char> {
    let num = value
        .parse::<u32>()
        .map_err(|_| error::Error::NotFoundReference(format!("#{}", value)))?;
    char::from_u32(num).ok_or(error::Error::NotFoundReference(format!("#{}", value)))
}

fn char_from_char16(value: &str) -> error::Result<char> {
    let num = u32::from_str_radix(value, 16)
        .map_err(|_| error::Error::NotFoundReference(format!("#x{}", value)))?;
    char::from_u32(num).ok_or(error::Error::NotFoundReference(format!("#x{}", value)))
}

fn entity(document: &XmlNode<XmlDocument>, name: &str) -> error::Result<XmlNode<XmlEntity>> {
    if let Some(declaration) = document.borrow().prolog().borrow().declaration() {
        if let Some(v) = declaration
            .borrow()
            .entities()
            .iter()
            .find(|v| v.borrow().name() == name)
            .cloned()
        {
            return Ok(v);
        }
    }

    match name {
        "lt" => Ok(node(XmlEntity::from(("lt", "<", document.clone())))),
        "gt" => Ok(node(XmlEntity::from(("gt", ">", document.clone())))),
        "amp" => Ok(node(XmlEntity::from(("amp", "&", document.clone())))),
        "apos" => Ok(node(XmlEntity::from(("apos", "'", document.clone())))),
        "quot" => Ok(node(XmlEntity::from(("quot", "\"", document.clone())))),
        _ => Err(error::Error::NotFoundReference(name.to_string())),
    }
}

fn equal_qname(a: xml_nom::model::QName, b: xml_nom::model::QName) -> bool {
    match a {
        xml_nom::model::QName::Prefixed(a) => match b {
            xml_nom::model::QName::Prefixed(b) => {
                a.prefix == b.prefix && a.local_part == b.local_part
            }
            xml_nom::model::QName::Unprefixed(_) => false,
        },
        xml_nom::model::QName::Unprefixed(a) => match b {
            xml_nom::model::QName::Prefixed(_) => false,
            xml_nom::model::QName::Unprefixed(b) => a == b,
        },
    }
}

fn external_id(id: &parser::ExternalId) -> (String, Option<String>) {
    match id {
        parser::ExternalId::Public(p, s) => (s.to_string(), Some(p.to_string())),
        parser::ExternalId::System(s) => (s.to_string(), None),
    }
}

fn node<T>(value: T) -> XmlNode<T> {
    Rc::new(RefCell::new(value))
}

fn normalize_ws(value: &str) -> String {
    let mut v = value.to_string();
    unsafe {
        v = v.replace(char::from_u32_unchecked(0x20), " ");
        v = v.replace(char::from_u32_unchecked(0x0D), " ");
        v = v.replace(char::from_u32_unchecked(0x0A), " ");
        v = v.replace(char::from_u32_unchecked(0x09), " ");
    }
    v
}

fn notation(document: &XmlNode<XmlDocument>, name: &str) -> Value<Option<XmlNode<XmlNotation>>> {
    match document.borrow().notations() {
        Some(notations) => {
            let mut matches = notations
                .iter()
                .filter(|n| n.borrow().name() == name)
                .collect::<Vec<XmlNode<XmlNotation>>>();
            match matches.len() {
                1 => Value::V(Some(matches.remove(0))),
                _ => Value::V(None),
            }
        }
        _ => Value::V(None),
    }
}

fn qname(name: &xml_nom::model::QName<'_>) -> (String, Option<String>) {
    match name {
        xml_nom::model::QName::Prefixed(n) => {
            (n.local_part.to_string(), Some(n.prefix.to_string()))
        }
        xml_nom::model::QName::Unprefixed(n) => (n.to_string(), None),
    }
}

fn str_from_name(name: &str, owner: &XmlNode<XmlDocument>) -> error::Result<String> {
    let entity = entity(owner, name)?;
    let mut parsed = String::new();
    for value in entity.borrow().values().unwrap_or_default() {
        match &value {
            XmlEntityValue::Parameter(_) => {
                unimplemented!("Not support parameter entity reference.")
            }
            XmlEntityValue::Reference(v) => {
                let v = str_from_reference(v, owner)?;
                parsed.push_str(v.as_str());
            }
            XmlEntityValue::Text(v) => parsed.push_str(v),
        }
    }
    Ok(parsed)
}

fn str_from_reference(value: &XmlReference, owner: &XmlNode<XmlDocument>) -> error::Result<String> {
    match value {
        XmlReference::Character(v, r) => match r {
            10 => char_from_char10(v).map(|v| v.to_string()),
            16 => char_from_char16(v).map(|v| v.to_string()),
            _ => unreachable!(),
        },
        XmlReference::Entity(v) => str_from_name(v, owner),
    }
}

fn xml_encoding(value: &parser::Document) -> String {
    value
        .prolog
        .declaration_xml
        .as_ref()
        .and_then(|v| v.encoding)
        .unwrap_or_default()
        .to_string()
}

fn xml_standalone(value: &parser::Document) -> Option<bool> {
    value
        .prolog
        .declaration_xml
        .as_ref()
        .and_then(|v| v.standalone)
}

fn xml_version(value: &parser::Document) -> Option<String> {
    value
        .prolog
        .declaration_xml
        .as_ref()
        .map(|v| v.version.to_string())
}

// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_min() {
        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();

        // Document
        let children = doc.borrow().children();
        assert_eq!(1, children.iter().len());

        let document_element = doc.borrow().document_element().unwrap();
        assert_eq!("root", document_element.borrow().local_name());

        let notations = doc.borrow().notations().unwrap();
        assert_eq!(0, notations.iter().len());

        let unparsed_entities = doc.borrow().unparsed_entities();
        assert_eq!(0, unparsed_entities.iter().len());

        let base_uri = doc.borrow().base_uri().to_string();
        assert_eq!("", base_uri);

        let character_encoding_scheme = doc.borrow().character_encoding_scheme().to_string();
        assert_eq!("", character_encoding_scheme);

        let standalone = doc.borrow().standalone();
        assert_eq!(None, standalone);

        let version = doc.borrow().version().map(|v| v.to_string());
        assert_eq!(None, version);

        let all_declarations_processed = doc.borrow().all_declarations_processed();
        assert!(all_declarations_processed);

        //PartialEq
        assert_eq!(doc, doc);
    }

    #[test]
    fn test_document_max() {
        let (rest, tree) = xml_parser::document("<?xml version='1.1' encoding='utf-8' standalone='yes'?><!DOCTYPE root [<!NOTATION aaa SYSTEM 'bbb'><!ENTITY ccc SYSTEM 'ddd' NDATA eee>]><root />").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();

        // Document
        let children = doc.borrow().children();
        assert_eq!(2, children.iter().len());

        let document_element = doc.borrow().document_element().unwrap();
        assert_eq!("root", document_element.borrow().local_name());

        let notations = doc.borrow().notations().unwrap();
        assert_eq!(1, notations.iter().len());

        let unparsed_entities = doc.borrow().unparsed_entities();
        assert_eq!(1, unparsed_entities.iter().len());

        let base_uri = doc.borrow().base_uri().to_string();
        assert_eq!("", base_uri);

        let character_encoding_scheme = doc.borrow().character_encoding_scheme().to_string();
        assert_eq!("utf-8", character_encoding_scheme);

        let standalone = doc.borrow().standalone();
        assert_eq!(Some(true), standalone);

        let version = doc.borrow().version().map(|v| v.to_string());
        assert_eq!(Some("1.1"), version.as_deref());

        let all_declarations_processed = doc.borrow().all_declarations_processed();
        assert!(all_declarations_processed);

        //PartialEq
        assert_eq!(doc, doc);
    }

    #[test]
    fn test_document_children() {
        let (rest, tree) = xml_parser::document(
            "<!--c1--><?p1?><!DOCTYPE root><!--c2--><?p2?><root /><!--c3--><?p3?>",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();

        // Document[children]
        let children = doc.borrow().children();
        assert_eq!(8, children.iter().len());

        let c1 = children.get(0).unwrap().as_comment().unwrap();
        assert_eq!("c1", c1.borrow().comment());

        let p1 = children.get(1).unwrap().as_pi().unwrap();
        assert_eq!("p1", p1.borrow().target());

        let doc_type = children.get(2).unwrap().as_document_type().unwrap();
        assert_eq!(None, doc_type.borrow().system_identifier());

        let c2 = children.get(3).unwrap().as_comment().unwrap();
        assert_eq!("c2", c2.borrow().comment());

        let p2 = children.get(4).unwrap().as_pi().unwrap();
        assert_eq!("p2", p2.borrow().target());

        let root = children.get(5).unwrap().as_element().unwrap();
        assert_eq!("root", root.borrow().local_name());

        let c3 = children.get(6).unwrap().as_comment().unwrap();
        assert_eq!("c3", c3.borrow().comment());

        let p3 = children.get(7).unwrap().as_pi().unwrap();
        assert_eq!("p3", p3.borrow().target());

        //PartialEq
        assert_eq!(doc, doc);
    }

    #[test]
    fn test_document_notations() {
        let (rest, tree) = xml_parser::document(
            "<!DOCTYPE root [<!NOTATION aaa SYSTEM 'bbb'><!NOTATION aaa SYSTEM 'ccc'>]><root />",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();

        // Document[notations]
        let notations = doc.borrow().notations();
        assert!(notations.is_none());

        //PartialEq
        assert_eq!(doc, doc);
    }

    #[test]
    fn test_element_min() {
        let (rest, tree) = xml_parser::document("<root />").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();

        // Element
        let namespace_name = root.borrow().namespace_name();
        assert!(namespace_name.unwrap().is_none());

        let local_name = root.borrow().local_name().to_string();
        assert_eq!("root", local_name);

        let prefix = root.borrow().prefix().map(|v| v.to_string());
        assert!(prefix.is_none());

        let children = root.borrow().children();
        assert_eq!(0, children.iter().len());

        let attributes = root.borrow().attributes();
        assert_eq!(0, attributes.iter().len());

        let namespace_attributes = root.borrow().namespace_attributes();
        assert_eq!(0, namespace_attributes.iter().len());

        let in_scope_namespace = root.borrow().in_scope_namespace().unwrap();
        assert_eq!(1, in_scope_namespace.iter().len());

        let base_uri = root.borrow().base_uri().to_string();
        assert_eq!("", base_uri);

        let parent = root.borrow().parent().unwrap();
        assert!(parent.as_document().is_some());

        //PartialEq
        assert_eq!(root, root);
    }

    #[test]
    fn test_element_max() {
        let (rest, tree) =
            xml_parser::document("<c:root a='b' xmlns:c='http://test/c'><child /></c:root>")
                .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();

        // Element
        let namespace_name = root.borrow().namespace_name();
        assert_eq!(Some("http://test/c"), namespace_name.unwrap().as_deref());

        let local_name = root.borrow().local_name().to_string();
        assert_eq!("root", local_name);

        let prefix = root.borrow().prefix().map(|v| v.to_string());
        assert_eq!(Some("c"), prefix.as_deref());

        let children = root.borrow().children();
        assert_eq!(1, children.iter().len());

        let attributes = root.borrow().attributes();
        assert_eq!(1, attributes.iter().len());

        let namespace_attributes = root.borrow().namespace_attributes();
        assert_eq!(1, namespace_attributes.iter().len());

        let in_scope_namespace = root.borrow().in_scope_namespace().unwrap();
        assert_eq!(2, in_scope_namespace.iter().len());

        let base_uri = root.borrow().base_uri().to_string();
        assert_eq!("", base_uri);

        let parent = root.borrow().parent().unwrap();
        assert!(parent.as_document().is_some());

        // Child
        let child = children.get(0).unwrap().as_element().unwrap();

        let namespace_name = child.borrow().namespace_name();
        assert!(namespace_name.unwrap().is_none());

        let local_name = child.borrow().local_name().to_string();
        assert_eq!("child", local_name);

        let prefix = child.borrow().prefix().map(|v| v.to_string());
        assert!(prefix.is_none());

        let children = child.borrow().children();
        assert_eq!(0, children.iter().len());

        let attributes = child.borrow().attributes();
        assert_eq!(0, attributes.iter().len());

        let namespace_attributes = child.borrow().namespace_attributes();
        assert_eq!(0, namespace_attributes.iter().len());

        let in_scope_namespace = child.borrow().in_scope_namespace().unwrap();
        assert_eq!(2, in_scope_namespace.iter().len());

        let base_uri = child.borrow().base_uri().to_string();
        assert_eq!("", base_uri);

        let parent = child.borrow().parent().unwrap();
        assert!(parent.as_element().is_some());

        //PartialEq
        assert_eq!(root, root);
    }

    #[test]
    fn test_element_children() {
        let (rest, tree) = xml_parser::document(
            "<root>t1&amp;<![CDATA[d1]]><child /><?p1?>&gt;<!--c1-->&#x3042;t2</root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();

        // Element[children]
        let children = root.borrow().children();
        assert_eq!(9, children.iter().len());

        let t1 = children.get(0).unwrap().as_text().unwrap();
        assert_eq!("t1", t1.borrow().character_code());

        let amp = children.get(1).unwrap().as_unexpanded().unwrap();
        assert_eq!("amp", amp.borrow().name());

        let d1 = children.get(2).unwrap().as_cdata().unwrap();
        assert_eq!("d1", d1.borrow().character_code());

        let child = children.get(3).unwrap().as_element().unwrap();
        assert_eq!("child", child.borrow().local_name());

        let p1 = children.get(4).unwrap().as_pi().unwrap();
        assert_eq!("p1", p1.borrow().target());

        let gt = children.get(5).unwrap().as_unexpanded().unwrap();
        assert_eq!("gt", gt.borrow().name());

        let c1 = children.get(6).unwrap().as_comment().unwrap();
        assert_eq!("c1", c1.borrow().comment());

        let a = children.get(7).unwrap().as_char_reference().unwrap();
        assert_eq!("あ", a.borrow().character_code());

        let t2 = children.get(8).unwrap().as_text().unwrap();
        assert_eq!("t2", t2.borrow().character_code());

        //PartialEq
        assert_eq!(root, root);
    }

    #[test]
    fn test_element_attribute() {
        let (rest, tree) = xml_parser::document(
            "<root a='1' xmlns='' xmlns:b='' d:c='2' xmlns:d='http://test/d' />",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();

        // Element[attributes]
        let attributes = root.borrow().attributes();
        assert_eq!(2, attributes.iter().len());

        let mut i = attributes.iter();
        let a = i.next().unwrap();
        assert_eq!("a", a.borrow().local_name());
        assert_eq!(None, a.borrow().prefix());

        let c = i.next().unwrap();
        assert_eq!("c", c.borrow().local_name());
        assert_eq!(Some("d"), c.borrow().prefix());

        //PartialEq
        assert_eq!(root, root);
    }

    #[test]
    fn test_element_namespace_attribute() {
        let (rest, tree) = xml_parser::document(
            "<root a='1' xmlns='' xmlns:b='' d:c='2' xmlns:d='http://test/d' />",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();

        // Element[namespace attributes]
        let attributes = root.borrow().namespace_attributes();
        assert_eq!(3, attributes.iter().len());

        let mut i = attributes.iter();
        let s = i.next().unwrap();
        assert_eq!("xmlns", s.borrow().local_name());
        assert_eq!(None, s.borrow().prefix());
        assert_eq!(
            Some("http://www.w3.org/2000/xmlns/".to_string()),
            s.borrow()
                .namespace_name()
                .unwrap()
                .map(|v| v.value().to_string())
        );

        let b = i.next().unwrap();
        assert_eq!("b", b.borrow().local_name());
        assert_eq!(Some("xmlns"), b.borrow().prefix());
        assert_eq!(
            Some("http://www.w3.org/2000/xmlns/".to_string()),
            b.borrow()
                .namespace_name()
                .unwrap()
                .map(|v| v.value().to_string())
        );

        let d = i.next().unwrap();
        assert_eq!("d", d.borrow().local_name());
        assert_eq!(Some("xmlns"), d.borrow().prefix());
        assert_eq!(
            Some("http://www.w3.org/2000/xmlns/".to_string()),
            d.borrow()
                .namespace_name()
                .unwrap()
                .map(|v| v.value().to_string())
        );

        //PartialEq
        assert_eq!(root, root);
    }

    #[test]
    fn test_element_in_scope_namespaces() {
        let (rest, tree) = xml_parser::document(
            "<root a='1' xmlns='' xmlns:b='' d:c='2' xmlns:d='http://test/d' />",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();

        // Element[in scope namespaces]
        let in_scope_namespace = root.borrow().in_scope_namespace().unwrap();
        assert_eq!(2, in_scope_namespace.iter().len());

        let mut i = in_scope_namespace.iter();
        let d = i.next().unwrap();
        assert_eq!(Some("d"), d.borrow().prefix());
        assert_eq!("http://test/d", d.borrow().namespace_name());

        let s = i.next().unwrap();
        assert_eq!(Some("xml"), s.borrow().prefix());
        assert_eq!(
            "http://www.w3.org/XML/1998/namespace",
            s.borrow().namespace_name()
        );

        //PartialEq
        assert_eq!(root, root);
    }

    #[test]
    fn test_attribute_min() {
        let (rest, tree) = xml_parser::document("<root a='1' />").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let attr = root.borrow().attributes().iter().next().unwrap();

        // Attribute
        let namespace_name = attr.borrow().namespace_name();
        assert!(namespace_name.unwrap().is_none());

        let local_name = attr.borrow().local_name().to_string();
        assert_eq!("a", local_name);

        let prefix = attr.borrow().prefix().map(|v| v.to_string());
        assert!(prefix.is_none());

        let value = attr.borrow().normalized_value().unwrap();
        assert_eq!("1", value);

        let specified = attr.borrow().specified();
        assert!(specified);

        if let Value::V(attribute_type) = attr.borrow().attribute_type() {
            assert!(attribute_type.is_none());
        } else {
            unreachable!();
        }

        if let Value::V(references) = attr.borrow().references() {
            assert!(references.is_none());
        } else {
            unreachable!();
        }

        let parent = attr.borrow().owner_element().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(attr, attr);
    }

    #[test]
    fn test_attribute_max() {
        let (rest, tree) = xml_parser::document("<!DOCTYPE root [<!ATTLIST root b:a ID #REQUIRED>]><root b:a=' 1 ' xmlns:b='http://test/b'/>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let attr = root.borrow().attributes().iter().next().unwrap();

        // Attribute
        let namespace_name = attr.borrow().namespace_name();
        assert_eq!(Some("http://test/b"), namespace_name.unwrap().as_deref());

        let local_name = attr.borrow().local_name().to_string();
        assert_eq!("a", local_name);

        let prefix = attr.borrow().prefix().map(|v| v.to_string());
        assert_eq!(Some("b"), prefix.as_deref());

        let value = attr.borrow().normalized_value().unwrap();
        assert_eq!(" 1 ", value);

        let specified = attr.borrow().specified();
        assert!(specified);

        if let Value::V(attribute_type) = attr.borrow().attribute_type() {
            assert_eq!(Some(XmlDeclarationAttType::Id), attribute_type);
        } else {
            unreachable!();
        }

        if let Value::V(references) = attr.borrow().references() {
            assert!(references.is_none());
        } else {
            unreachable!();
        }

        let parent = attr.borrow().owner_element().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(attr, attr);
    }

    #[test]
    fn test_attribute_normalized_value() {
        let (rest, tree) = xml_parser::document("<root a='a\n&amp;b&#x3042;\tc' />").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let attr = root.borrow().attributes().iter().next().unwrap();

        // Attribute[normalized value]
        let value = attr.borrow().normalized_value().unwrap();
        assert_eq!("a &bあ c", value);

        //PartialEq
        assert_eq!(attr, attr);
    }

    #[test]
    fn test_pi_min() {
        let (rest, tree) = xml_parser::document("<root><?p1?></root>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let pi = root.borrow().children().get(0).unwrap().as_pi().unwrap();

        // Processing Instruction
        let target = pi.borrow().target().to_string();
        assert_eq!("p1", target);

        let content = pi.borrow().content().to_string();
        assert_eq!("", content);

        let base_uri = pi.borrow().base_uri().to_string();
        assert_eq!("", base_uri);

        if let Value::V(notation) = pi.borrow().notation() {
            assert!(notation.is_none());
        } else {
            unreachable!();
        }

        let parent = pi.borrow().parent().as_element().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(pi, pi);
    }

    #[test]
    fn test_pi_max() {
        let (rest, tree) = xml_parser::document(
            "<!DOCTYPE root [<!NOTATION p1 SYSTEM 'bbb'>]><root><?p1 aaa?></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let pi = root.borrow().children().get(0).unwrap().as_pi().unwrap();

        // Processing Instruction
        let target = pi.borrow().target().to_string();
        assert_eq!("p1", target);

        let content = pi.borrow().content().to_string();
        assert_eq!("aaa", content);

        let base_uri = pi.borrow().base_uri().to_string();
        assert_eq!("", base_uri);

        if let Value::V(notation) = pi.borrow().notation() {
            assert_eq!("p1", notation.unwrap().borrow().name());
        } else {
            unreachable!();
        }

        let parent = pi.borrow().parent().as_element().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(pi, pi);
    }

    #[test]
    fn test_pi_notation() {
        let (rest, tree) = xml_parser::document(
            "<!DOCTYPE root [<!NOTATION p1 SYSTEM 'bbb'><!NOTATION p1 SYSTEM 'bbb'>]><root><?p1 aaa?></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let pi = root.borrow().children().get(0).unwrap().as_pi().unwrap();

        // Processing Instruction[notation]
        if let Value::V(notation) = pi.borrow().notation() {
            assert!(notation.is_none());
        } else {
            unreachable!();
        }

        //PartialEq
        assert_eq!(pi, pi);
    }

    #[test]
    fn test_unexpanded_min() {
        let (rest, tree) = xml_parser::document("<root>&amp;</root>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let amp = root
            .borrow()
            .children()
            .get(0)
            .unwrap()
            .as_unexpanded()
            .unwrap();

        // Unexpanded Entity Reference
        let name = amp.borrow().name().to_string();
        assert_eq!("amp", name);

        if let Value::V(system_identifier) = amp.borrow().system_identifier() {
            assert!(system_identifier.is_none());
        } else {
            unreachable!();
        }

        if let Value::V(public_identifier) = amp.borrow().public_identifier() {
            assert!(public_identifier.is_none());
        } else {
            unreachable!();
        }

        let declaration_base_uri = amp.borrow().declaration_base_uri().to_string();
        assert_eq!("", declaration_base_uri);

        let parent = amp.borrow().parent();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(amp, amp);
    }

    #[test]
    fn test_unexpanded_max() {
        let (rest, tree) = xml_parser::document(
            "<!DOCTYPE root [<!ENTITY aaa PUBLIC 'bbb' 'ccc'>]><root>&aaa;</root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let amp = root
            .borrow()
            .children()
            .get(0)
            .unwrap()
            .as_unexpanded()
            .unwrap();

        // Unexpanded Entity Reference
        let name = amp.borrow().name().to_string();
        assert_eq!("aaa", name);

        if let Value::V(system_identifier) = amp.borrow().system_identifier() {
            assert_eq!(Some("ccc"), system_identifier);
        } else {
            unreachable!();
        }

        if let Value::V(public_identifier) = amp.borrow().public_identifier() {
            assert_eq!(Some("bbb"), public_identifier);
        } else {
            unreachable!();
        }

        let declaration_base_uri = amp.borrow().declaration_base_uri().to_string();
        assert_eq!("", declaration_base_uri);

        let parent = amp.borrow().parent();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(amp, amp);
    }

    #[test]
    fn test_cdata() {
        let (rest, tree) = xml_parser::document("<root><![CDATA[aaa]]></root>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let cdata = root.borrow().children().get(0).unwrap().as_cdata().unwrap();

        // Character
        let character_code = cdata.borrow().character_code().to_string();
        assert_eq!("aaa", character_code);

        if let Value::V(element_content_whitespace) = cdata.borrow().element_content_whitespace() {
            assert!(element_content_whitespace.is_none());
        } else {
            unreachable!();
        }

        let parent = cdata.borrow().parent().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(cdata, cdata);
    }

    #[test]
    fn char_reference_10() {
        let (rest, tree) = xml_parser::document("<root>&#12354;</root>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let char_ref = root
            .borrow()
            .children()
            .get(0)
            .unwrap()
            .as_char_reference()
            .unwrap();

        // Character
        let character_code = char_ref.borrow().character_code().to_string();
        assert_eq!("あ", character_code);

        if let Value::V(element_content_whitespace) = char_ref.borrow().element_content_whitespace()
        {
            assert!(element_content_whitespace.is_none());
        } else {
            unreachable!();
        }

        let parent = char_ref.borrow().parent().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(char_ref, char_ref);
    }

    #[test]
    fn char_reference_16() {
        let (rest, tree) = xml_parser::document("<root>&#x3042;</root>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let char_ref = root
            .borrow()
            .children()
            .get(0)
            .unwrap()
            .as_char_reference()
            .unwrap();

        // Character
        let character_code = char_ref.borrow().character_code().to_string();
        assert_eq!("あ", character_code);

        if let Value::V(element_content_whitespace) = char_ref.borrow().element_content_whitespace()
        {
            assert!(element_content_whitespace.is_none());
        } else {
            unreachable!();
        }

        let parent = char_ref.borrow().parent().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(char_ref, char_ref);
    }

    #[test]
    fn test_text() {
        let (rest, tree) = xml_parser::document("<root>aaa</root>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let text = root.borrow().children().get(0).unwrap().as_text().unwrap();

        // Character
        let character_code = text.borrow().character_code().to_string();
        assert_eq!("aaa", character_code);

        if let Value::V(element_content_whitespace) = text.borrow().element_content_whitespace() {
            assert!(element_content_whitespace.is_none());
        } else {
            unreachable!();
        }

        let parent = text.borrow().parent().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(text, text);
    }

    #[test]
    fn test_comment() {
        let (rest, tree) = xml_parser::document("<root><!--aaa--></root>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let comment = root
            .borrow()
            .children()
            .get(0)
            .unwrap()
            .as_comment()
            .unwrap();

        // Character
        let text = comment.borrow().comment().to_string();
        assert_eq!("aaa", text);

        let parent = comment.borrow().parent().unwrap().as_element().unwrap();
        assert_eq!("root", parent.borrow().local_name());

        //PartialEq
        assert_eq!(comment, comment);
    }

    #[test]
    fn test_unparsed_min() {
        let (rest, tree) =
            xml_parser::document("<!DOCTYPE root [<!ENTITY aaa SYSTEM 'bbb' NDATA ccc>]><root/>")
                .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let entity = doc.borrow().unparsed_entities().iter().next().unwrap();

        // Unparsed Entity
        let name = entity.borrow().name().to_string();
        assert_eq!("aaa", name);

        let system_identifier = entity.borrow().system_identifier().to_string();
        assert_eq!("bbb", system_identifier);

        let public_identifier = entity.borrow().public_identifier().map(|v| v.to_string());
        assert!(public_identifier.is_none());

        let notation_name = entity.borrow().notation_name().to_string();
        assert_eq!("ccc", notation_name);

        if let Value::V(notation) = entity.borrow().notation() {
            assert!(notation.is_none());
        } else {
            unreachable!();
        }

        //PartialEq
        assert_eq!(entity, entity);
    }

    #[test]
    fn test_unparsed_max() {
        let (rest, tree) =
            xml_parser::document("<!DOCTYPE root [<!ENTITY aaa PUBLIC 'bbb' 'ccc' NDATA ddd><!NOTATION ddd SYSTEM 'eee'>]><root/>")
                .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let entity = doc.borrow().unparsed_entities().iter().next().unwrap();

        // Unparsed Entity
        let name = entity.borrow().name().to_string();
        assert_eq!("aaa", name);

        let system_identifier = entity.borrow().system_identifier().to_string();
        assert_eq!("ccc", system_identifier);

        let public_identifier = entity.borrow().public_identifier().map(|v| v.to_string());
        assert_eq!(Some("bbb"), public_identifier.as_deref());

        let notation_name = entity.borrow().notation_name().to_string();
        assert_eq!("ddd", notation_name);

        if let Value::V(notation) = entity.borrow().notation() {
            assert_eq!("ddd", notation.unwrap().borrow().name());
        } else {
            unreachable!();
        }

        //PartialEq
        assert_eq!(entity, entity);
    }

    #[test]
    fn test_unparsed_notation() {
        let (rest, tree) =
            xml_parser::document("<!DOCTYPE root [<!ENTITY aaa PUBLIC 'bbb' 'ccc' NDATA ddd><!NOTATION ddd SYSTEM 'eee'><!NOTATION ddd SYSTEM 'eee'>]><root/>")
                .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let entity = doc.borrow().unparsed_entities().iter().next().unwrap();

        // Unparsed Entity[notation]
        if let Value::V(notation) = entity.borrow().notation() {
            assert!(notation.is_none());
        } else {
            unreachable!();
        }

        //PartialEq
        assert_eq!(entity, entity);
    }

    #[test]
    fn test_notation_min() {
        let (rest, tree) =
            xml_parser::document("<!DOCTYPE root [<!NOTATION aaa PUBLIC 'bbb'>]><root/>").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let notation = doc.borrow().notations().unwrap().iter().next().unwrap();

        // Notation
        let name = notation.borrow().name().to_string();
        assert_eq!("aaa", name);

        let system_identifier = notation.borrow().system_identifier().map(|v| v.to_string());
        assert!(system_identifier.is_none());

        let public_identifier = notation.borrow().public_identifier().map(|v| v.to_string());
        assert_eq!(Some("bbb"), public_identifier.as_deref());

        let declaration_base_uri = notation.borrow().declaration_base_uri().to_string();
        assert_eq!("", declaration_base_uri);

        //PartialEq
        assert_eq!(notation, notation);
    }

    #[test]
    fn test_notation_max() {
        let (rest, tree) =
            xml_parser::document("<!DOCTYPE root [<!NOTATION aaa PUBLIC 'bbb' 'ccc'>]><root/>")
                .unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let notation = doc.borrow().notations().unwrap().iter().next().unwrap();

        // Notation
        let name = notation.borrow().name().to_string();
        assert_eq!("aaa", name);

        let system_identifier = notation.borrow().system_identifier().map(|v| v.to_string());
        assert_eq!(Some("ccc"), system_identifier.as_deref());

        let public_identifier = notation.borrow().public_identifier().map(|v| v.to_string());
        assert_eq!(Some("bbb"), public_identifier.as_deref());

        let declaration_base_uri = notation.borrow().declaration_base_uri().to_string();
        assert_eq!("", declaration_base_uri);

        //PartialEq
        assert_eq!(notation, notation);
    }

    #[test]
    fn test_namespace_min() {
        let (rest, tree) = xml_parser::document("<root xmlns='http://test' />").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let ns = root
            .borrow()
            .in_scope_namespace()
            .unwrap()
            .iter()
            .next()
            .unwrap();

        // Namespace
        let prefix = ns.borrow().prefix().map(|v| v.to_string());
        assert!(prefix.is_none());

        let namespace_name = ns.borrow().namespace_name().to_string();
        assert_eq!("http://test", namespace_name);

        //PartialEq
        assert_eq!(ns, ns);
    }

    #[test]
    fn test_namespace_max() {
        let (rest, tree) = xml_parser::document("<root xmlns:aaa='http://test/aaa' />").unwrap();
        assert_eq!("", rest);

        let doc = XmlDocument::new(&tree).unwrap();
        let root = doc.borrow().document_element().unwrap();
        let ns = root
            .borrow()
            .in_scope_namespace()
            .unwrap()
            .iter()
            .next()
            .unwrap();

        // Namespace
        let prefix = ns.borrow().prefix().map(|v| v.to_string());
        assert_eq!(Some("aaa"), prefix.as_deref());

        let namespace_name = ns.borrow().namespace_name().to_string();
        assert_eq!("http://test/aaa", namespace_name);

        //PartialEq
        assert_eq!(ns, ns);
    }
}
