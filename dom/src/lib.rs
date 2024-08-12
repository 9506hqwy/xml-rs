pub mod error;

use std::fmt;
use std::iter::Iterator;
use xml_info as info;
use xml_info::{
    Attribute as InfoAttribute, Character as InfoCharacter, Comment as InfoComment,
    Document as InfoDocument, DocumentTypeDeclaration as InfoDocumentTypeDeclaration,
    Element as InfoElement, HasQName as InfoHasQName, Namespace as InfoNamespace,
    Notation as InfoNotation, ProcessingInstruction as InfoProcessingInstruction,
    Sortable as InfoSortable,
};

// TODO: read only.
// TODO: re-implement ResolvedText

pub type ExpandedName = (String, Option<String>, Option<String>);

// -----------------------------------------------------------------------------------------------

pub trait DomImplementation {
    fn has_feature(&self, feature: &str, version: Option<&str>) -> bool;
}

// -----------------------------------------------------------------------------------------------

pub trait DocumentFragment: Node {}

// -----------------------------------------------------------------------------------------------

pub trait Document: Node {
    fn doc_type(&self) -> Option<XmlDocumentType>;

    fn implementation(&self) -> XmlDomImplementation;

    fn document_element(&self) -> error::Result<XmlElement>;

    fn get_elements_by_tag_name(&self, tag_name: &str) -> error::Result<XmlNodeList>;
}

pub trait DocumentMut: Document + NodeMut {
    fn create_element(&self, tag_name: &str) -> error::Result<XmlElement>;

    fn create_document_fragment(&self) -> XmlDocumentFragment;

    fn create_text_node(&self, data: &str) -> error::Result<XmlText>;

    fn create_comment(&self, data: &str) -> error::Result<XmlComment>;

    fn create_cdata_section(&self, data: &str) -> error::Result<XmlCDataSection>;

    fn create_processing_instruction(
        &self,
        target: &str,
        data: &str,
    ) -> error::Result<XmlProcessingInstruction>;

    fn create_attribute(&self, name: &str) -> error::Result<XmlAttr>;

    fn create_entity_reference(&self, name: &str) -> error::Result<XmlEntityReference>;
}

// -----------------------------------------------------------------------------------------------

pub trait Node {
    fn node_name(&self) -> String;

    fn node_value(&self) -> error::Result<Option<String>>;

    fn node_type(&self) -> NodeType;

    fn parent_node(&self) -> Option<XmlNode>;

    fn child_nodes(&self) -> XmlNodeList;

    fn first_child(&self) -> Option<XmlNode>;

    fn last_child(&self) -> Option<XmlNode>;

    fn previous_sibling(&self) -> Option<XmlNode>;

    fn next_sibling(&self) -> Option<XmlNode>;

    fn attributes(&self) -> Option<XmlNamedNodeMap>;

    fn owner_document(&self) -> Option<XmlDocument>;

    fn has_child(&self) -> bool;
}

pub trait NodeMut {
    fn set_node_value(&mut self, value: &str) -> error::Result<()>;

    fn insert_before(
        &mut self,
        new_child: XmlNode,
        ref_child: Option<&XmlNode>,
    ) -> error::Result<XmlNode>;

    fn replace_child(&mut self, new_child: XmlNode, old_child: &XmlNode) -> error::Result<XmlNode> {
        self.insert_before(new_child, Some(old_child))?;
        self.remove_child(old_child)
    }

    fn remove_child(&mut self, old_child: &XmlNode) -> error::Result<XmlNode>;

    fn append_child(&mut self, new_child: XmlNode) -> error::Result<XmlNode> {
        self.insert_before(new_child, None)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum NodeType {
    Element = 1,
    Attribute = 2,
    Text = 3,
    CData = 4,
    EntityReference = 5,
    Entity = 6,
    PI = 7,
    Comment = 8,
    Document = 9,
    DocumentType = 10,
    DocumentFragment = 11,
    Notation = 12,
}

// -----------------------------------------------------------------------------------------------

pub trait NodeList {
    fn item(&self, index: usize) -> Option<XmlNode>;

    fn length(&self) -> usize;
}

// -----------------------------------------------------------------------------------------------

pub trait NamedNodeMap {
    fn get_named_item(&self, name: &str) -> Option<XmlNode>;

    fn item(&self, index: usize) -> Option<XmlNode>;

    fn length(&self) -> usize;
}

pub trait NamedNodeMapMut: NamedNodeMap {
    fn set_named_item(&mut self, arg: XmlNode) -> error::Result<XmlNode>;

    fn remove_named_item(&mut self, name: &str) -> error::Result<XmlNode>;
}

// -----------------------------------------------------------------------------------------------

pub trait CharacterData: Node {
    fn data(&self) -> error::Result<String>;

    fn length(&self) -> usize;

    fn substring_data(&self, offset: usize, count: usize) -> String;
}

pub trait CharacterDataMut: CharacterData + NodeMut {
    fn set_data(&mut self, data: &str) -> error::Result<()> {
        self.replace_data(0, self.length(), data)
    }

    fn append_data(&mut self, arg: &str) -> error::Result<()> {
        self.insert_data(self.length(), arg)
    }

    fn insert_data(&mut self, offset: usize, arg: &str) -> error::Result<()>;

    fn delete_data(&mut self, offset: usize, count: usize) -> error::Result<()>;

    fn replace_data(&mut self, offset: usize, count: usize, arg: &str) -> error::Result<()> {
        self.delete_data(offset, count)?;
        self.insert_data(offset, arg)
    }
}

// -----------------------------------------------------------------------------------------------

pub trait Attr: Node {
    fn name(&self) -> String;

    fn specified(&self) -> bool;

    fn value(&self) -> error::Result<String>;
}

pub trait AttrMut: Attr + NodeMut {
    fn set_value(&mut self, value: &str) -> error::Result<()> {
        self.set_node_value(value)
    }
}

// -----------------------------------------------------------------------------------------------

pub trait Element: Node {
    fn tag_name(&self) -> String;

    fn get_attribute(&self, name: &str) -> error::Result<String>;

    fn get_attribute_node(&self, name: &str) -> Option<XmlAttr>;

    fn get_elements_by_tag_name(&self, tag_name: &str) -> XmlNodeList;
}

pub trait ElementMut: Element + NodeMut {
    fn set_attribute(&mut self, name: &str, value: &str) -> error::Result<()>;

    fn remove_attribute(&mut self, name: &str) -> error::Result<()>;

    fn set_attribute_node(&mut self, new_attr: XmlAttr) -> error::Result<Option<XmlAttr>>;

    fn remove_attribute_node(&mut self, old_attr: XmlAttr) -> error::Result<Option<XmlAttr>> {
        if let Some(attr) = self.get_attribute_node(old_attr.name().as_str()) {
            self.remove_attribute(old_attr.name().as_str())?;
            Ok(Some(attr))
        } else {
            Ok(None)
        }
    }

    fn normalize(&mut self);
}

// -----------------------------------------------------------------------------------------------

pub trait Text: CharacterData {}

pub trait TextMut: CharacterDataMut {
    fn split_text(&mut self, offset: usize) -> error::Result<XmlResolvedText>;
}

// -----------------------------------------------------------------------------------------------

pub trait Comment: CharacterData {}

pub trait CommentMut: CharacterDataMut {}

// -----------------------------------------------------------------------------------------------

pub trait CDataSection: Text {}

pub trait CDataSectionMut: TextMut {}

// -----------------------------------------------------------------------------------------------

pub trait DocumentType: Node {
    fn name(&self) -> String;

    fn entities(&self) -> XmlNamedNodeMap;

    fn notations(&self) -> XmlNamedNodeMap;
}

// -----------------------------------------------------------------------------------------------

pub trait Notation: Node {
    fn public_id(&self) -> Option<String>;

    fn system_id(&self) -> Option<String>;
}

// -----------------------------------------------------------------------------------------------

pub trait Entity: Node {
    fn public_id(&self) -> Option<String>;

    fn system_id(&self) -> Option<String>;

    fn notation_name(&self) -> Option<String>;
}

// -----------------------------------------------------------------------------------------------

pub trait EntityReference: Node {}

// -----------------------------------------------------------------------------------------------

pub trait ProcessingInstruction: Node {
    fn target(&self) -> String;

    fn data(&self) -> String;
}

pub trait ProcessingInstructionMut: ProcessingInstruction + NodeMut {
    fn set_data(&mut self, data: &str) -> error::Result<()>;
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum XmlNode {
    Element(XmlElement),
    Attribute(XmlAttr),
    Text(XmlText),
    CData(XmlCDataSection),
    EntityReference(XmlEntityReference),
    Entity(XmlEntity),
    PI(XmlProcessingInstruction),
    Comment(XmlComment),
    Document(XmlDocument),
    DocumentType(XmlDocumentType),
    DocumentFragment(XmlDocumentFragment),
    Notation(XmlNotation),
    Namespace(XmlNamespace),
    ResolvedText(XmlResolvedText),
}

impl Node for XmlNode {
    fn node_name(&self) -> String {
        match self {
            XmlNode::Element(v) => v.node_name(),
            XmlNode::Attribute(v) => v.node_name(),
            XmlNode::Text(v) => v.node_name(),
            XmlNode::CData(v) => v.node_name(),
            XmlNode::EntityReference(v) => v.node_name(),
            XmlNode::Entity(v) => v.node_name(),
            XmlNode::PI(v) => v.node_name(),
            XmlNode::Comment(v) => v.node_name(),
            XmlNode::Document(v) => v.node_name(),
            XmlNode::DocumentType(v) => v.node_name(),
            XmlNode::DocumentFragment(v) => v.node_name(),
            XmlNode::Notation(v) => v.node_name(),
            XmlNode::Namespace(v) => v.node_name(),
            XmlNode::ResolvedText(v) => v.node_name(),
        }
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        match self {
            XmlNode::Element(v) => v.node_value(),
            XmlNode::Attribute(v) => v.node_value(),
            XmlNode::Text(v) => v.node_value(),
            XmlNode::CData(v) => v.node_value(),
            XmlNode::EntityReference(v) => v.node_value(),
            XmlNode::Entity(v) => v.node_value(),
            XmlNode::PI(v) => v.node_value(),
            XmlNode::Comment(v) => v.node_value(),
            XmlNode::Document(v) => v.node_value(),
            XmlNode::DocumentType(v) => v.node_value(),
            XmlNode::DocumentFragment(v) => v.node_value(),
            XmlNode::Notation(v) => v.node_value(),
            XmlNode::Namespace(v) => v.node_value(),
            XmlNode::ResolvedText(v) => v.node_value(),
        }
    }

    fn node_type(&self) -> NodeType {
        match self {
            XmlNode::Element(v) => v.node_type(),
            XmlNode::Attribute(v) => v.node_type(),
            XmlNode::Text(v) => v.node_type(),
            XmlNode::CData(v) => v.node_type(),
            XmlNode::EntityReference(v) => v.node_type(),
            XmlNode::Entity(v) => v.node_type(),
            XmlNode::PI(v) => v.node_type(),
            XmlNode::Comment(v) => v.node_type(),
            XmlNode::Document(v) => v.node_type(),
            XmlNode::DocumentType(v) => v.node_type(),
            XmlNode::DocumentFragment(v) => v.node_type(),
            XmlNode::Notation(v) => v.node_type(),
            XmlNode::Namespace(v) => v.node_type(),
            XmlNode::ResolvedText(v) => v.node_type(),
        }
    }

    fn parent_node(&self) -> Option<XmlNode> {
        match self {
            XmlNode::Element(v) => v.parent_node(),
            XmlNode::Attribute(v) => v.parent_node(),
            XmlNode::Text(v) => v.parent_node(),
            XmlNode::CData(v) => v.parent_node(),
            XmlNode::EntityReference(v) => v.parent_node(),
            XmlNode::Entity(v) => v.parent_node(),
            XmlNode::PI(v) => v.parent_node(),
            XmlNode::Comment(v) => v.parent_node(),
            XmlNode::Document(v) => v.parent_node(),
            XmlNode::DocumentType(v) => v.parent_node(),
            XmlNode::DocumentFragment(v) => v.parent_node(),
            XmlNode::Notation(v) => v.parent_node(),
            XmlNode::Namespace(v) => v.parent_node(),
            XmlNode::ResolvedText(v) => v.parent_node(),
        }
    }

    fn child_nodes(&self) -> XmlNodeList {
        match self {
            XmlNode::Element(v) => v.child_nodes(),
            XmlNode::Attribute(v) => v.child_nodes(),
            XmlNode::Text(v) => v.child_nodes(),
            XmlNode::CData(v) => v.child_nodes(),
            XmlNode::EntityReference(v) => v.child_nodes(),
            XmlNode::Entity(v) => v.child_nodes(),
            XmlNode::PI(v) => v.child_nodes(),
            XmlNode::Comment(v) => v.child_nodes(),
            XmlNode::Document(v) => v.child_nodes(),
            XmlNode::DocumentType(v) => v.child_nodes(),
            XmlNode::DocumentFragment(v) => v.child_nodes(),
            XmlNode::Notation(v) => v.child_nodes(),
            XmlNode::Namespace(v) => v.child_nodes(),
            XmlNode::ResolvedText(v) => v.child_nodes(),
        }
    }

    fn first_child(&self) -> Option<XmlNode> {
        match self {
            XmlNode::Element(v) => v.first_child(),
            XmlNode::Attribute(v) => v.first_child(),
            XmlNode::Text(v) => v.first_child(),
            XmlNode::CData(v) => v.first_child(),
            XmlNode::EntityReference(v) => v.first_child(),
            XmlNode::Entity(v) => v.first_child(),
            XmlNode::PI(v) => v.first_child(),
            XmlNode::Comment(v) => v.first_child(),
            XmlNode::Document(v) => v.first_child(),
            XmlNode::DocumentType(v) => v.first_child(),
            XmlNode::DocumentFragment(v) => v.first_child(),
            XmlNode::Notation(v) => v.first_child(),
            XmlNode::Namespace(v) => v.first_child(),
            XmlNode::ResolvedText(v) => v.first_child(),
        }
    }

    fn last_child(&self) -> Option<XmlNode> {
        match self {
            XmlNode::Element(v) => v.last_child(),
            XmlNode::Attribute(v) => v.last_child(),
            XmlNode::Text(v) => v.last_child(),
            XmlNode::CData(v) => v.last_child(),
            XmlNode::EntityReference(v) => v.last_child(),
            XmlNode::Entity(v) => v.last_child(),
            XmlNode::PI(v) => v.last_child(),
            XmlNode::Comment(v) => v.last_child(),
            XmlNode::Document(v) => v.last_child(),
            XmlNode::DocumentType(v) => v.last_child(),
            XmlNode::DocumentFragment(v) => v.last_child(),
            XmlNode::Notation(v) => v.last_child(),
            XmlNode::Namespace(v) => v.last_child(),
            XmlNode::ResolvedText(v) => v.last_child(),
        }
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        match self {
            XmlNode::Element(v) => v.previous_sibling(),
            XmlNode::Attribute(v) => v.previous_sibling(),
            XmlNode::Text(v) => v.previous_sibling(),
            XmlNode::CData(v) => v.previous_sibling(),
            XmlNode::EntityReference(v) => v.previous_sibling(),
            XmlNode::Entity(v) => v.previous_sibling(),
            XmlNode::PI(v) => v.previous_sibling(),
            XmlNode::Comment(v) => v.previous_sibling(),
            XmlNode::Document(v) => v.previous_sibling(),
            XmlNode::DocumentType(v) => v.previous_sibling(),
            XmlNode::DocumentFragment(v) => v.previous_sibling(),
            XmlNode::Notation(v) => v.previous_sibling(),
            XmlNode::Namespace(v) => v.previous_sibling(),
            XmlNode::ResolvedText(v) => v.previous_sibling(),
        }
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        match self {
            XmlNode::Element(v) => v.next_sibling(),
            XmlNode::Attribute(v) => v.next_sibling(),
            XmlNode::Text(v) => v.next_sibling(),
            XmlNode::CData(v) => v.next_sibling(),
            XmlNode::EntityReference(v) => v.next_sibling(),
            XmlNode::Entity(v) => v.next_sibling(),
            XmlNode::PI(v) => v.next_sibling(),
            XmlNode::Comment(v) => v.next_sibling(),
            XmlNode::Document(v) => v.next_sibling(),
            XmlNode::DocumentType(v) => v.next_sibling(),
            XmlNode::DocumentFragment(v) => v.next_sibling(),
            XmlNode::Notation(v) => v.next_sibling(),
            XmlNode::Namespace(v) => v.next_sibling(),
            XmlNode::ResolvedText(v) => v.next_sibling(),
        }
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        match self {
            XmlNode::Element(v) => v.attributes(),
            XmlNode::Attribute(v) => v.attributes(),
            XmlNode::Text(v) => v.attributes(),
            XmlNode::CData(v) => v.attributes(),
            XmlNode::EntityReference(v) => v.attributes(),
            XmlNode::Entity(v) => v.attributes(),
            XmlNode::PI(v) => v.attributes(),
            XmlNode::Comment(v) => v.attributes(),
            XmlNode::Document(v) => v.attributes(),
            XmlNode::DocumentType(v) => v.attributes(),
            XmlNode::DocumentFragment(v) => v.attributes(),
            XmlNode::Notation(v) => v.attributes(),
            XmlNode::Namespace(v) => v.attributes(),
            XmlNode::ResolvedText(v) => v.attributes(),
        }
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        match self {
            XmlNode::Element(v) => v.owner_document(),
            XmlNode::Attribute(v) => v.owner_document(),
            XmlNode::Text(v) => v.owner_document(),
            XmlNode::CData(v) => v.owner_document(),
            XmlNode::EntityReference(v) => v.owner_document(),
            XmlNode::Entity(v) => v.owner_document(),
            XmlNode::PI(v) => v.owner_document(),
            XmlNode::Comment(v) => v.owner_document(),
            XmlNode::Document(v) => v.owner_document(),
            XmlNode::DocumentType(v) => v.owner_document(),
            XmlNode::DocumentFragment(v) => v.owner_document(),
            XmlNode::Notation(v) => v.owner_document(),
            XmlNode::Namespace(v) => v.owner_document(),
            XmlNode::ResolvedText(v) => v.owner_document(),
        }
    }

    fn has_child(&self) -> bool {
        match self {
            XmlNode::Element(v) => v.has_child(),
            XmlNode::Attribute(v) => v.has_child(),
            XmlNode::Text(v) => v.has_child(),
            XmlNode::CData(v) => v.has_child(),
            XmlNode::EntityReference(v) => v.has_child(),
            XmlNode::Entity(v) => v.has_child(),
            XmlNode::PI(v) => v.has_child(),
            XmlNode::Comment(v) => v.has_child(),
            XmlNode::Document(v) => v.has_child(),
            XmlNode::DocumentType(v) => v.has_child(),
            XmlNode::DocumentFragment(v) => v.has_child(),
            XmlNode::Notation(v) => v.has_child(),
            XmlNode::Namespace(v) => v.has_child(),
            XmlNode::ResolvedText(v) => v.has_child(),
        }
    }
}

impl AsExpandedName for XmlNode {
    fn as_expanded_name(&self) -> error::Result<Option<ExpandedName>> {
        match self {
            XmlNode::Element(v) => v.as_expanded_name(),
            XmlNode::Attribute(v) => v.as_expanded_name(),
            XmlNode::Text(_) => Ok(None),
            XmlNode::CData(_) => Ok(None),
            XmlNode::EntityReference(_) => Ok(None),
            XmlNode::Entity(_) => Ok(None),
            XmlNode::PI(v) => v.as_expanded_name(),
            XmlNode::Comment(_) => Ok(None),
            XmlNode::Document(_) => Ok(None),
            XmlNode::DocumentType(_) => Ok(None),
            XmlNode::DocumentFragment(_) => Ok(None),
            XmlNode::Notation(_) => Ok(None),
            XmlNode::Namespace(v) => v.as_expanded_name(),
            XmlNode::ResolvedText(_) => Ok(None),
        }
    }
}

impl AsStringValue for XmlNode {
    fn as_string_value(&self) -> error::Result<String> {
        match self {
            XmlNode::Element(v) => v.as_string_value(),
            XmlNode::Attribute(v) => v.as_string_value(),
            XmlNode::Text(v) => v.as_string_value(),
            XmlNode::CData(v) => v.as_string_value(),
            XmlNode::EntityReference(_) => Ok("".to_string()),
            XmlNode::Entity(_) => Ok("".to_string()),
            XmlNode::PI(v) => v.as_string_value(),
            XmlNode::Comment(v) => v.as_string_value(),
            XmlNode::Document(v) => v.as_string_value(),
            XmlNode::DocumentType(_) => Ok("".to_string()),
            XmlNode::DocumentFragment(v) => v.as_string_value(),
            XmlNode::Notation(_) => Ok("".to_string()),
            XmlNode::Namespace(v) => v.as_string_value(),
            XmlNode::ResolvedText(v) => v.as_string_value(),
        }
    }
}

impl XmlNode {
    pub fn order(&self) -> i64 {
        match self {
            XmlNode::Attribute(v) => v.attribute.borrow().order(),
            XmlNode::CData(v) => v.data.borrow().order(),
            XmlNode::Comment(v) => v.data.borrow().order(),
            XmlNode::Document(v) => v.document.borrow().order(),
            XmlNode::DocumentFragment(v) => v.document.borrow().order(),
            XmlNode::DocumentType(v) => v.declaration.borrow().order(),
            XmlNode::Element(v) => v.element.borrow().order(),
            XmlNode::Entity(_) => 0,
            XmlNode::EntityReference(v) => v.order,
            XmlNode::Namespace(v) => v.namespace.borrow().order(),
            XmlNode::Notation(_) => 0,
            XmlNode::PI(_) => 0,
            XmlNode::ResolvedText(v) => v.data[0].order(),
            XmlNode::Text(v) => v.data.borrow().order(),
        }
    }

    fn previous_sibling_child(&self, node: XmlNode) -> Option<XmlNode> {
        let children = match &self {
            XmlNode::Element(v) => v.children(),
            XmlNode::Attribute(v) => v.children(),
            XmlNode::EntityReference(v) => v.children(),
            XmlNode::Entity(v) => v.children(),
            XmlNode::Document(v) => v.children(),
            XmlNode::DocumentFragment(v) => v.children(),
            _ => return None,
        };

        children
            .iter()
            .rev()
            .skip_while(|&v| v.order() != node.order())
            .nth(1)
            .cloned()
    }

    fn next_sibling_child(&self, node: XmlNode) -> Option<XmlNode> {
        let children = match &self {
            XmlNode::Element(v) => v.children(),
            XmlNode::Attribute(v) => v.children(),
            XmlNode::EntityReference(v) => v.children(),
            XmlNode::Entity(v) => v.children(),
            XmlNode::Document(v) => v.children(),
            XmlNode::DocumentFragment(v) => v.children(),
            _ => return None,
        };

        children
            .iter()
            .skip_while(|&v| v.order() != node.order())
            .nth(1)
            .cloned()
    }
}

impl From<info::XmlItem> for XmlNode {
    fn from(value: info::XmlItem) -> Self {
        match value {
            info::XmlItem::Attribute(v) => XmlAttr::from(v).as_node(),
            info::XmlItem::CData(v) => XmlCDataSection::from(v).as_node(),
            info::XmlItem::CharReference(v) => XmlEntityReference::from(v).as_node(),
            info::XmlItem::Comment(v) => XmlComment::from(v).as_node(),
            info::XmlItem::DeclarationAttList(_) => unimplemented!("declaration attribute"),
            info::XmlItem::Document(v) => XmlDocument::from(v).as_node(),
            info::XmlItem::DocumentType(v) => XmlDocumentType::from(v).as_node(),
            info::XmlItem::Element(v) => XmlElement::from(v).as_node(),
            info::XmlItem::Namespace(v) => XmlNamespace::from(v).as_node(),
            info::XmlItem::Notation(v) => XmlNotation::from(v).as_node(),
            info::XmlItem::PI(v) => XmlProcessingInstruction::from(v).as_node(),
            info::XmlItem::Text(v) => XmlText::from(v).as_node(),
            info::XmlItem::Unexpanded(v) => XmlEntityReference::from(v).as_node(),
            info::XmlItem::Unparsed(v) => XmlEntity::from(v).as_node(),
            info::XmlItem::Entity(v) => XmlEntity::from(v).as_node(),
        }
    }
}

impl fmt::Display for XmlNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            XmlNode::Element(v) => v.fmt(f),
            XmlNode::Attribute(v) => v.fmt(f),
            XmlNode::Text(v) => v.fmt(f),
            XmlNode::CData(v) => v.fmt(f),
            XmlNode::EntityReference(v) => v.fmt(f),
            XmlNode::Entity(v) => v.fmt(f),
            XmlNode::PI(v) => v.fmt(f),
            XmlNode::Comment(v) => v.fmt(f),
            XmlNode::Document(v) => v.fmt(f),
            XmlNode::DocumentType(v) => v.fmt(f),
            XmlNode::DocumentFragment(v) => v.fmt(f),
            XmlNode::Notation(v) => v.fmt(f),
            XmlNode::Namespace(v) => v.fmt(f),
            XmlNode::ResolvedText(v) => v.fmt(f),
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub trait AsNode {
    fn as_node(&self) -> XmlNode;
}

// -----------------------------------------------------------------------------------------------

pub trait AsExpandedName {
    fn as_expanded_name(&self) -> error::Result<Option<ExpandedName>>;
}

// -----------------------------------------------------------------------------------------------

pub trait AsStringValue {
    fn as_string_value(&self) -> error::Result<String>;
}

// -----------------------------------------------------------------------------------------------

trait HasChild {
    fn children(&self) -> Vec<XmlNode>;

    fn first_child_node(&self) -> Option<XmlNode> {
        let mut children = self.children();
        if children.is_empty() {
            None
        } else {
            Some(children.remove(0))
        }
    }

    fn last_child_node(&self) -> Option<XmlNode> {
        let mut children = self.children();
        if children.is_empty() {
            None
        } else {
            Some(children.remove(children.len() - 1))
        }
    }

    fn has_child_node(&self) -> bool {
        !self.children().is_empty()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlDomImplementation;

impl DomImplementation for XmlDomImplementation {
    fn has_feature(&self, feature: &str, version: Option<&str>) -> bool {
        feature.to_ascii_lowercase() == "xml" && version.map(|v| v == "1.0").unwrap_or(true)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlDocumentFragment {
    document: info::XmlNode<info::XmlDocument>,
    parent: Option<info::XmlNode<info::XmlDocument>>,
}

impl DocumentFragment for XmlDocumentFragment {}

impl Node for XmlDocumentFragment {
    fn node_name(&self) -> String {
        "#document-fragment".to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::DocumentFragment
    }

    fn parent_node(&self) -> Option<XmlNode> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        self.parent.clone().map(XmlDocument::from)
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl AsNode for XmlDocumentFragment {
    fn as_node(&self) -> XmlNode {
        XmlNode::DocumentFragment(self.clone())
    }
}

impl AsStringValue for XmlDocumentFragment {
    fn as_string_value(&self) -> error::Result<String> {
        self.root_element()?.as_string_value()
    }
}

impl HasChild for XmlDocumentFragment {
    fn children(&self) -> Vec<XmlNode> {
        self.document
            .borrow()
            .children()
            .iter()
            .map(XmlNode::from)
            .collect()
    }
}

impl fmt::Debug for XmlDocumentFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlDocumentFragment {{ {:?} }}", self.root_element())
    }
}

impl fmt::Display for XmlDocumentFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.document.borrow().fmt(f)
    }
}

impl XmlDocumentFragment {
    fn root_element(&self) -> error::Result<XmlElement> {
        let element = self.document.borrow().document_element()?.clone();
        Ok(XmlElement::from(element))
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlDocument {
    document: info::XmlNode<info::XmlDocument>,
}

impl Document for XmlDocument {
    fn doc_type(&self) -> Option<XmlDocumentType> {
        self.document
            .borrow()
            .prolog()
            .borrow()
            .declaration()
            .cloned()
            .map(XmlDocumentType::from)
    }

    fn implementation(&self) -> XmlDomImplementation {
        XmlDomImplementation {}
    }

    fn document_element(&self) -> error::Result<XmlElement> {
        self.root_element()
    }

    fn get_elements_by_tag_name(&self, tag_name: &str) -> error::Result<XmlNodeList> {
        let mut nodes: Vec<XmlNode> = vec![];

        for v in self.root_element()?.elements_by_tag_name(tag_name) {
            nodes.push(v.as_node())
        }

        Ok(XmlNodeList { nodes })
    }
}

impl Node for XmlDocument {
    fn node_name(&self) -> String {
        "#document".to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::Document
    }

    fn parent_node(&self) -> Option<XmlNode> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        None
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl AsNode for XmlDocument {
    fn as_node(&self) -> XmlNode {
        XmlNode::Document(self.clone())
    }
}

impl AsStringValue for XmlDocument {
    fn as_string_value(&self) -> error::Result<String> {
        self.root_element()?.as_string_value()
    }
}

impl HasChild for XmlDocument {
    fn children(&self) -> Vec<XmlNode> {
        self.document
            .borrow()
            .children()
            .iter()
            .map(XmlNode::from)
            .collect()
    }
}

impl From<info::XmlNode<info::XmlDocument>> for XmlDocument {
    fn from(value: info::XmlNode<info::XmlDocument>) -> Self {
        XmlDocument { document: value }
    }
}

impl fmt::Debug for XmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlDocument {{ {:?} }}", self.root_element())
    }
}

impl fmt::Display for XmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.document.borrow().fmt(f)
    }
}

impl XmlDocument {
    pub fn from_raw(value: &str) -> error::Result<(&str, Self)> {
        let (rest, tree) = xml_parser::document(value)?;
        let document = info::XmlDocument::new(&tree)?;
        let dom = XmlDocument::from(document);
        Ok((rest, dom))
    }

    fn root_element(&self) -> error::Result<XmlElement> {
        let element = self.document.borrow().document_element()?;
        Ok(XmlElement::from(element))
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlNodeList {
    nodes: Vec<XmlNode>,
}

impl NodeList for XmlNodeList {
    fn item(&self, index: usize) -> Option<XmlNode> {
        let node = self.nodes.get(index);
        node.cloned()
    }

    fn length(&self) -> usize {
        self.nodes.len()
    }
}

impl XmlNodeList {
    pub fn empty() -> Self {
        XmlNodeList { nodes: vec![] }
    }

    pub fn iter(&self) -> XmlNodeIter {
        XmlNodeIter {
            nodes: self.clone(),
            index: 0,
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub struct XmlNodeIter {
    nodes: XmlNodeList,
    index: usize,
}

impl Iterator for XmlNodeIter {
    type Item = XmlNode;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.nodes.item(self.index);
        self.index += 1;
        item
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlNamedNodeMap {
    nodes: Vec<(String, XmlNode)>,
}

impl NamedNodeMap for XmlNamedNodeMap {
    fn get_named_item(&self, name: &str) -> Option<XmlNode> {
        let node = self.nodes.iter().find(|v| v.0 == name).map(|v| &v.1);
        node.cloned()
    }

    fn item(&self, index: usize) -> Option<XmlNode> {
        let node = self.nodes.get(index).map(|v| &v.1);
        node.cloned()
    }

    fn length(&self) -> usize {
        self.nodes.len()
    }
}

impl XmlNamedNodeMap {
    pub fn empty() -> Self {
        XmlNamedNodeMap { nodes: vec![] }
    }

    pub fn iter(&self) -> XmlNamedNodeIter {
        XmlNamedNodeIter {
            nodes: self.clone(),
            index: 0,
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub struct XmlNamedNodeIter {
    nodes: XmlNamedNodeMap,
    index: usize,
}

impl Iterator for XmlNamedNodeIter {
    type Item = XmlNode;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.nodes.item(self.index);
        self.index += 1;
        item
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlAttr {
    attribute: info::XmlNode<info::XmlAttribute>,
}

impl Attr for XmlAttr {
    fn name(&self) -> String {
        self.attribute.borrow().local_name().to_string()
    }

    fn specified(&self) -> bool {
        self.attribute.borrow().owner_element().is_ok()
    }

    fn value(&self) -> error::Result<String> {
        Ok(self.attribute.borrow().normalized_value()?)
    }
}

impl Node for XmlAttr {
    fn node_name(&self) -> String {
        self.name()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(Some(self.value()?))
    }

    fn node_type(&self) -> NodeType {
        NodeType::Attribute
    }

    fn parent_node(&self) -> Option<XmlNode> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.attribute.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl AsNode for XmlAttr {
    fn as_node(&self) -> XmlNode {
        XmlNode::Attribute(self.clone())
    }
}

impl AsExpandedName for XmlAttr {
    fn as_expanded_name(&self) -> error::Result<Option<ExpandedName>> {
        let local_name = self.attribute.borrow().local_name().to_string();
        let (prefix, ns) = if let Some(XmlNode::Element(element)) = self.parent_node() {
            let prefix = self
                .attribute
                .borrow()
                .prefix()
                .unwrap_or("xmlns")
                .to_string();
            let namespaces = element.in_scope_namespace()?;
            if let Some(ns) = namespaces.iter().find(|v| v.node_name() == prefix) {
                (Some(prefix), ns.node_value()?)
            } else {
                (Some(prefix), None)
            }
        } else {
            (None, None)
        };
        Ok(Some((local_name, prefix, ns)))
    }
}

impl AsStringValue for XmlAttr {
    fn as_string_value(&self) -> error::Result<String> {
        self.value()
    }
}

impl HasChild for XmlAttr {
    fn children(&self) -> Vec<XmlNode> {
        let mut nodes: Vec<XmlNode> = vec![];

        for v in self.attribute.borrow().values() {
            match v {
                info::XmlAttributeValue::Reference(v) => {
                    nodes.push(XmlEntityReference::from(v.clone()).as_node());
                }
                info::XmlAttributeValue::Text(data) => {
                    let i = info::XmlText::new(
                        data,
                        self.attribute.borrow().owner_element().unwrap(),
                        self.attribute.borrow().owner(),
                    );
                    if !data.is_empty() {
                        nodes.push(XmlText::from(i).as_node());
                    }
                }
            }
        }

        nodes
    }
}

impl From<info::XmlNode<info::XmlAttribute>> for XmlAttr {
    fn from(value: info::XmlNode<info::XmlAttribute>) -> Self {
        XmlAttr { attribute: value }
    }
}

impl fmt::Debug for XmlAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlAttr {{ {} }}", self.name())
    }
}

impl fmt::Display for XmlAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.attribute.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlElement {
    element: info::XmlNode<info::XmlElement>,
}

impl Element for XmlElement {
    fn tag_name(&self) -> String {
        self.element.borrow().local_name().to_string()
    }

    fn get_attribute(&self, name: &str) -> error::Result<String> {
        let attr = self.get_attribute_node(name);
        if let Some(attr) = attr {
            attr.value()
        } else {
            Ok(String::new())
        }
    }

    fn get_attribute_node(&self, name: &str) -> Option<XmlAttr> {
        self.element
            .borrow()
            .attributes()
            .iter()
            .find(|v| v.borrow().local_name() == name)
            .map(XmlAttr::from)
    }

    fn get_elements_by_tag_name(&self, tag_name: &str) -> XmlNodeList {
        let nodes = self
            .elements_by_tag_name(tag_name)
            .iter()
            .map(|v| v.as_node())
            .collect();
        XmlNodeList { nodes }
    }
}

impl Node for XmlElement {
    fn node_name(&self) -> String {
        self.tag_name()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::Element
    }

    fn parent_node(&self) -> Option<XmlNode> {
        self.element.borrow().parent().ok().map(XmlNode::from)
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        let nodes = self
            .element
            .borrow()
            .attributes()
            .iter()
            .map(XmlAttr::from)
            .map(|v| (v.name(), v.as_node()))
            .collect();

        Some(XmlNamedNodeMap { nodes })
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.element.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl AsNode for XmlElement {
    fn as_node(&self) -> XmlNode {
        XmlNode::Element(self.clone())
    }
}

impl AsExpandedName for XmlElement {
    fn as_expanded_name(&self) -> error::Result<Option<ExpandedName>> {
        let local_name = self.element.borrow().local_name().to_string();
        let prefix = self
            .element
            .borrow()
            .prefix()
            .unwrap_or("xmlns")
            .to_string();
        let namespaces = self.in_scope_namespace()?;
        let ns = if let Some(ns) = namespaces.iter().find(|v| v.node_name() == prefix) {
            ns.node_value()?
        } else {
            None
        };
        Ok(Some((local_name, Some(prefix), ns)))
    }
}

impl AsStringValue for XmlElement {
    fn as_string_value(&self) -> error::Result<String> {
        let mut s = String::new();
        for child in self.children() {
            match child {
                XmlNode::Attribute(_) => {}
                XmlNode::CData(v) => s.push_str(&v.as_string_value()?),
                XmlNode::Comment(_) => {}
                XmlNode::Document(_) => {}
                XmlNode::DocumentFragment(_) => {}
                XmlNode::DocumentType(_) => {}
                XmlNode::Element(v) => s.push_str(&v.as_string_value()?),
                XmlNode::Entity(_) => {}
                XmlNode::EntityReference(_) => {}
                XmlNode::Namespace(_) => {}
                XmlNode::Notation(_) => {}
                XmlNode::PI(_) => {}
                XmlNode::ResolvedText(v) => s.push_str(&v.as_string_value()?),
                XmlNode::Text(v) => s.push_str(&v.as_string_value()?),
            }
        }
        Ok(s)
    }
}

impl HasChild for XmlElement {
    fn children(&self) -> Vec<XmlNode> {
        let mut children = vec![];

        let mut text: Option<XmlResolvedText> = None;
        for child in self.element.borrow().children().iter() {
            let child = XmlNode::from(child);
            match child {
                XmlNode::CData(v) => {
                    if let Some(t) = text.as_mut() {
                        t.push_cdata(v);
                    } else {
                        text = Some(XmlResolvedText::from(v));
                    }
                }
                XmlNode::EntityReference(v) => {
                    if let Some(t) = text.as_mut() {
                        t.push_reference(v);
                    } else {
                        text = Some(XmlResolvedText::from(v));
                    }
                }
                XmlNode::Text(v) => {
                    if let Some(t) = text.as_mut() {
                        t.push_text(v);
                    } else {
                        text = Some(XmlResolvedText::from(v));
                    }
                }
                _ => {
                    if let Some(t) = text {
                        children.push(t.as_node());
                    }

                    text = None;
                    children.push(child);
                }
            }
        }

        if let Some(t) = text {
            children.push(t.as_node());
        }

        children
    }
}

impl XmlElement {
    pub fn in_scope_namespace(&self) -> error::Result<Vec<XmlNamespace>> {
        Ok(self
            .element
            .borrow()
            .in_scope_namespace()?
            .iter()
            .map(XmlNamespace::from)
            .collect())
    }

    fn elements_by_tag_name(&self, tag_name: &str) -> Vec<XmlElement> {
        let mut elems = vec![];

        if self.match_tag_name(tag_name) {
            elems.push(self.clone());
        }

        for child in self.children() {
            if let XmlNode::Element(child) = child {
                let mut descendant = child.elements_by_tag_name(tag_name);
                elems.append(&mut descendant);
            }
        }

        elems
    }

    fn match_tag_name(&self, tag_name: &str) -> bool {
        tag_name == "*" || self.node_name() == tag_name
    }
}

impl From<info::XmlNode<info::XmlElement>> for XmlElement {
    fn from(value: info::XmlNode<info::XmlElement>) -> Self {
        XmlElement { element: value }
    }
}

impl fmt::Debug for XmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlElement {{ {} }}", self.node_name())
    }
}

impl fmt::Display for XmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.element.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlText {
    data: info::XmlNode<info::XmlText>,
}

impl Text for XmlText {}

impl TextMut for XmlText {
    fn split_text(&mut self, offset: usize) -> error::Result<XmlResolvedText> {
        todo!();
    }
}

impl CharacterData for XmlText {
    fn data(&self) -> error::Result<String> {
        Ok(self.data.borrow().character_code().to_string())
    }

    fn length(&self) -> usize {
        self.data.borrow().len()
    }

    fn substring_data(&self, offset: usize, count: usize) -> String {
        self.data.borrow().substring(offset..(offset + count))
    }
}

impl CharacterDataMut for XmlText {
    fn insert_data(&mut self, offset: usize, arg: &str) -> error::Result<()> {
        if self.length() < offset {
            Err(error::Error::IndexSizeErr)
        } else {
            self.data.borrow_mut().insert(offset, arg)?;
            Ok(())
        }
    }

    fn delete_data(&mut self, offset: usize, count: usize) -> error::Result<()> {
        if self.length() < (offset + count) {
            Err(error::Error::IndexSizeErr)
        } else {
            self.data.borrow_mut().delete(offset, count);
            Ok(())
        }
    }
}

impl Node for XmlText {
    fn node_name(&self) -> String {
        "#text".to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(Some(self.data()?))
    }

    fn node_type(&self) -> NodeType {
        NodeType::Text
    }

    fn parent_node(&self) -> Option<XmlNode> {
        self.data
            .borrow()
            .parent()
            .ok()
            .map(XmlElement::from)
            .map(|v| v.as_node())
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.data.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl NodeMut for XmlText {
    fn set_node_value(&mut self, value: &str) -> error::Result<()> {
        self.set_data(value)
    }

    fn insert_before(&mut self, _: XmlNode, _: Option<&XmlNode>) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }

    fn remove_child(&mut self, _: &XmlNode) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }
}

impl AsNode for XmlText {
    fn as_node(&self) -> XmlNode {
        XmlNode::Text(self.clone())
    }
}

impl AsStringValue for XmlText {
    fn as_string_value(&self) -> error::Result<String> {
        self.data()
    }
}

impl From<info::XmlNode<info::XmlText>> for XmlText {
    fn from(value: info::XmlNode<info::XmlText>) -> Self {
        XmlText { data: value }
    }
}

impl fmt::Debug for XmlText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlText {{ {} }}", self.data.borrow().character_code())
    }
}

impl fmt::Display for XmlText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.data.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlComment {
    data: info::XmlNode<info::XmlComment>,
}

impl Comment for XmlComment {}

impl CommentMut for XmlComment {}

impl CharacterData for XmlComment {
    fn data(&self) -> error::Result<String> {
        Ok(self.data.borrow().comment().to_string())
    }

    fn length(&self) -> usize {
        self.data.borrow().len()
    }

    fn substring_data(&self, offset: usize, count: usize) -> String {
        self.data.borrow().substring(offset..(offset + count))
    }
}

impl CharacterDataMut for XmlComment {
    fn insert_data(&mut self, offset: usize, arg: &str) -> error::Result<()> {
        if self.length() < offset {
            Err(error::Error::IndexSizeErr)
        } else {
            self.data.borrow_mut().insert(offset, arg)?;
            Ok(())
        }
    }

    fn delete_data(&mut self, offset: usize, count: usize) -> error::Result<()> {
        if self.length() < (offset + count) {
            Err(error::Error::IndexSizeErr)
        } else {
            self.data.borrow_mut().delete(offset, count);
            Ok(())
        }
    }
}

impl Node for XmlComment {
    fn node_name(&self) -> String {
        "#comment".to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(Some(self.data()?))
    }

    fn node_type(&self) -> NodeType {
        NodeType::Comment
    }

    fn parent_node(&self) -> Option<XmlNode> {
        self.data.borrow().parent().ok().map(XmlNode::from)
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.data.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl NodeMut for XmlComment {
    fn set_node_value(&mut self, value: &str) -> error::Result<()> {
        self.set_data(value)
    }

    fn insert_before(&mut self, _: XmlNode, _: Option<&XmlNode>) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }

    fn remove_child(&mut self, _: &XmlNode) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }
}

impl AsNode for XmlComment {
    fn as_node(&self) -> XmlNode {
        XmlNode::Comment(self.clone())
    }
}

impl AsStringValue for XmlComment {
    fn as_string_value(&self) -> error::Result<String> {
        self.data()
    }
}

impl From<info::XmlNode<info::XmlComment>> for XmlComment {
    fn from(value: info::XmlNode<info::XmlComment>) -> Self {
        XmlComment { data: value }
    }
}

impl fmt::Debug for XmlComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlComment {{ {} }}", self.data.borrow().comment())
    }
}

impl fmt::Display for XmlComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.data.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlCDataSection {
    data: info::XmlNode<info::XmlCData>,
}

impl CDataSection for XmlCDataSection {}

impl Text for XmlCDataSection {}

impl TextMut for XmlCDataSection {
    fn split_text(&mut self, offset: usize) -> error::Result<XmlResolvedText> {
        todo!();
    }
}

impl CharacterData for XmlCDataSection {
    fn data(&self) -> error::Result<String> {
        Ok(self.data.borrow().character_code().to_string())
    }

    fn length(&self) -> usize {
        self.data.borrow().len()
    }

    fn substring_data(&self, offset: usize, count: usize) -> String {
        self.data.borrow().substring(offset..(offset + count))
    }
}

impl CharacterDataMut for XmlCDataSection {
    fn insert_data(&mut self, offset: usize, arg: &str) -> error::Result<()> {
        if self.length() < offset {
            Err(error::Error::IndexSizeErr)
        } else {
            self.data.borrow_mut().insert(offset, arg)?;
            Ok(())
        }
    }

    fn delete_data(&mut self, offset: usize, count: usize) -> error::Result<()> {
        if self.length() < (offset + count) {
            Err(error::Error::IndexSizeErr)
        } else {
            self.data.borrow_mut().delete(offset, count);
            Ok(())
        }
    }
}

impl Node for XmlCDataSection {
    fn node_name(&self) -> String {
        "#cdata-section".to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(Some(self.data()?))
    }

    fn node_type(&self) -> NodeType {
        NodeType::CData
    }

    fn parent_node(&self) -> Option<XmlNode> {
        self.data
            .borrow()
            .parent()
            .ok()
            .map(XmlElement::from)
            .map(|v| v.as_node())
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.data.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl NodeMut for XmlCDataSection {
    fn set_node_value(&mut self, value: &str) -> error::Result<()> {
        self.set_data(value)
    }

    fn insert_before(&mut self, _: XmlNode, _: Option<&XmlNode>) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }

    fn remove_child(&mut self, _: &XmlNode) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }
}

impl AsNode for XmlCDataSection {
    fn as_node(&self) -> XmlNode {
        XmlNode::CData(self.clone())
    }
}

impl AsStringValue for XmlCDataSection {
    fn as_string_value(&self) -> error::Result<String> {
        self.data()
    }
}

impl From<info::XmlNode<info::XmlCData>> for XmlCDataSection {
    fn from(value: info::XmlNode<info::XmlCData>) -> Self {
        XmlCDataSection { data: value }
    }
}

impl fmt::Debug for XmlCDataSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "XmlCDataSection {{ {} }}",
            self.data.borrow().character_code()
        )
    }
}

impl fmt::Display for XmlCDataSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.data.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlDocumentType {
    declaration: info::XmlNode<info::XmlDocumentTypeDeclaration>,
}

impl DocumentType for XmlDocumentType {
    fn name(&self) -> String {
        self.declaration.borrow().local_name().to_string()
    }

    fn entities(&self) -> XmlNamedNodeMap {
        let nodes = self
            .declaration
            .borrow()
            .entities()
            .iter()
            .cloned()
            .map(XmlEntity::from)
            .map(|v| (v.node_name(), v.as_node()))
            .collect();

        XmlNamedNodeMap { nodes }
    }

    fn notations(&self) -> XmlNamedNodeMap {
        let nodes = self
            .declaration
            .borrow()
            .notations()
            .iter()
            .cloned()
            .map(XmlNotation::from)
            .map(|v| (v.node_name(), v.as_node()))
            .collect();

        XmlNamedNodeMap { nodes }
    }
}

impl Node for XmlDocumentType {
    fn node_name(&self) -> String {
        self.name()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::DocumentType
    }

    fn parent_node(&self) -> Option<XmlNode> {
        Some(XmlDocument::from(self.declaration.borrow().parent()).as_node())
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.declaration.borrow().parent()))
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl AsNode for XmlDocumentType {
    fn as_node(&self) -> XmlNode {
        XmlNode::DocumentType(self.clone())
    }
}

impl From<info::XmlNode<info::XmlDocumentTypeDeclaration>> for XmlDocumentType {
    fn from(value: info::XmlNode<info::XmlDocumentTypeDeclaration>) -> Self {
        XmlDocumentType { declaration: value }
    }
}

impl fmt::Debug for XmlDocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlDocumentType {{ {} }}", self.name())
    }
}

impl fmt::Display for XmlDocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.declaration.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlNotation {
    notation: info::XmlNode<info::XmlNotation>,
}

impl Notation for XmlNotation {
    fn public_id(&self) -> Option<String> {
        self.notation
            .borrow()
            .public_identifier()
            .map(|v| v.to_string())
    }

    fn system_id(&self) -> Option<String> {
        self.notation
            .borrow()
            .system_identifier()
            .map(|v| v.to_string())
    }
}

impl Node for XmlNotation {
    fn node_name(&self) -> String {
        self.notation.borrow().name().to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::Notation
    }

    fn parent_node(&self) -> Option<XmlNode> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        let parent = XmlNode::from(self.notation.borrow().parent());
        parent.previous_sibling_child(self.as_node())
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        let parent = XmlNode::from(self.notation.borrow().parent());
        parent.next_sibling_child(self.as_node())
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.notation.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl AsNode for XmlNotation {
    fn as_node(&self) -> XmlNode {
        XmlNode::Notation(self.clone())
    }
}

impl From<info::XmlNode<info::XmlNotation>> for XmlNotation {
    fn from(value: info::XmlNode<info::XmlNotation>) -> Self {
        XmlNotation { notation: value }
    }
}

impl fmt::Debug for XmlNotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlNotation {{ {} }}", self.node_name())
    }
}

impl fmt::Display for XmlNotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.notation.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlEntity {
    entity: info::XmlNode<info::XmlEntity>,
}

impl Entity for XmlEntity {
    fn public_id(&self) -> Option<String> {
        self.entity
            .borrow()
            .public_identifier()
            .map(|v| v.to_string())
    }

    fn system_id(&self) -> Option<String> {
        self.entity
            .borrow()
            .system_identifier()
            .map(|v| v.to_string())
    }

    fn notation_name(&self) -> Option<String> {
        self.entity.borrow().notation_name().map(|v| v.to_string())
    }
}

impl Node for XmlEntity {
    fn node_name(&self) -> String {
        self.entity.borrow().name().to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::Entity
    }

    fn parent_node(&self) -> Option<XmlNode> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        let parent = self.entity.borrow().parent().map(XmlNode::from);
        parent.and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        let parent = self.entity.borrow().parent().map(XmlNode::from);
        parent.and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.entity.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        !self.children().is_empty()
    }
}

impl AsNode for XmlEntity {
    fn as_node(&self) -> XmlNode {
        XmlNode::Entity(self.clone())
    }
}

impl HasChild for XmlEntity {
    fn children(&self) -> Vec<XmlNode> {
        // TODO:
        vec![]
    }
}

impl From<info::XmlNode<info::XmlEntity>> for XmlEntity {
    fn from(value: info::XmlNode<info::XmlEntity>) -> Self {
        XmlEntity { entity: value }
    }
}

impl From<info::XmlNode<info::XmlUnparsedEntity>> for XmlEntity {
    fn from(value: info::XmlNode<info::XmlUnparsedEntity>) -> Self {
        XmlEntity {
            entity: value.borrow().entity(),
        }
    }
}

impl fmt::Debug for XmlEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlEntity {{ {} }}", self.node_name())
    }
}

impl fmt::Display for XmlEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.entity.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlEntityReference {
    reference: info::XmlNode<info::XmlReference>,
    order: i64,
}

impl EntityReference for XmlEntityReference {}

impl Node for XmlEntityReference {
    fn node_name(&self) -> String {
        match self.reference.borrow().value() {
            info::XmlReferenceValue::Character(ch, radix) => match radix {
                10 => format!("#{}", ch),
                16 => format!("#x{}", ch),
                _ => unreachable!(),
            },
            info::XmlReferenceValue::Entity(e) => e.to_string(),
        }
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::EntityReference
    }

    fn parent_node(&self) -> Option<XmlNode> {
        self.reference.borrow().parent().ok().map(XmlNode::from)
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.reference.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        !self.children().is_empty()
    }
}

impl AsNode for XmlEntityReference {
    fn as_node(&self) -> XmlNode {
        XmlNode::EntityReference(self.clone())
    }
}

impl HasChild for XmlEntityReference {
    fn children(&self) -> Vec<XmlNode> {
        // TODO:
        vec![]
    }
}

impl From<info::XmlNode<info::XmlCharReference>> for XmlEntityReference {
    fn from(value: info::XmlNode<info::XmlCharReference>) -> Self {
        let order = value.borrow().order();
        let reference = info::XmlReference::new_from_char_ref(value);
        XmlEntityReference { reference, order }
    }
}

impl From<info::XmlNode<info::XmlReference>> for XmlEntityReference {
    fn from(value: info::XmlNode<info::XmlReference>) -> Self {
        XmlEntityReference {
            reference: value,
            order: -1,
        }
    }
}

impl From<info::XmlNode<info::XmlUnexpandedEntityReference>> for XmlEntityReference {
    fn from(value: info::XmlNode<info::XmlUnexpandedEntityReference>) -> Self {
        let order = value.borrow().order();
        let reference = info::XmlReference::new_from_ref(value.borrow().entity());
        XmlEntityReference { reference, order }
    }
}

impl fmt::Debug for XmlEntityReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlEntityReference {{ {} }}", self.node_name())
    }
}

impl fmt::Display for XmlEntityReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.reference.borrow().fmt(f)
    }
}

impl XmlEntityReference {
    pub fn value(&self) -> error::Result<String> {
        Ok(self.reference.borrow().resolve()?)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlProcessingInstruction {
    pi: info::XmlNode<info::XmlProcessingInstruction>,
}

impl ProcessingInstruction for XmlProcessingInstruction {
    fn target(&self) -> String {
        self.pi.borrow().target().to_string()
    }

    fn data(&self) -> String {
        self.pi.borrow().content().to_string()
    }
}

impl Node for XmlProcessingInstruction {
    fn node_name(&self) -> String {
        self.target()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(None)
    }

    fn node_type(&self) -> NodeType {
        NodeType::PI
    }

    fn parent_node(&self) -> Option<XmlNode> {
        Some(XmlNode::from(self.pi.borrow().parent()))
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.pi.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl AsNode for XmlProcessingInstruction {
    fn as_node(&self) -> XmlNode {
        XmlNode::PI(self.clone())
    }
}

impl AsExpandedName for XmlProcessingInstruction {
    fn as_expanded_name(&self) -> error::Result<Option<ExpandedName>> {
        Ok(Some((self.node_name(), None, None)))
    }
}

impl AsStringValue for XmlProcessingInstruction {
    fn as_string_value(&self) -> error::Result<String> {
        Ok(self.pi.borrow().content().to_string())
    }
}

impl From<info::XmlNode<info::XmlProcessingInstruction>> for XmlProcessingInstruction {
    fn from(value: info::XmlNode<info::XmlProcessingInstruction>) -> Self {
        XmlProcessingInstruction { pi: value }
    }
}

impl fmt::Debug for XmlProcessingInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "XmlProcessingInstruction {{ {} }}", self.node_name())
    }
}

impl fmt::Display for XmlProcessingInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.pi.borrow().fmt(f)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlNamespace {
    namespace: info::XmlNode<info::XmlNamespace>,
}

impl Node for XmlNamespace {
    fn node_name(&self) -> String {
        self.namespace
            .borrow()
            .prefix()
            .unwrap_or("xmlns")
            .to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(Some(self.namespace.borrow().namespace_name().to_string()))
    }

    fn node_type(&self) -> NodeType {
        NodeType::Attribute
    }

    fn parent_node(&self) -> Option<XmlNode> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList { nodes: vec![] }
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        None
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        None
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl AsNode for XmlNamespace {
    fn as_node(&self) -> XmlNode {
        XmlNode::Namespace(self.clone())
    }
}

impl AsExpandedName for XmlNamespace {
    fn as_expanded_name(&self) -> error::Result<Option<ExpandedName>> {
        Ok(Some((self.node_name(), None, None)))
    }
}

impl AsStringValue for XmlNamespace {
    fn as_string_value(&self) -> error::Result<String> {
        Ok(self.namespace.borrow().namespace_name().to_string())
    }
}

impl From<info::XmlNode<info::XmlNamespace>> for XmlNamespace {
    fn from(value: info::XmlNode<info::XmlNamespace>) -> Self {
        XmlNamespace { namespace: value }
    }
}

impl fmt::Debug for XmlNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "XmlNamespace {{ {} }}",
            self.node_value().map_err(|_| fmt::Error)?.unwrap()
        )
    }
}

impl fmt::Display for XmlNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.implicit() {
            Ok(())
        } else {
            self.namespace.borrow().fmt(f)
        }
    }
}

impl XmlNamespace {
    pub fn implicit(&self) -> bool {
        self.namespace.borrow().implicit()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlResolvedText {
    data: Vec<XmlNode>,
}

impl Text for XmlResolvedText {}

impl TextMut for XmlResolvedText {
    fn split_text(&mut self, offset: usize) -> error::Result<XmlResolvedText> {
        todo!();
    }
}

impl CharacterData for XmlResolvedText {
    fn data(&self) -> error::Result<String> {
        let mut s = String::new();
        for d in self.data.as_slice() {
            match d {
                XmlNode::CData(v) => s.push_str(v.data()?.as_str()),
                XmlNode::EntityReference(v) => s.push_str(v.value()?.as_str()),
                XmlNode::Text(v) => s.push_str(v.data()?.as_str()),
                _ => unreachable!(),
            }
        }
        Ok(s)
    }

    fn length(&self) -> usize {
        self.data().unwrap_or_default().chars().count()
    }

    fn substring_data(&self, offset: usize, count: usize) -> String {
        self.data()
            .unwrap_or_default()
            .chars()
            .skip(offset)
            .take(count)
            .collect()
    }
}

impl CharacterDataMut for XmlResolvedText {
    fn insert_data(&mut self, offset: usize, arg: &str) -> error::Result<()> {
        if self.length() < offset {
            Err(error::Error::IndexSizeErr)
        } else {
            let mut length = 0;
            for d in self.data.as_mut_slice() {
                match d {
                    XmlNode::CData(v) => {
                        length += v.length();
                        if offset <= length {
                            v.insert_data(offset - (length - v.length()), arg)?;
                            break;
                        }
                    }
                    XmlNode::EntityReference(v) => {
                        length += v.value()?.chars().count();
                        if offset <= length {
                            return Err(error::Error::NoDataAllowedErr);
                        }
                    }
                    XmlNode::Text(v) => {
                        length += v.length();
                        if offset <= length {
                            v.insert_data(offset - (length - v.length()), arg)?;
                            break;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Ok(())
        }
    }

    fn delete_data(&mut self, offset: usize, count: usize) -> error::Result<()> {
        if self.length() < (offset + count) {
            Err(error::Error::IndexSizeErr)
        } else {
            let mut length = 0;
            for d in self.data.as_mut_slice() {
                match d {
                    XmlNode::CData(v) => {
                        length += v.length();
                        if offset < length {
                            v.delete_data(offset - (length - v.length()), count)?;
                            break;
                        }
                    }
                    XmlNode::EntityReference(v) => {
                        length += v.value()?.chars().count();
                        if offset < length {
                            return Err(error::Error::NoDataAllowedErr);
                        }
                    }
                    XmlNode::Text(v) => {
                        length += v.length();
                        if offset < length {
                            v.delete_data(offset - (length - v.length()), count)?;
                            break;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Ok(())
        }
    }
}

impl Node for XmlResolvedText {
    fn node_name(&self) -> String {
        "#text".to_string()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(Some(self.data()?))
    }

    fn node_type(&self) -> NodeType {
        NodeType::Text
    }

    fn parent_node(&self) -> Option<XmlNode> {
        self.data[0].parent_node()
    }

    fn child_nodes(&self) -> XmlNodeList {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode> {
        None
    }

    fn last_child(&self) -> Option<XmlNode> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode> {
        self.parent_node()
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        self.data[0].owner_document()
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl NodeMut for XmlResolvedText {
    fn set_node_value(&mut self, value: &str) -> error::Result<()> {
        self.set_data(value)
    }

    fn insert_before(&mut self, _: XmlNode, _: Option<&XmlNode>) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }

    fn remove_child(&mut self, _: &XmlNode) -> error::Result<XmlNode> {
        Err(error::Error::HierarchyRequestErr)
    }
}

impl AsNode for XmlResolvedText {
    fn as_node(&self) -> XmlNode {
        XmlNode::ResolvedText(self.clone())
    }
}

impl AsStringValue for XmlResolvedText {
    fn as_string_value(&self) -> error::Result<String> {
        self.data()
    }
}

impl From<XmlCDataSection> for XmlResolvedText {
    fn from(value: XmlCDataSection) -> Self {
        XmlResolvedText {
            data: vec![value.as_node()],
        }
    }
}

impl From<XmlEntityReference> for XmlResolvedText {
    fn from(value: XmlEntityReference) -> Self {
        XmlResolvedText {
            data: vec![value.as_node()],
        }
    }
}

impl From<XmlText> for XmlResolvedText {
    fn from(value: XmlText) -> Self {
        XmlResolvedText {
            data: vec![value.as_node()],
        }
    }
}

impl fmt::Display for XmlResolvedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for d in self.data.as_slice() {
            d.fmt(f)?;
        }

        Ok(())
    }
}

impl XmlResolvedText {
    fn push_cdata(&mut self, value: XmlCDataSection) {
        self.data.push(value.as_node());
    }

    fn push_reference(&mut self, value: XmlEntityReference) {
        self.data.push(value.as_node());
    }

    fn push_text(&mut self, value: XmlText) {
        self.data.push(value.as_node());
    }
}

// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_implmentation() {
        let m = XmlDomImplementation {};
        assert!(!m.has_feature("html", None));
        assert!(m.has_feature("xml", None));
        assert!(!m.has_feature("xml", Some("0.9")));
        assert!(m.has_feature("xml", Some("1.0")));
    }

    #[test]
    fn test_document_fragment() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let root = XmlNode::Element(XmlElement {
            element: document.borrow().document_element().unwrap(),
        });

        let m = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // Node
        assert_eq!("#document-fragment", m.node_name());
        assert_eq!(None, m.node_value().unwrap());
        assert_eq!(NodeType::DocumentFragment, m.node_type());
        assert_eq!(None, m.parent_node());
        for child in m.child_nodes().iter() {
            assert_eq!(root, child);
        }
        assert_eq!(Some(root.clone()), m.first_child());
        assert_eq!(Some(root.clone()), m.last_child());
        assert_eq!(None, m.previous_sibling());
        assert_eq!(None, m.next_sibling());
        assert_eq!(None, m.attributes());
        assert_eq!(
            Some(XmlDocument::from(document.clone())),
            m.owner_document()
        );
        assert!(m.has_child());

        // XmlNode
        let node = m.as_node();
        assert_eq!("#document-fragment", node.node_name());
        assert_eq!(None, node.node_value().unwrap());
        assert_eq!(NodeType::DocumentFragment, node.node_type());
        assert_eq!(None, node.parent_node());
        for child in node.child_nodes().iter() {
            assert_eq!(root, child);
        }
        assert_eq!(Some(root.clone()), node.first_child());
        assert_eq!(Some(root.clone()), node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(
            Some(XmlDocument::from(document.clone())),
            node.owner_document()
        );
        assert!(node.has_child());

        // AsStringValue
        assert_eq!("", m.as_string_value().unwrap());
    }

    #[test]
    fn test_document() {
        let (_, m) = XmlDocument::from_raw("<root></root>").unwrap();
        let elem = XmlElement {
            element: m.document.borrow().document_element().unwrap(),
        };
        let root = XmlNode::Element(elem.clone());

        // Document
        assert_eq!(None, m.doc_type());
        assert_eq!(XmlDomImplementation {}, m.implementation());
        assert_eq!(elem, m.document_element().unwrap());
        for child in m.get_elements_by_tag_name("root").unwrap().iter() {
            assert_eq!(root, child);
        }

        // Node
        assert_eq!("#document", m.node_name());
        assert_eq!(None, m.node_value().unwrap());
        assert_eq!(NodeType::Document, m.node_type());
        assert_eq!(None, m.parent_node());
        for child in m.child_nodes().iter() {
            assert_eq!(root, child);
        }
        assert_eq!(Some(root.clone()), m.first_child());
        assert_eq!(Some(root.clone()), m.last_child());
        assert_eq!(None, m.previous_sibling());
        assert_eq!(None, m.next_sibling());
        assert_eq!(None, m.attributes());
        assert_eq!(None, m.owner_document());
        assert!(m.has_child());

        // XmlNode
        let node = m.as_node();
        assert_eq!("#document", node.node_name());
        assert_eq!(None, node.node_value().unwrap());
        assert_eq!(NodeType::Document, node.node_type());
        assert_eq!(None, node.parent_node());
        for child in node.child_nodes().iter() {
            assert_eq!(root, child);
        }
        assert_eq!(Some(root.clone()), node.first_child());
        assert_eq!(Some(root.clone()), node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(None, node.owner_document());
        assert!(node.has_child());

        // AsStringValue
        assert_eq!("", m.as_string_value().unwrap());
    }

    #[test]
    fn test_attr() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let elem = doc.document_element().unwrap();
        let attr = elem.get_attribute_node("a").unwrap();
        let text = XmlNode::Text(XmlText {
            data: info::XmlText::new(
                "b",
                doc.document.borrow().document_element().unwrap(),
                doc.document.clone(),
            ),
        });

        // Attr
        assert!(attr.specified());

        // Node
        assert_eq!("a", attr.node_name());
        assert_eq!(Some("b".to_string()), attr.node_value().unwrap());
        assert_eq!(NodeType::Attribute, attr.node_type());
        assert_eq!(None, attr.parent_node());
        for child in attr.child_nodes().iter() {
            assert_eq!(text, child);
        }
        assert_eq!(Some(text.clone()), attr.first_child());
        assert_eq!(Some(text.clone()), attr.last_child());
        assert_eq!(None, attr.previous_sibling());
        assert_eq!(None, attr.next_sibling());
        assert_eq!(None, attr.attributes());
        assert_eq!(Some(doc.clone()), attr.owner_document());
        assert!(attr.has_child());

        // XmlNode
        let node = attr.as_node();
        assert_eq!("a", node.node_name());
        assert_eq!(Some("b".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::Attribute, node.node_type());
        assert_eq!(None, node.parent_node());
        for child in node.child_nodes().iter() {
            assert_eq!(text, child);
        }
        assert_eq!(Some(text.clone()), node.first_child());
        assert_eq!(Some(text.clone()), node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(node.has_child());

        // AsStringValue
        assert_eq!("b", attr.as_string_value().unwrap());
    }

    #[test]
    fn test_element() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 =
            if let XmlNode::Element(e) = root.get_elements_by_tag_name("elem1").item(0).unwrap() {
                e.clone()
            } else {
                unreachable!()
            };
        let elem2 =
            if let XmlNode::Element(e) = root.get_elements_by_tag_name("elem2").item(0).unwrap() {
                e.clone()
            } else {
                unreachable!()
            };
        let attra = elem1.get_attribute_node("a").unwrap();
        let attrc = elem2.get_attribute_node("c").unwrap();
        let data1 = XmlResolvedText::from(XmlText {
            data: info::XmlText::new(
                "data1",
                doc.document.borrow().document_element().unwrap(),
                doc.document.clone(),
            ),
        })
        .as_node();

        // Element
        assert_eq!("b", elem1.get_attribute("a").unwrap());
        assert_eq!(Some(attra.clone()), elem1.get_attribute_node("a"));

        // Node (elem1)
        assert_eq!("elem1", elem1.node_name());
        assert_eq!(None, elem1.node_value().unwrap());
        assert_eq!(NodeType::Element, elem1.node_type());
        assert_eq!(Some(root.as_node()), elem1.parent_node());
        for child in elem1.child_nodes().iter() {
            assert_eq!(data1, child);
        }
        assert_eq!(Some(data1.clone()), elem1.first_child());
        assert_eq!(Some(data1.clone()), elem1.last_child());
        assert_eq!(None, elem1.previous_sibling());
        assert_eq!(Some(elem2.as_node()), elem1.next_sibling());
        for child in elem1.attributes().unwrap().iter() {
            assert_eq!(attra.as_node(), child);
        }
        assert_eq!(Some(doc.clone()), elem1.owner_document());
        assert!(elem1.has_child());

        // Node (elem2)
        assert_eq!("elem2", elem2.node_name());
        assert_eq!(None, elem2.node_value().unwrap());
        assert_eq!(NodeType::Element, elem2.node_type());
        assert_eq!(Some(root.as_node()), elem2.parent_node());
        assert_eq!(XmlNodeList { nodes: vec![] }, elem2.child_nodes());
        assert_eq!(None, elem2.first_child());
        assert_eq!(None, elem2.last_child());
        assert_eq!(Some(elem1.as_node()), elem2.previous_sibling());
        assert_eq!(None, elem2.next_sibling());
        for child in elem2.attributes().unwrap().iter() {
            assert_eq!(attrc.as_node(), child);
        }
        assert_eq!(Some(doc.clone()), elem2.owner_document());
        assert!(!elem2.has_child());

        // XmlNode (elem1)
        let node = elem1.as_node();
        assert_eq!("elem1", node.node_name());
        assert_eq!(None, node.node_value().unwrap());
        assert_eq!(NodeType::Element, node.node_type());
        assert_eq!(Some(root.as_node()), node.parent_node());
        for child in node.child_nodes().iter() {
            assert_eq!(data1, child);
        }
        assert_eq!(Some(data1.clone()), node.first_child());
        assert_eq!(Some(data1.clone()), node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(Some(elem2.as_node()), node.next_sibling());
        for child in node.attributes().unwrap().iter() {
            assert_eq!(attra.as_node(), child);
        }
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(node.has_child());

        // AsStringValue
        assert_eq!("data1", elem1.as_string_value().unwrap());
        assert_eq!("", elem2.as_string_value().unwrap());
    }

    #[test]
    fn test_text() {
        let (_, doc) = XmlDocument::from_raw("<root>text</root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut text = if let XmlNode::ResolvedText(e) = root.child_nodes().item(0).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(4, text.length());
        assert_eq!("ex", text.substring_data(1, 2));

        // CharacterDataMut
        text.set_data("あいう").unwrap();
        assert_eq!(Some("あいう"), text.node_value().unwrap().as_deref());
        text.append_data("えお").unwrap();
        assert_eq!(Some("あいうえお"), text.node_value().unwrap().as_deref());
        text.insert_data(1, "abc").unwrap();
        assert_eq!(Some("あabcいうえお"), text.node_value().unwrap().as_deref());
        text.delete_data(4, 2).unwrap();
        assert_eq!(Some("あabcえお"), text.node_value().unwrap().as_deref());
        text.replace_data(1, 3, "いう").unwrap();
        assert_eq!(Some("あいうえお"), text.node_value().unwrap().as_deref());
        text.set_data("text").unwrap();

        // Node
        assert_eq!("#text", text.node_name());
        assert_eq!(Some("text".to_string()), text.node_value().unwrap());
        assert_eq!(NodeType::Text, text.node_type());
        assert_eq!(Some(root.as_node()), text.parent_node());
        assert_eq!(XmlNodeList::empty(), text.child_nodes());
        assert_eq!(None, text.first_child());
        assert_eq!(None, text.last_child());
        assert_eq!(None, text.previous_sibling());
        assert_eq!(None, text.next_sibling());
        assert_eq!(None, text.attributes());
        assert_eq!(Some(doc.clone()), text.owner_document());
        assert!(!text.has_child());

        // Nodemut
        let e = root.clone().as_node();
        text.set_node_value("abc").unwrap();
        assert_eq!(Some("abc"), text.node_value().unwrap().as_deref());
        text.insert_before(e.clone(), Some(&e)).err().unwrap();
        text.replace_child(e.clone(), &e).err().unwrap();
        text.remove_child(&e).err().unwrap();
        text.append_child(e.clone()).err().unwrap();
        text.set_node_value("text").unwrap();

        // XmlNode
        let node = text.as_node();
        assert_eq!("#text", node.node_name());
        assert_eq!(Some("text".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::Text, node.node_type());
        assert_eq!(Some(root.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());

        // AsStringValue
        assert_eq!("text", text.as_string_value().unwrap());
    }

    #[test]
    fn test_comment() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut comment = if let XmlNode::Comment(e) = root.child_nodes().item(0).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(9, comment.length());
        assert_eq!("co", comment.substring_data(1, 2));

        // CharacterDataMut
        comment.set_data("あいう").unwrap();
        assert_eq!(Some("あいう"), comment.node_value().unwrap().as_deref());
        comment.append_data("えお").unwrap();
        assert_eq!(Some("あいうえお"), comment.node_value().unwrap().as_deref());
        comment.insert_data(1, "abc").unwrap();
        assert_eq!(
            Some("あabcいうえお"),
            comment.node_value().unwrap().as_deref()
        );
        comment.delete_data(4, 2).unwrap();
        assert_eq!(Some("あabcえお"), comment.node_value().unwrap().as_deref());
        comment.replace_data(1, 3, "いう").unwrap();
        assert_eq!(Some("あいうえお"), comment.node_value().unwrap().as_deref());
        comment.set_data(" comment ").unwrap();

        // Node
        assert_eq!("#comment", comment.node_name());
        assert_eq!(Some(" comment ".to_string()), comment.node_value().unwrap());
        assert_eq!(NodeType::Comment, comment.node_type());
        assert_eq!(Some(root.as_node()), comment.parent_node());
        assert_eq!(XmlNodeList::empty(), comment.child_nodes());
        assert_eq!(None, comment.first_child());
        assert_eq!(None, comment.last_child());
        assert_eq!(None, comment.previous_sibling());
        assert_eq!(None, comment.next_sibling());
        assert_eq!(None, comment.attributes());
        assert_eq!(Some(doc.clone()), comment.owner_document());
        assert!(!comment.has_child());

        // Nodemut
        let e = root.clone().as_node();
        comment.set_node_value("abc").unwrap();
        assert_eq!(Some("abc"), comment.node_value().unwrap().as_deref());
        comment.insert_before(e.clone(), Some(&e)).err().unwrap();
        comment.replace_child(e.clone(), &e).err().unwrap();
        comment.remove_child(&e).err().unwrap();
        comment.append_child(e.clone()).err().unwrap();
        comment.set_node_value(" comment ").unwrap();

        // XmlNode
        let node = comment.as_node();
        assert_eq!("#comment", node.node_name());
        assert_eq!(Some(" comment ".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::Comment, node.node_type());
        assert_eq!(Some(root.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());

        // AsStringValue
        assert_eq!(" comment ", comment.as_string_value().unwrap());
    }

    #[test]
    fn test_cdata() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut cdata = if let XmlNode::ResolvedText(e) = root.child_nodes().item(0).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(4, cdata.length());
        assert_eq!("<>", cdata.substring_data(1, 2));

        // CharacterDataMut
        cdata.set_data("あいう").unwrap();
        assert_eq!(Some("あいう"), cdata.node_value().unwrap().as_deref());
        cdata.append_data("えお").unwrap();
        assert_eq!(Some("あいうえお"), cdata.node_value().unwrap().as_deref());
        cdata.insert_data(1, "abc").unwrap();
        assert_eq!(
            Some("あabcいうえお"),
            cdata.node_value().unwrap().as_deref()
        );
        cdata.delete_data(4, 2).unwrap();
        assert_eq!(Some("あabcえお"), cdata.node_value().unwrap().as_deref());
        cdata.replace_data(1, 3, "いう").unwrap();
        assert_eq!(Some("あいうえお"), cdata.node_value().unwrap().as_deref());
        cdata.set_data("&<>\"").unwrap();

        // Node
        assert_eq!("#text", cdata.node_name());
        assert_eq!(Some("&<>\"".to_string()), cdata.node_value().unwrap());
        assert_eq!(NodeType::Text, cdata.node_type());
        assert_eq!(Some(root.as_node()), cdata.parent_node());
        assert_eq!(XmlNodeList::empty(), cdata.child_nodes());
        assert_eq!(None, cdata.first_child());
        assert_eq!(None, cdata.last_child());
        assert_eq!(None, cdata.previous_sibling());
        assert_eq!(None, cdata.next_sibling());
        assert_eq!(None, cdata.attributes());
        assert_eq!(Some(doc.clone()), cdata.owner_document());
        assert!(!cdata.has_child());

        // Nodemut
        let e = root.clone().as_node();
        cdata.set_node_value("abc").unwrap();
        assert_eq!(Some("abc"), cdata.node_value().unwrap().as_deref());
        cdata.insert_before(e.clone(), Some(&e)).err().unwrap();
        cdata.replace_child(e.clone(), &e).err().unwrap();
        cdata.remove_child(&e).err().unwrap();
        cdata.append_child(e.clone()).err().unwrap();
        cdata.set_node_value("&<>\"").unwrap();

        // XmlNode
        let node = cdata.as_node();
        assert_eq!("#text", node.node_name());
        assert_eq!(Some("&<>\"".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::Text, node.node_type());
        assert_eq!(Some(root.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());

        // AsStringValue
        assert_eq!("&<>\"", cdata.as_string_value().unwrap());
    }

    #[test]
    fn test_namespace() {
        let (_, doc) = XmlDocument::from_raw("<root xmlns:a='http://test/a'></root>").unwrap();
        let root = doc.document_element().unwrap();
        let namespaces = root.in_scope_namespace().unwrap();
        let ns = namespaces.first().unwrap();

        // Node
        assert_eq!("a", ns.node_name());
        assert_eq!(Some("http://test/a".to_string()), ns.node_value().unwrap());
        assert_eq!(NodeType::Attribute, ns.node_type());
        assert_eq!(None, ns.parent_node());
        assert_eq!(XmlNodeList::empty(), ns.child_nodes());
        assert_eq!(None, ns.first_child());
        assert_eq!(None, ns.last_child());
        assert_eq!(None, ns.previous_sibling());
        assert_eq!(None, ns.next_sibling());
        assert_eq!(None, ns.attributes());
        assert_eq!(None, ns.owner_document());
        assert!(!ns.has_child());

        // XmlNode
        let node = ns.as_node();
        assert_eq!("a", node.node_name());
        assert_eq!(
            Some("http://test/a".to_string()),
            node.node_value().unwrap()
        );
        assert_eq!(NodeType::Attribute, node.node_type());
        assert_eq!(None, node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(None, node.owner_document());
        assert!(!node.has_child());

        // AsStringValue
        assert_eq!("http://test/a", ns.as_string_value().unwrap());
    }

    #[test]
    fn test_resolved_text() {
        let (_, doc) =
            XmlDocument::from_raw("<root>a<![CDATA[b]]>c<a />&#x3042;d&amp;d</root>").unwrap();
        let root = doc.document_element().unwrap();
        let text = if let XmlNode::ResolvedText(e) = root.child_nodes().item(0).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };
        let a = if let XmlNode::Element(e) = root.child_nodes().item(1).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(3, text.length());
        assert_eq!("bc", text.substring_data(1, 2));

        // Node
        assert_eq!("#text", text.node_name());
        assert_eq!(Some("abc".to_string()), text.node_value().unwrap());
        assert_eq!(NodeType::Text, text.node_type());
        assert_eq!(Some(root.as_node()), text.parent_node());
        assert_eq!(XmlNodeList::empty(), text.child_nodes());
        assert_eq!(None, text.first_child());
        assert_eq!(None, text.last_child());
        assert_eq!(None, text.previous_sibling());
        assert_eq!(Some(a.as_node()), text.next_sibling());
        assert_eq!(None, text.attributes());
        assert_eq!(Some(doc.clone()), text.owner_document());
        assert!(!text.has_child());

        // XmlNode
        let node = text.as_node();
        assert_eq!("#text", node.node_name());
        assert_eq!(Some("abc".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::Text, node.node_type());
        assert_eq!(Some(root.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(Some(a.as_node()), node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());

        // AsStringValue
        assert_eq!("abc", text.as_string_value().unwrap());

        let text = if let XmlNode::ResolvedText(e) = root.child_nodes().item(2).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(4, text.length());
        assert_eq!("d&", text.substring_data(1, 2));

        // Node
        assert_eq!("#text", text.node_name());
        assert_eq!(Some("あd&d".to_string()), text.node_value().unwrap());
        assert_eq!(NodeType::Text, text.node_type());
        assert_eq!(Some(root.as_node()), text.parent_node());
        assert_eq!(XmlNodeList::empty(), text.child_nodes());
        assert_eq!(None, text.first_child());
        assert_eq!(None, text.last_child());
        assert_eq!(Some(a.as_node()), text.previous_sibling());
        assert_eq!(None, text.next_sibling());
        assert_eq!(None, text.attributes());
        assert_eq!(Some(doc.clone()), text.owner_document());
        assert!(!text.has_child());

        // AsStringValue
        assert_eq!("あd&d", text.as_string_value().unwrap());
    }
}

// -----------------------------------------------------------------------------------------------
