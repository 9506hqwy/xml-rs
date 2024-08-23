pub mod error;

use std::any::Any;
use std::convert;
use std::fmt;
use std::iter::Iterator;
use xml_info as info;
use xml_info::{
    Attribute as InfoAttribute, Character as InfoCharacter, Comment as InfoComment,
    Document as InfoDocument, DocumentTypeDeclaration as InfoDocumentTypeDeclaration,
    Element as InfoElement, HasChildren as InfoHasChildren, HasOwner as InfoHasOwner,
    HasQName as InfoHasQName, Namespace as InfoNamespace, Notation as InfoNotation,
    ProcessingInstruction as InfoProcessingInstruction, Sortable as InfoSortable,
};

// TODO: re-implement ResolvedText
// TODO: error handling for write method.

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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>>;

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

pub trait NamedNodeMap<T> {
    fn get_named_item(&self, name: &str) -> Option<T>;

    fn item(&self, index: usize) -> Option<T>;

    fn length(&self) -> usize;
}

pub trait NamedNodeMapMut<T>: NamedNodeMap<T> {
    fn set_named_item(&mut self, arg: T) -> error::Result<Option<T>>;

    fn remove_named_item(&mut self, name: &str) -> error::Result<T>;
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

    fn entities(&self) -> XmlNamedNodeMap<XmlEntity>;

    fn notations(&self) -> XmlNamedNodeMap<XmlNotation>;
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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
            XmlNode::EntityReference(v) => v.reference.borrow().order(),
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

impl convert::TryFrom<XmlNode> for info::XmlItem {
    type Error = error::Error;

    fn try_from(value: XmlNode) -> Result<Self, Self::Error> {
        let v = match value {
            XmlNode::Attribute(v) => v.attribute.into(),
            XmlNode::CData(v) => v.data.into(),
            XmlNode::Comment(v) => v.data.into(),
            XmlNode::Document(v) => v.document.into(),
            XmlNode::DocumentFragment(v) => v.document.into(),
            XmlNode::DocumentType(v) => v.declaration.into(),
            XmlNode::Element(v) => v.element.into(),
            XmlNode::Entity(v) => v.entity.into(),
            XmlNode::EntityReference(v) => v.reference.try_into()?,
            XmlNode::Namespace(v) => v.namespace.into(),
            XmlNode::Notation(v) => v.notation.into(),
            XmlNode::PI(v) => v.pi.into(),
            XmlNode::ResolvedText(_) => unimplemented!("multi text node."),
            XmlNode::Text(v) => v.data.into(),
        };
        Ok(v)
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

impl XmlNode {
    pub fn as_comment(&self) -> Option<XmlComment> {
        if let XmlNode::Comment(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_doctype(&self) -> Option<XmlDocumentType> {
        if let XmlNode::DocumentType(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_element(&self) -> Option<XmlElement> {
        if let XmlNode::Element(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_entity_ref(&self) -> Option<XmlEntityReference> {
        if let XmlNode::EntityReference(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_pi(&self) -> Option<XmlProcessingInstruction> {
        if let XmlNode::PI(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<XmlText> {
        if let XmlNode::Text(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_resolved_text(&self) -> Option<XmlResolvedText> {
        if let XmlNode::ResolvedText(v) = self {
            Some(v.clone())
        } else {
            None
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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
            .document_declaration()
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

impl DocumentMut for XmlDocument {
    fn create_element(&self, tag_name: &str) -> error::Result<XmlElement> {
        let element = info::XmlElement::empty(tag_name, self.document.clone())?;
        Ok(XmlElement { element })
    }

    fn create_document_fragment(&self) -> XmlDocumentFragment {
        let document = info::XmlDocument::empty();
        XmlDocumentFragment {
            document,
            parent: Some(self.document.clone()),
        }
    }

    fn create_text_node(&self, data: &str) -> error::Result<XmlText> {
        let text = info::XmlText::empty(self.document.clone());
        text.borrow_mut().insert(0, data)?;
        Ok(XmlText { data: text })
    }

    fn create_comment(&self, data: &str) -> error::Result<XmlComment> {
        let comment = info::XmlComment::empty(self.document.clone());
        comment.borrow_mut().insert(0, data)?;
        Ok(XmlComment { data: comment })
    }

    fn create_cdata_section(&self, data: &str) -> error::Result<XmlCDataSection> {
        let cdata = info::XmlCData::empty(self.document.clone());
        cdata.borrow_mut().insert(0, data)?;
        Ok(XmlCDataSection { data: cdata })
    }

    fn create_processing_instruction(
        &self,
        target: &str,
        data: &str,
    ) -> error::Result<XmlProcessingInstruction> {
        let pi = info::XmlProcessingInstruction::empty(target, self.document.clone())?;
        pi.borrow_mut().set_content(data)?;
        Ok(XmlProcessingInstruction { pi })
    }

    fn create_attribute(&self, name: &str) -> error::Result<XmlAttr> {
        let attribute = info::XmlAttribute::empty(name, self.document.clone())?;
        Ok(XmlAttr { attribute })
    }

    fn create_entity_reference(&self, name: &str) -> error::Result<XmlEntityReference> {
        let reference = info::XmlReference::new_from_value(name, self.document.clone())?;
        Ok(XmlEntityReference { reference })
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        None
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl NodeMut for XmlDocument {
    fn set_node_value(&mut self, _: &str) -> error::Result<()> {
        Err(error::Error::NoDataAllowedErr)
    }

    fn insert_before(
        &mut self,
        new_child: XmlNode,
        ref_child: Option<&XmlNode>,
    ) -> error::Result<XmlNode> {
        if Some(self.clone()) != new_child.owner_document() {
            return Err(error::Error::WrongDoucmentErr);
        }

        let value = if let Some(r) = ref_child {
            if Some(self.clone()) != r.owner_document() {
                return Err(error::Error::WrongDoucmentErr);
            }

            // TODO: remove new_child from teee.
            self.document
                .borrow_mut()
                .insert_before_order(new_child.try_into()?, r.order())?
        } else {
            self.document.borrow_mut().append(new_child.try_into()?)?
        };

        self.reset_order();

        Ok(XmlNode::from(value))
    }

    fn remove_child(&mut self, old_child: &XmlNode) -> error::Result<XmlNode> {
        if Some(self.clone()) != old_child.owner_document() {
            return Err(error::Error::WrongDoucmentErr);
        }

        match self
            .document
            .borrow_mut()
            .delete_by_order(old_child.order())
        {
            Some(v) => Ok(XmlNode::from(v)),
            _ => Err(error::Error::NotFoundErr),
        }
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

    fn reset_order(&mut self) {
        self.document.borrow_mut().reset_order();
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
pub struct XmlNamedNodeMap<T>
where
    T: Node + Clone,
{
    nodes: Vec<(String, T)>,
    parent: XmlNode,
}

impl<T> NamedNodeMap<T> for XmlNamedNodeMap<T>
where
    T: Node + Clone,
{
    fn get_named_item(&self, name: &str) -> Option<T> {
        let node = self.nodes.iter().find(|v| v.0 == name).map(|v| &v.1);
        node.cloned()
    }

    fn item(&self, index: usize) -> Option<T> {
        let node = self.nodes.get(index).map(|v| &v.1);
        node.cloned()
    }

    fn length(&self) -> usize {
        self.nodes.len()
    }
}

impl<T> NamedNodeMapMut<T> for XmlNamedNodeMap<T>
where
    T: Node + Clone + Any,
{
    fn set_named_item(&mut self, arg: T) -> error::Result<Option<T>> {
        let name = arg.node_name();
        match self.remove_named_item(name.as_str()) {
            Ok(v) => {
                self.nodes.push((name, arg.clone()));
                self.add_item_to_parent(arg)?;
                Ok(Some(v))
            }
            _ => {
                self.nodes.push((name, arg.clone()));
                self.add_item_to_parent(arg)?;
                Ok(None)
            }
        }
    }

    fn remove_named_item(&mut self, name: &str) -> error::Result<T> {
        if let Some(v) = self.nodes.iter().find(|v| v.0 == name).cloned() {
            self.nodes.retain(|v| v.0 != name);
            self.remove_item_from_parent(v.1.clone())?;
            Ok(v.1)
        } else {
            Err(error::Error::NotFoundErr)
        }
    }
}

impl<T> XmlNamedNodeMap<T>
where
    T: Node + Clone + Any,
{
    pub fn empty(parent: XmlNode) -> Self {
        XmlNamedNodeMap {
            nodes: vec![],
            parent,
        }
    }

    pub fn iter(&self) -> XmlNamedNodeIter<T> {
        XmlNamedNodeIter {
            nodes: self.clone(),
            index: 0,
        }
    }

    fn add_item_to_parent(&mut self, arg: T) -> error::Result<()> {
        let obj = &arg as &dyn Any;

        match &mut self.parent {
            XmlNode::Element(e) => {
                if let Some(v) = obj.downcast_ref::<XmlAttr>() {
                    e.set_attribute_node(v.clone())?;
                    Ok(())
                } else {
                    unreachable!()
                }
            }
            XmlNode::DocumentType(_) => Err(error::Error::NoModificationAllowedErr),
            _ => unreachable!(),
        }
    }

    fn remove_item_from_parent(&mut self, arg: T) -> error::Result<()> {
        let obj = &arg as &dyn Any;

        match &mut self.parent {
            XmlNode::Element(e) => {
                if let Some(v) = obj.downcast_ref::<XmlAttr>() {
                    e.remove_attribute_node(v.clone())?;
                    Ok(())
                } else {
                    unreachable!()
                }
            }
            XmlNode::DocumentType(_) => Err(error::Error::NoModificationAllowedErr),
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub struct XmlNamedNodeIter<T>
where
    T: Node + Clone,
{
    nodes: XmlNamedNodeMap<T>,
    index: usize,
}

impl<T> Iterator for XmlNamedNodeIter<T>
where
    T: Node + Clone,
{
    type Item = T;

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

impl AttrMut for XmlAttr {}

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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.attribute.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl NodeMut for XmlAttr {
    fn set_node_value(&mut self, value: &str) -> error::Result<()> {
        self.attribute.borrow_mut().set_values(value)?;
        Ok(())
    }

    fn insert_before(
        &mut self,
        new_child: XmlNode,
        ref_child: Option<&XmlNode>,
    ) -> error::Result<XmlNode> {
        if self.owner_document() != new_child.owner_document() {
            return Err(error::Error::WrongDoucmentErr);
        }

        let value = if let Some(r) = ref_child {
            if self.owner_document() != r.owner_document() {
                return Err(error::Error::WrongDoucmentErr);
            }

            // TODO: remove new_child from teee.
            self.attribute
                .borrow_mut()
                .insert_before_order(new_child.try_into()?, r.order())?
        } else {
            self.attribute.borrow_mut().append(new_child.try_into()?)?
        };

        self.owner_document().unwrap().reset_order();

        Ok(XmlNode::from(value))
    }

    fn remove_child(&mut self, old_child: &XmlNode) -> error::Result<XmlNode> {
        if self.owner_document() != old_child.owner_document() {
            return Err(error::Error::WrongDoucmentErr);
        }

        match self
            .attribute
            .borrow_mut()
            .delete_by_order(old_child.order())
        {
            Some(v) => Ok(XmlNode::from(v)),
            _ => Err(error::Error::NotFoundErr),
        }
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
        let (prefix, ns) = if let Ok(element) = self.attribute.borrow().owner_element() {
            // TODO: prefix is None
            let prefix = self
                .attribute
                .borrow()
                .prefix()
                .unwrap_or("xmlns")
                .to_string();
            let namespaces = XmlElement::from(element).in_scope_namespace()?;
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
                info::XmlAttributeValue::Text(v) => {
                    nodes.push(XmlText::from(v.clone()).as_node());
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

impl ElementMut for XmlElement {
    fn set_attribute(&mut self, name: &str, value: &str) -> error::Result<()> {
        let mut attr = self.owner_document().unwrap().create_attribute(name)?;
        attr.set_value(value)?;
        self.set_attribute_node(attr)?;
        Ok(())
    }

    fn remove_attribute(&mut self, name: &str) -> error::Result<()> {
        self.element.borrow_mut().remove_attribute(name);
        self.owner_document().unwrap().reset_order();
        Ok(())
    }

    fn set_attribute_node(&mut self, new_attr: XmlAttr) -> error::Result<Option<XmlAttr>> {
        if self.owner_document() != new_attr.owner_document() {
            return Err(error::Error::WrongDoucmentErr);
        }

        let attr = self
            .element
            .borrow_mut()
            .remove_attribute(new_attr.name().as_str());

        self.element
            .borrow_mut()
            .append_attribute(new_attr.attribute);

        self.owner_document().unwrap().reset_order();

        Ok(attr.map(XmlAttr::from))
    }

    fn normalize(&mut self) {
        todo!()
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
        let nodes = self
            .element
            .borrow()
            .attributes()
            .iter()
            .map(XmlAttr::from)
            .map(|v| (v.name(), v))
            .collect();

        Some(XmlNamedNodeMap {
            nodes,
            parent: self.as_node(),
        })
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.element.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl NodeMut for XmlElement {
    fn set_node_value(&mut self, _: &str) -> error::Result<()> {
        Err(error::Error::NoDataAllowedErr)
    }

    fn insert_before(
        &mut self,
        new_child: XmlNode,
        ref_child: Option<&XmlNode>,
    ) -> error::Result<XmlNode> {
        if self.owner_document() != new_child.owner_document() {
            return Err(error::Error::WrongDoucmentErr);
        }

        let value = if let Some(r) = ref_child {
            if self.owner_document() != r.owner_document() {
                return Err(error::Error::WrongDoucmentErr);
            }

            // TODO: remove new_child from teee.
            self.element
                .borrow_mut()
                .insert_before_order(new_child.try_into()?, r.order())?
        } else {
            self.element.borrow_mut().append(new_child.try_into()?)?
        };

        self.owner_document().unwrap().reset_order();

        Ok(XmlNode::from(value))
    }

    fn remove_child(&mut self, old_child: &XmlNode) -> error::Result<XmlNode> {
        if self.owner_document() != old_child.owner_document() {
            return Err(error::Error::WrongDoucmentErr);
        }

        match self.element.borrow_mut().delete_by_order(old_child.order()) {
            Some(v) => Ok(XmlNode::from(v)),
            _ => Err(error::Error::NotFoundErr),
        }
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
        // TODO: prefix is None
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

// -----------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct XmlText {
    data: info::XmlNode<info::XmlText>,
}

impl Text for XmlText {}

impl TextMut for XmlText {
    fn split_text(&mut self, offset: usize) -> error::Result<XmlResolvedText> {
        if self.length() < offset {
            return Err(error::Error::IndexSizeErr);
        }

        let parent = self.data.borrow().parent_item();
        match parent {
            Some(info::XmlItem::Attribute(v)) => {
                let data2 = self.data.borrow_mut().split_at(offset);
                let new_order = self.data.borrow().order() + 1;

                let inserted = v
                    .borrow_mut()
                    .insert_before_order(data2.clone().into(), new_order);

                match inserted {
                    Ok(_) => {}
                    Err(info::error::Error::OufOfIndex(_)) => {
                        v.borrow_mut().append(data2.clone().into())?;
                    }
                    Err(e) => {
                        return Err(error::Error::from(e));
                    }
                }

                self.owner_document().unwrap().reset_order();

                Ok(XmlResolvedText::from(XmlText::from(data2)))
            }
            Some(info::XmlItem::Element(v)) => {
                let data2 = self.data.borrow_mut().split_at(offset);
                let new_order = self.data.borrow().order() + 1;

                let inserted = v
                    .borrow_mut()
                    .insert_before_order(data2.clone().into(), new_order);

                match inserted {
                    Ok(_) => {}
                    Err(info::error::Error::OufOfIndex(_)) => {
                        v.borrow_mut().append(data2.clone().into())?;
                    }
                    Err(e) => {
                        return Err(error::Error::from(e));
                    }
                }

                self.owner_document().unwrap().reset_order();

                Ok(XmlResolvedText::from(XmlText::from(data2)))
            }
            _ => Err(error::Error::HierarchyRequestErr),
        }
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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
        if self.length() < offset {
            return Err(error::Error::IndexSizeErr);
        }

        let v = self.data.borrow().parent()?;

        let data2 = self.data.borrow_mut().split_at(offset);
        let new_order = self.data.borrow().order() + 1;

        let inserted = v
            .borrow_mut()
            .insert_before_order(data2.clone().into(), new_order);

        match inserted {
            Ok(_) => {}
            Err(info::error::Error::OufOfIndex(_)) => {
                v.borrow_mut().append(data2.clone().into())?;
            }
            Err(e) => {
                return Err(error::Error::from(e));
            }
        }

        self.owner_document().unwrap().reset_order();

        Ok(XmlResolvedText::from(XmlCDataSection::from(data2)))
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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

    fn entities(&self) -> XmlNamedNodeMap<XmlEntity> {
        let nodes = self
            .declaration
            .borrow()
            .entities()
            .iter()
            .cloned()
            .map(XmlEntity::from)
            .map(|v| (v.node_name(), v))
            .collect();

        XmlNamedNodeMap {
            nodes,
            parent: self.as_node(),
        }
    }

    fn notations(&self) -> XmlNamedNodeMap<XmlNotation> {
        let nodes = self
            .declaration
            .borrow()
            .notations()
            .iter()
            .cloned()
            .map(XmlNotation::from)
            .map(|v| (v.node_name(), v))
            .collect();

        XmlNamedNodeMap {
            nodes,
            parent: self.as_node(),
        }
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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
        let reference = info::XmlReference::new_from_char_ref(value);
        XmlEntityReference { reference }
    }
}

impl From<info::XmlNode<info::XmlReference>> for XmlEntityReference {
    fn from(value: info::XmlNode<info::XmlReference>) -> Self {
        XmlEntityReference { reference: value }
    }
}

impl From<info::XmlNode<info::XmlUnexpandedEntityReference>> for XmlEntityReference {
    fn from(value: info::XmlNode<info::XmlUnexpandedEntityReference>) -> Self {
        let reference = info::XmlReference::new_from_ref(value.borrow().entity());
        XmlEntityReference { reference }
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

impl ProcessingInstructionMut for XmlProcessingInstruction {
    fn set_data(&mut self, data: &str) -> error::Result<()> {
        self.pi.borrow_mut().set_content(data)?;
        Ok(())
    }
}

impl Node for XmlProcessingInstruction {
    fn node_name(&self) -> String {
        self.target()
    }

    fn node_value(&self) -> error::Result<Option<String>> {
        Ok(Some(self.data()))
    }

    fn node_type(&self) -> NodeType {
        NodeType::PI
    }

    fn parent_node(&self) -> Option<XmlNode> {
        self.pi.borrow().parent().ok().map(XmlNode::from)
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
        None
    }

    fn owner_document(&self) -> Option<XmlDocument> {
        Some(XmlDocument::from(self.pi.borrow().owner()))
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl NodeMut for XmlProcessingInstruction {
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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
        if self.length() < offset {
            Err(error::Error::IndexSizeErr)
        } else {
            let mut data2 = XmlResolvedText { data: vec![] };
            let mut length = 0;
            for (i, d) in self.data.iter_mut().enumerate() {
                match d {
                    XmlNode::CData(v) => {
                        length += v.length();
                        if offset <= length {
                            data2 = v.split_text(offset - (length - v.length()))?;
                            data2.data.append(&mut self.data.split_off(i + 1));
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
                            data2 = v.split_text(offset - (length - v.length()))?;
                            data2.data.append(&mut self.data.split_off(i + 1));
                            break;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Ok(data2)
        }
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

    fn attributes(&self) -> Option<XmlNamedNodeMap<XmlAttr>> {
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
    fn test_document_fragment_node() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let root = XmlNode::Element(XmlElement {
            element: document.borrow().document_element().unwrap(),
        });

        let flag = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // Node
        assert_eq!("#document-fragment", flag.node_name());
        assert_eq!(None, flag.node_value().unwrap());
        assert_eq!(NodeType::DocumentFragment, flag.node_type());
        assert_eq!(None, flag.parent_node());
        for child in flag.child_nodes().iter() {
            assert_eq!(root, child);
        }
        assert_eq!(Some(root.clone()), flag.first_child());
        assert_eq!(Some(root.clone()), flag.last_child());
        assert_eq!(None, flag.previous_sibling());
        assert_eq!(None, flag.next_sibling());
        assert_eq!(None, flag.attributes());
        assert_eq!(
            Some(XmlDocument::from(document.clone())),
            flag.owner_document()
        );
        assert!(flag.has_child());
    }

    #[test]
    fn test_document_fragment_as_node() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let root = XmlNode::Element(XmlElement {
            element: document.borrow().document_element().unwrap(),
        });

        let flag = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // AsNode
        let node = flag.as_node();
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
    }

    #[test]
    fn test_document_fragment_as_string_value() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let flag = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // AsStringValue
        assert_eq!("", flag.as_string_value().unwrap());
    }

    #[test]
    fn test_document_fragment_children() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let root = XmlNode::Element(XmlElement {
            element: document.borrow().document_element().unwrap(),
        });

        let flag = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // HasChild
        assert_eq!(vec![root], flag.children());
    }

    #[test]
    fn test_document_fragment_debug() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let flag = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // fmt::Debug
        assert_eq!(
            "XmlDocumentFragment { Ok(XmlElement { root }) }",
            format!("{:?}", flag)
        );
    }

    #[test]
    fn test_document_fragment_display() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let flag = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // fmt::Display
        assert_eq!("<root />", format!("{}", flag));
    }

    #[test]
    fn test_document_fragment_impl() {
        let (_, tree) = xml_parser::document("<root></root>").unwrap();
        let document = info::XmlDocument::new(&tree).unwrap();

        let root = XmlElement {
            element: document.borrow().document_element().unwrap(),
        };

        let flag = XmlDocumentFragment {
            document: document.clone(),
            parent: Some(document.clone()),
        };

        // XmlDocumentFragment
        assert_eq!(root, flag.root_element().unwrap());
    }

    #[test]
    fn test_document_document() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();
        let elem = XmlElement {
            element: doc.document.borrow().document_element().unwrap(),
        };
        let root = XmlNode::Element(elem.clone());

        // Document
        assert_eq!(None, doc.doc_type());
        assert_eq!(XmlDomImplementation {}, doc.implementation());
        assert_eq!(elem, doc.document_element().unwrap());
        for child in doc.get_elements_by_tag_name("root").unwrap().iter() {
            assert_eq!(root, child);
        }
    }

    #[test]
    fn test_document_document_mut() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();

        // DocumentMut
        let elem = doc.create_element("e").unwrap();
        assert_eq!("e", elem.tag_name());
        let _ = doc.create_document_fragment();
        let text = doc.create_text_node("t").unwrap();
        assert_eq!("t", text.data().unwrap());
        let comment = doc.create_comment("c").unwrap();
        assert_eq!("c", comment.data().unwrap());
        let cdata = doc.create_cdata_section("d").unwrap();
        assert_eq!("d", cdata.data().unwrap());
        let pi = doc.create_processing_instruction("t", "c").unwrap();
        assert_eq!("t", pi.target());
        assert_eq!("c", pi.data());
        let attr = doc.create_attribute("a").unwrap();
        assert_eq!("a", attr.name());
        let eref = doc.create_entity_reference("&amp;").unwrap();
        assert_eq!("amp", eref.node_name());
    }

    #[test]
    fn test_document_node() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();
        let elem = XmlElement {
            element: doc.document.borrow().document_element().unwrap(),
        };
        let root = XmlNode::Element(elem.clone());

        // Node
        assert_eq!("#document", doc.node_name());
        assert_eq!(None, doc.node_value().unwrap());
        assert_eq!(NodeType::Document, doc.node_type());
        assert_eq!(None, doc.parent_node());
        for child in doc.child_nodes().iter() {
            assert_eq!(root, child);
        }
        assert_eq!(Some(root.clone()), doc.first_child());
        assert_eq!(Some(root.clone()), doc.last_child());
        assert_eq!(None, doc.previous_sibling());
        assert_eq!(None, doc.next_sibling());
        assert_eq!(None, doc.attributes());
        assert_eq!(None, doc.owner_document());
        assert!(doc.has_child());
    }

    #[test]
    fn test_document_node_mut() {
        let (_, mut doc) = XmlDocument::from_raw("<root></root>").unwrap();

        // NodeMut
        doc.set_node_value("a").err().unwrap();
        let a = doc
            .insert_before(doc.create_comment("a").unwrap().as_node(), None)
            .unwrap();
        assert_eq!("<root /><!--a-->", format!("{}", doc));
        let b = doc
            .insert_before(doc.create_comment("b").unwrap().as_node(), Some(&a))
            .unwrap();
        assert_eq!("<root /><!--b--><!--a-->", format!("{}", doc));
        let _ = doc
            .replace_child(doc.create_comment("c").unwrap().as_node(), &b)
            .unwrap();
        assert_eq!("<root /><!--c--><!--a-->", format!("{}", doc));
        let a = doc.remove_child(&a).unwrap();
        assert_eq!("<root /><!--c-->", format!("{}", doc));
        doc.append_child(a).unwrap();
        assert_eq!("<root /><!--c--><!--a-->", format!("{}", doc));
    }

    #[test]
    fn test_document_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();
        let elem = XmlElement {
            element: doc.document.borrow().document_element().unwrap(),
        };
        let root = XmlNode::Element(elem.clone());

        // AsNode
        let node = doc.as_node();
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
    }

    #[test]
    fn test_document_as_string_value() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();

        // AsStringValue
        assert_eq!("", doc.as_string_value().unwrap());
    }

    #[test]
    fn test_document_children() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();
        let elem = XmlElement {
            element: doc.document.borrow().document_element().unwrap(),
        };
        let root = XmlNode::Element(elem.clone());

        // HasChild
        assert_eq!(vec![root], doc.children());
    }

    #[test]
    fn test_document_debug() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();

        // fmt::Debug
        assert_eq!(
            "XmlDocument { Ok(XmlElement { root }) }",
            format!("{:?}", doc)
        );
    }

    #[test]
    fn test_document_display() {
        let (_, doc) = XmlDocument::from_raw("<root></root>").unwrap();

        // fmt::Display
        assert_eq!("<root />", format!("{}", doc));
    }

    #[test]
    fn test_document_impl() {
        let (_, mut doc) = XmlDocument::from_raw("<root></root>").unwrap();
        let elem = XmlElement {
            element: doc.document.borrow().document_element().unwrap(),
        };

        // XmlDocument
        assert_eq!(elem, doc.root_element().unwrap());
        doc.reset_order();
    }

    #[test]
    fn test_node_list_node_list() {
        let (_, doc) = XmlDocument::from_raw("<root><e>1</e><e>2</e></root>").unwrap();
        let children = doc.root_element().unwrap().get_elements_by_tag_name("e");

        // NodeList
        assert_eq!("1", children.item(0).unwrap().as_string_value().unwrap());
        assert_eq!(2, children.length());
    }

    #[test]
    fn test_node_list_impl() {
        // AsNodeList
        let list = XmlNodeList::empty();
        assert_eq!(0, list.length());
        let iter = list.iter();
        assert_eq!(0, iter.count());
    }

    #[test]
    fn test_node_list_iter() {
        let (_, doc) = XmlDocument::from_raw("<root><e>1</e><e>2</e></root>").unwrap();
        let children = doc.root_element().unwrap().get_elements_by_tag_name("e");

        // Iterator
        assert_eq!(2, children.iter().count());
    }

    #[test]
    fn test_named_noed_map_named_node_map() {
        let (_, doc) = XmlDocument::from_raw("<root a='1' b='2'/>").unwrap();
        let attrs = doc.root_element().unwrap().attributes().unwrap();

        // NamedNodeMap
        assert_eq!(
            "1",
            attrs
                .get_named_item("a")
                .unwrap()
                .as_string_value()
                .unwrap()
        );
        assert_eq!("2", attrs.item(1).unwrap().as_string_value().unwrap());
        assert_eq!(2, attrs.length());
    }

    #[test]
    fn test_named_noed_map_named_node_map_mut() {
        let (_, doc) = XmlDocument::from_raw("<root a='1' b='2'/>").unwrap();
        let mut root = doc.root_element().unwrap();
        let mut attrs = root.attributes().unwrap();

        // NamedNodeMapMut
        let c = attrs
            .set_named_item(doc.create_attribute("c").unwrap())
            .unwrap();
        assert_eq!(None, c);
        let c = attrs.get_named_item("c").unwrap();
        assert_eq!(c, attrs.get_named_item("c").unwrap());
        assert_eq!(c, root.get_attribute_node("c").unwrap());
        let d = root
            .set_attribute_node(doc.create_attribute("d").unwrap())
            .unwrap();
        assert_eq!(None, d);
        let d = root.get_attribute_node("d").unwrap();
        // FIXME:
        //assert_eq!(d, attrs.get_named_item("d").unwrap());
        assert_eq!(d, root.get_attribute_node("d").unwrap());
        attrs.remove_named_item("c").unwrap();
        assert_eq!(None, attrs.get_named_item("c"));
        assert_eq!(None, root.get_attribute_node("c"));
        root.remove_attribute("d").unwrap();
        // FIXME:
        //assert_eq!(None, attrs.get_named_item("d"));
        assert_eq!(None, root.get_attribute_node("d"));
    }

    #[test]
    fn test_named_noed_map_impl() {
        let (_, doc) = XmlDocument::from_raw("<root />").unwrap();

        // XmlNamedNodeMap
        let map = XmlNamedNodeMap::<XmlAttr>::empty(doc.as_node());
        assert_eq!(0, map.length());
        let iter = map.iter();
        assert_eq!(0, iter.count());
    }

    #[test]
    fn test_named_noed_map_iter() {
        let (_, doc) = XmlDocument::from_raw("<root a='1' b='2'/>").unwrap();
        let attrs = doc.root_element().unwrap().attributes().unwrap();

        // Iterator
        assert_eq!(2, attrs.iter().count());
    }

    #[test]
    fn test_attr_attr() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        // Attr
        assert_eq!("a", attr.name());
        assert!(attr.specified());
        assert_eq!("b", attr.value().unwrap());
    }

    #[test]
    fn test_attr_attr_mut() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let mut attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        // AttrMut
        attr.set_value("c").unwrap();
        assert_eq!("c", attr.value().unwrap());
    }

    #[test]
    fn test_attr_node() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();
        let text = XmlNode::Text(XmlText {
            data: info::XmlText::new("b", None, doc.document.clone()),
        });

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
    }

    #[test]
    fn test_attr_node_mut() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let mut attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        // NodeMut
        attr.set_node_value("a&amp;b&amp;c").unwrap();
        assert_eq!("a&b&c", attr.value().unwrap());
        let d = attr
            .insert_before(doc.create_text_node("d").unwrap().as_node(), None)
            .unwrap();
        assert_eq!("a&b&cd", attr.value().unwrap());
        let e = attr
            .insert_before(doc.create_text_node("e").unwrap().as_node(), Some(&d))
            .unwrap();
        assert_eq!("a&b&ced", attr.value().unwrap());
        let _ = attr
            .replace_child(doc.create_text_node("f").unwrap().as_node(), &e)
            .unwrap();
        assert_eq!("a&b&cfd", attr.value().unwrap());
        let d = attr.remove_child(&d).unwrap();
        assert_eq!("a&b&cf", attr.value().unwrap());
        attr.append_child(d).unwrap();
        assert_eq!("a&b&cfd", attr.value().unwrap());
    }

    #[test]
    fn test_attr_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();
        let text = XmlNode::Text(XmlText {
            data: info::XmlText::new("b", None, doc.document.clone()),
        });

        // AsNode
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
    }

    #[test]
    fn test_attr_as_expanded_name_prefix() {
        let (_, doc) =
            XmlDocument::from_raw("<root c:a='b' xmlns:c='http://test/c'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        dbg!(&attr.parent_node());

        // AsExpandedName
        assert_eq!(
            (
                "a".to_string(),
                Some("c".to_string()),
                Some("http://test/c".to_string())
            ),
            attr.as_expanded_name().unwrap().unwrap()
        );
    }

    #[test]
    fn test_attr_as_expanded_name_unprefix() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        // AsExpandedName
        assert_eq!(
            ("a".to_string(), Some("xmlns".to_string()), None),
            attr.as_expanded_name().unwrap().unwrap()
        );
    }

    #[test]
    fn test_attr_as_string_value() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        // AsStringValue
        assert_eq!("b", attr.as_string_value().unwrap());
    }

    #[test]
    fn test_attr_children() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();
        let text = XmlNode::Text(XmlText {
            data: info::XmlText::new("b", None, doc.document.clone()),
        });

        // HasChild
        assert_eq!(vec![text], attr.children());
    }

    #[test]
    fn test_attr_debug() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        // fmt::Debug
        assert_eq!("XmlAttr { a }", format!("{:?}", attr));
    }

    #[test]
    fn test_attr_display() {
        let (_, doc) = XmlDocument::from_raw("<root a='b'></root>").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();

        // fmt::Display
        assert_eq!("a=\"b\"", format!("{}", attr));
    }

    #[test]
    fn test_element_element() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();
        let attra = elem1.get_attribute_node("a").unwrap();

        // Element
        assert_eq!("elem1", elem1.tag_name());
        assert_eq!("b", elem1.get_attribute("a").unwrap());
        assert_eq!(Some(attra.clone()), elem1.get_attribute_node("a"));
    }

    #[test]
    fn test_element_element_mut() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let mut elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();

        // ElementMut
        elem1.set_attribute("d", "e").unwrap();
        elem1.remove_attribute("d").unwrap();
        elem1
            .set_attribute_node(doc.create_attribute("d").unwrap())
            .unwrap();
        elem1
            .remove_attribute_node(elem1.get_attribute_node("d").unwrap())
            .unwrap();
    }

    #[test]
    fn test_element_node() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();
        let elem2 = root
            .get_elements_by_tag_name("elem2")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();
        let attra = elem1.get_attribute_node("a").unwrap();
        let attrc = elem2.get_attribute_node("c").unwrap();
        let data1 = XmlResolvedText::from(XmlText {
            data: info::XmlText::new(
                "data1",
                Some(doc.document.borrow().document_element().unwrap().into()),
                doc.document.clone(),
            ),
        })
        .as_node();

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
            assert_eq!(attra, child);
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
            assert_eq!(attrc, child);
        }
        assert_eq!(Some(doc.clone()), elem2.owner_document());
        assert!(!elem2.has_child());
    }

    #[test]
    fn test_element_node_mut() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let mut elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();

        // NodeMut
        elem1.set_node_value("a").err().unwrap();
        let d = elem1
            .insert_before(doc.create_text_node("d").unwrap().as_node(), None)
            .unwrap();
        assert_eq!("data1d", elem1.as_string_value().unwrap());
        let e = elem1
            .insert_before(doc.create_text_node("e").unwrap().as_node(), Some(&d))
            .unwrap();
        assert_eq!("data1ed", elem1.as_string_value().unwrap());
        let _ = elem1
            .replace_child(doc.create_text_node("f").unwrap().as_node(), &e)
            .unwrap();
        assert_eq!("data1fd", elem1.as_string_value().unwrap());
        let d = elem1.remove_child(&d).unwrap();
        assert_eq!("data1f", elem1.as_string_value().unwrap());
        elem1.append_child(d).unwrap();
        assert_eq!("data1fd", elem1.as_string_value().unwrap());
    }

    #[test]
    fn test_element_as_node() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();
        let elem2 = root
            .get_elements_by_tag_name("elem2")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();
        let attra = elem1.get_attribute_node("a").unwrap();
        let data1 = XmlResolvedText::from(XmlText {
            data: info::XmlText::new(
                "data1",
                Some(doc.document.borrow().document_element().unwrap().into()),
                doc.document.clone(),
            ),
        })
        .as_node();

        // AsNode (elem1)
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
            assert_eq!(attra, child);
        }
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(node.has_child());
    }

    #[test]
    fn test_element_as_string_value() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();
        let elem2 = root
            .get_elements_by_tag_name("elem2")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();

        // AsStringValue
        assert_eq!("data1", elem1.as_string_value().unwrap());
        assert_eq!("", elem2.as_string_value().unwrap());
    }

    #[test]
    fn test_element_children() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();
        let data1 = XmlNode::ResolvedText(XmlResolvedText::from(XmlText {
            data: info::XmlText::new("data1", None, doc.document.clone()),
        }));

        // HasChild
        assert_eq!(vec![data1], elem1.children());
    }

    #[test]
    fn test_element_debug() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();

        // fmt::Debug
        assert_eq!("XmlElement { elem1 }", format!("{:?}", elem1));
    }

    #[test]
    fn test_element_display() {
        let (_, doc) = XmlDocument::from_raw(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let root = doc.document_element().unwrap();
        let elem1 = root
            .get_elements_by_tag_name("elem1")
            .item(0)
            .unwrap()
            .as_element()
            .unwrap();

        // fmt::Display
        assert_eq!("<elem1 a=\"b\">data1</elem1>", format!("{}", elem1));
    }

    #[test]
    fn test_text_split_text_attribute() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();
        let mut text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // TextMut
        let text2 = text.split_text(2).unwrap();

        assert_eq!(Some("te"), text.node_value().unwrap().as_deref());
        assert_eq!(Some("xt"), text2.node_value().unwrap().as_deref());
    }

    #[test]
    fn test_text_split_text_element() {
        let (_, doc) = XmlDocument::from_raw("<root>text</root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut text = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // TextMut
        let text2 = text.split_text(2).unwrap();

        assert_eq!(Some("te"), text.node_value().unwrap().as_deref());
        assert_eq!(Some("xt"), text2.node_value().unwrap().as_deref());
    }

    #[test]
    fn test_text_character_data() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();
        let text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // CharacterData
        assert_eq!("text", text.data().unwrap());
        assert_eq!(4, text.length());
        assert_eq!("ex", text.substring_data(1, 2));
    }

    #[test]
    fn test_text_character_data_mut() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let attr = doc
            .document_element()
            .unwrap()
            .get_attribute_node("a")
            .unwrap();
        let mut text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

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
    }

    #[test]
    fn test_text_node() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // Node
        assert_eq!("#text", text.node_name());
        assert_eq!(Some("text".to_string()), text.node_value().unwrap());
        assert_eq!(NodeType::Text, text.node_type());
        assert_eq!(None, text.parent_node());
        assert_eq!(XmlNodeList::empty(), text.child_nodes());
        assert_eq!(None, text.first_child());
        assert_eq!(None, text.last_child());
        assert_eq!(None, text.previous_sibling());
        assert_eq!(None, text.next_sibling());
        assert_eq!(None, text.attributes());
        assert_eq!(Some(doc.clone()), text.owner_document());
        assert!(!text.has_child());
    }

    #[test]
    fn test_text_node_mut() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let mut text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // Nodemut
        let e = root.clone().as_node();
        text.set_node_value("abc").unwrap();
        assert_eq!(Some("abc"), text.node_value().unwrap().as_deref());
        text.insert_before(e.clone(), Some(&e)).err().unwrap();
        text.replace_child(e.clone(), &e).err().unwrap();
        text.remove_child(&e).err().unwrap();
        text.append_child(e.clone()).err().unwrap();
        text.set_node_value("text").unwrap();
    }

    #[test]
    fn test_text_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // AsNode
        let node = text.as_node();
        assert_eq!("#text", node.node_name());
        assert_eq!(Some("text".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::Text, node.node_type());
        assert_eq!(None, node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());
    }

    #[test]
    fn test_text_as_string_value() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // AsStringValue
        assert_eq!("text", text.as_string_value().unwrap());
    }

    #[test]
    fn test_text_debug() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // fmt::Debug
        assert_eq!("XmlText { text }", format!("{:?}", text));
    }

    #[test]
    fn test_text_display() {
        let (_, doc) = XmlDocument::from_raw("<root a='text' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let text = attr.child_nodes().item(0).unwrap().as_text().unwrap();

        // fmt::Display
        assert_eq!("text", format!("{}", text));
    }

    #[test]
    fn test_comment_character_data() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

        // CharacterData
        assert_eq!(" comment ", comment.data().unwrap());
        assert_eq!(9, comment.length());
        assert_eq!("co", comment.substring_data(1, 2));
    }

    #[test]
    fn test_comment_character_data_mut() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

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
    }

    #[test]
    fn test_comment_node() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

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
    }

    #[test]
    fn test_comment_node_mut() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

        // Nodemut
        let e = root.clone().as_node();
        comment.set_node_value("abc").unwrap();
        assert_eq!(Some("abc"), comment.node_value().unwrap().as_deref());
        comment.insert_before(e.clone(), Some(&e)).err().unwrap();
        comment.replace_child(e.clone(), &e).err().unwrap();
        comment.remove_child(&e).err().unwrap();
        comment.append_child(e.clone()).err().unwrap();
        comment.set_node_value(" comment ").unwrap();
    }

    #[test]
    fn test_comment_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

        // AsNode
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
    }

    #[test]
    fn test_comment_as_string_value() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

        // AsStringValue
        assert_eq!(" comment ", comment.as_string_value().unwrap());
    }

    #[test]
    fn test_comment_debug() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

        // fmt::Debug
        assert_eq!("XmlComment {  comment  }", format!("{:?}", comment));
    }

    #[test]
    fn test_comment_display() {
        let (_, doc) = XmlDocument::from_raw("<root><!-- comment --></root>").unwrap();
        let root = doc.document_element().unwrap();
        let comment = root.child_nodes().item(0).unwrap().as_comment().unwrap();

        // fmt::Display
        assert_eq!("<!-- comment -->", format!("{}", comment));
    }

    #[test]
    fn test_cdata_split_text() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[cdata]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // TextMut
        let cdata2 = cdata.split_text(1).unwrap();

        assert_eq!(Some("c"), cdata.node_value().unwrap().as_deref());
        assert_eq!(Some("data"), cdata2.node_value().unwrap().as_deref());
    }

    #[test]
    fn test_cdata_character_data() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // CharacterData
        assert_eq!("&<>\"", cdata.data().unwrap());
        assert_eq!(4, cdata.length());
        assert_eq!("<>", cdata.substring_data(1, 2));
    }

    #[test]
    fn test_cdata_character_data_mut() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

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
    }

    #[test]
    fn test_cdata_node() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

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
    }

    #[test]
    fn test_cdata_node_mut() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // Nodemut
        let e = root.clone().as_node();
        cdata.set_node_value("abc").unwrap();
        assert_eq!(Some("abc"), cdata.node_value().unwrap().as_deref());
        cdata.insert_before(e.clone(), Some(&e)).err().unwrap();
        cdata.replace_child(e.clone(), &e).err().unwrap();
        cdata.remove_child(&e).err().unwrap();
        cdata.append_child(e.clone()).err().unwrap();
        cdata.set_node_value("&<>\"").unwrap();
    }

    #[test]
    fn test_cdata_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // AsNode
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
    }

    #[test]
    fn test_cdata_as_string_value() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // AsStringValue
        assert_eq!("&<>\"", cdata.as_string_value().unwrap());
    }

    #[test]
    fn test_cdata_debug() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // fmt::Debug
        assert_eq!(
            "XmlResolvedText { data: [CData(XmlCDataSection { &<>\" })] }",
            format!("{:?}", cdata)
        );
    }

    #[test]
    fn test_cdata_display() {
        let (_, doc) = XmlDocument::from_raw("<root><![CDATA[&<>\"]]></root>").unwrap();
        let root = doc.document_element().unwrap();
        let cdata = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // fmt::Display
        assert_eq!("<![CDATA[&<>\"]]>", format!("{}", cdata));
    }

    #[test]
    fn test_doctype_document_type() {
        let (_, doc) = XmlDocument::from_raw(
            "<!DOCTYPE root [<!NOTATION a SYSTEM 'b'><!ENTITY c 'd'>]><root />",
        )
        .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();

        // DocumentType
        assert_eq!("root", doctype.name());
        assert_eq!(1, doctype.entities().length());
        assert_eq!(1, doctype.notations().length());
    }

    #[test]
    fn test_doctype_node() {
        let (_, doc) = XmlDocument::from_raw(
            "<!DOCTYPE root [<!NOTATION a SYSTEM 'b'><!ENTITY c 'd'>]><root />",
        )
        .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let root = doc.document_element().unwrap();

        // Node
        assert_eq!("root", doctype.node_name());
        assert_eq!(None, doctype.node_value().unwrap());
        assert_eq!(NodeType::DocumentType, doctype.node_type());
        assert_eq!(Some(doc.as_node()), doctype.parent_node());
        assert_eq!(XmlNodeList::empty(), doctype.child_nodes());
        assert_eq!(None, doctype.first_child());
        assert_eq!(None, doctype.last_child());
        assert_eq!(None, doctype.previous_sibling());
        assert_eq!(Some(root.as_node()), doctype.next_sibling());
        assert_eq!(None, doctype.attributes());
        assert_eq!(Some(doc.clone()), doctype.owner_document());
        assert!(!doctype.has_child());
    }

    #[test]
    fn test_doctype_as_node() {
        let (_, doc) = XmlDocument::from_raw(
            "<!DOCTYPE root [<!NOTATION a SYSTEM 'b'><!ENTITY c 'd'>]><root />",
        )
        .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let root = doc.document_element().unwrap();

        // AsNode
        let node = doctype.as_node();
        assert_eq!("root", node.node_name());
        assert_eq!(None, node.node_value().unwrap());
        assert_eq!(NodeType::DocumentType, node.node_type());
        assert_eq!(Some(doc.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(Some(root.as_node()), node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());
    }

    #[test]
    fn test_doctype_debug() {
        let (_, doc) = XmlDocument::from_raw(
            "<!DOCTYPE root [<!NOTATION a SYSTEM 'b'><!ENTITY c 'd'>]><root />",
        )
        .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();

        // fmt::Debug
        assert_eq!("XmlDocumentType { root }", format!("{:?}", doctype));
    }

    #[test]
    fn test_doctype_display() {
        let (_, doc) = XmlDocument::from_raw(
            "<!DOCTYPE root [<!NOTATION a SYSTEM 'b'><!ENTITY c 'd'>]><root />",
        )
        .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();

        // fmt::Display
        assert_eq!(
            "<!DOCTYPE root [<!NOTATION a SYSTEM \"b\"><!ENTITY c \"d\">]>",
            format!("{}", doctype)
        );
    }

    #[test]
    fn test_notation_notation() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!NOTATION a PUBLIC 'b' 'c'>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let notation = doctype.notations().item(0).unwrap();

        // Notation
        assert_eq!("b", notation.public_id().unwrap());
        assert_eq!("c", notation.system_id().unwrap());
    }

    #[test]
    fn test_notation_node() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!NOTATION a PUBLIC 'b' 'c'>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let notation = doctype.notations().item(0).unwrap();

        // Node
        assert_eq!("a", notation.node_name());
        assert_eq!(None, notation.node_value().unwrap());
        assert_eq!(NodeType::Notation, notation.node_type());
        assert_eq!(None, notation.parent_node());
        assert_eq!(XmlNodeList::empty(), notation.child_nodes());
        assert_eq!(None, notation.first_child());
        assert_eq!(None, notation.last_child());
        assert_eq!(None, notation.previous_sibling());
        assert_eq!(None, notation.next_sibling());
        assert_eq!(None, notation.attributes());
        assert_eq!(Some(doc.clone()), notation.owner_document());
        assert!(!notation.has_child());
    }

    #[test]
    fn test_notation_as_node() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!NOTATION a PUBLIC 'b' 'c'>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let notation = doctype.notations().item(0).unwrap();

        // AsNode
        let node = notation.as_node();
        assert_eq!("a", node.node_name());
        assert_eq!(None, node.node_value().unwrap());
        assert_eq!(NodeType::Notation, node.node_type());
        assert_eq!(None, node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());
    }

    #[test]
    fn test_notation_debug() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!NOTATION a PUBLIC 'b' 'c'>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let notation = doctype.notations().item(0).unwrap();

        // fmt::Debug
        assert_eq!("XmlNotation { a }", format!("{:?}", notation));
    }

    #[test]
    fn test_notation_display() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!NOTATION a PUBLIC 'b' 'c'>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let notation = doctype.notations().item(0).unwrap();

        // fmt::Display
        assert_eq!("<!NOTATION a PUBLIC \"b\" \"c\">", format!("{}", notation));
    }

    #[test]
    fn test_entity_entity() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!ENTITY a PUBLIC 'b' 'c' NDATA d>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let entity = doctype.entities().item(0).unwrap();

        // Notation
        assert_eq!("b", entity.public_id().unwrap());
        assert_eq!("c", entity.system_id().unwrap());
        assert_eq!("d", entity.notation_name().unwrap());
    }

    #[test]
    fn test_entity_node() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!ENTITY a PUBLIC 'b' 'c' NDATA d>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let entity = doctype.entities().item(0).unwrap();

        // Node
        assert_eq!("a", entity.node_name());
        assert_eq!(None, entity.node_value().unwrap());
        assert_eq!(NodeType::Entity, entity.node_type());
        assert_eq!(None, entity.parent_node());
        assert_eq!(XmlNodeList::empty(), entity.child_nodes());
        assert_eq!(None, entity.first_child());
        assert_eq!(None, entity.last_child());
        assert_eq!(None, entity.previous_sibling());
        assert_eq!(None, entity.next_sibling());
        assert_eq!(None, entity.attributes());
        assert_eq!(Some(doc.clone()), entity.owner_document());
        assert!(!entity.has_child());
    }

    #[test]
    fn test_entity_as_node() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!ENTITY a PUBLIC 'b' 'c' NDATA d>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let entity = doctype.entities().item(0).unwrap();

        // AsNode
        let node = entity.as_node();
        assert_eq!("a", node.node_name());
        assert_eq!(None, node.node_value().unwrap());
        assert_eq!(NodeType::Entity, node.node_type());
        assert_eq!(None, node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());
    }

    #[test]
    fn test_entity_children() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!ENTITY a PUBLIC 'b' 'c' NDATA d>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let entity = doctype.entities().item(0).unwrap();

        // HasChild
        assert_eq!(0, entity.children().len());
    }

    #[test]
    fn test_entity_debug() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!ENTITY a PUBLIC 'b' 'c' NDATA d>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let entity = doctype.entities().item(0).unwrap();

        // fmt::Debug
        assert_eq!("XmlEntity { a }", format!("{:?}", entity));
    }

    #[test]
    fn test_entity_display() {
        let (_, doc) =
            XmlDocument::from_raw("<!DOCTYPE root [<!ENTITY a PUBLIC 'b' 'c' NDATA d>]><root />")
                .unwrap();
        let doctype = doc.child_nodes().item(0).unwrap().as_doctype().unwrap();
        let entity = doctype.entities().item(0).unwrap();

        // fmt::Display
        assert_eq!(
            "<!ENTITY a PUBLIC \"b\" \"c\" NDATA d>",
            format!("{}", entity)
        );
    }

    #[test]
    fn test_ref_node() {
        let (_, doc) = XmlDocument::from_raw("<root a='&amp;' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let eref = attr.child_nodes().item(0).unwrap().as_entity_ref().unwrap();

        // Node
        assert_eq!("amp", eref.node_name());
        assert_eq!(None, eref.node_value().unwrap());
        assert_eq!(NodeType::EntityReference, eref.node_type());
        assert_eq!(Some(attr.as_node()), eref.parent_node());
        assert_eq!(XmlNodeList::empty(), eref.child_nodes());
        assert_eq!(None, eref.first_child());
        assert_eq!(None, eref.last_child());
        assert_eq!(None, eref.previous_sibling());
        assert_eq!(None, eref.next_sibling());
        assert_eq!(None, eref.attributes());
        assert_eq!(Some(doc.clone()), eref.owner_document());
        assert!(!eref.has_child());
    }

    #[test]
    fn test_ref_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root a='&amp;' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let eref = attr.child_nodes().item(0).unwrap().as_entity_ref().unwrap();

        // AsNode
        let node = eref.as_node();
        assert_eq!("amp", node.node_name());
        assert_eq!(None, node.node_value().unwrap());
        assert_eq!(NodeType::EntityReference, node.node_type());
        assert_eq!(Some(attr.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());
    }

    #[test]
    fn test_ref_children() {
        let (_, doc) = XmlDocument::from_raw("<root a='&amp;' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let eref = attr.child_nodes().item(0).unwrap().as_entity_ref().unwrap();

        // HasChild
        assert_eq!(0, eref.children().len());
    }

    #[test]
    fn test_ref_debug() {
        let (_, doc) = XmlDocument::from_raw("<root a='&amp;' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let eref = attr.child_nodes().item(0).unwrap().as_entity_ref().unwrap();

        // fmt::Debug
        assert_eq!("XmlEntityReference { amp }", format!("{:?}", eref));
    }

    #[test]
    fn test_ref_display() {
        let (_, doc) = XmlDocument::from_raw("<root a='&amp;' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let eref = attr.child_nodes().item(0).unwrap().as_entity_ref().unwrap();

        // fmt::Display
        assert_eq!("&amp;", format!("{}", eref));
    }

    #[test]
    fn test_ref_impl() {
        let (_, doc) = XmlDocument::from_raw("<root a='&amp;' />").unwrap();
        let root = doc.document_element().unwrap();
        let attr = root.get_attribute_node("a").unwrap();
        let eref = attr.child_nodes().item(0).unwrap().as_entity_ref().unwrap();

        // XmlEntityReference
        assert_eq!("&", eref.value().unwrap());
    }

    #[test]
    fn test_pi_pi() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // ProcessingInstruction
        assert_eq!("a", pi.target());
        assert_eq!("b", pi.data());
    }

    #[test]
    fn test_pi_pi_mut() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // ProcessingInstructionMut
        pi.set_data("c").unwrap();
        assert_eq!("c", pi.data());
    }

    #[test]
    fn test_pi_node() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // Node
        assert_eq!("a", pi.node_name());
        assert_eq!(Some("b".to_string()), pi.node_value().unwrap());
        assert_eq!(NodeType::PI, pi.node_type());
        assert_eq!(Some(root.as_node()), pi.parent_node());
        assert_eq!(XmlNodeList::empty(), pi.child_nodes());
        assert_eq!(None, pi.first_child());
        assert_eq!(None, pi.last_child());
        assert_eq!(None, pi.previous_sibling());
        assert_eq!(None, pi.next_sibling());
        assert_eq!(None, pi.attributes());
        assert_eq!(Some(doc.clone()), pi.owner_document());
        assert!(!pi.has_child());
    }

    #[test]
    fn test_pi_node_mut() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let mut pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // NodeMut
        let e = root.clone().as_node();
        pi.set_node_value("c").unwrap();
        assert_eq!("c", pi.data());
        pi.insert_before(e.clone(), Some(&e)).err().unwrap();
        pi.replace_child(e.clone(), &e).err().unwrap();
        pi.remove_child(&e).err().unwrap();
        pi.append_child(e.clone()).err().unwrap();
        pi.set_node_value("b").unwrap();
    }

    #[test]
    fn test_pi_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // AsNode
        let node = pi.as_node();
        assert_eq!("a", node.node_name());
        assert_eq!(Some("b".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::PI, node.node_type());
        assert_eq!(Some(root.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(None, node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());
    }

    #[test]
    fn test_pi_as_expanded_name() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // AsExpandedName
        assert_eq!(
            Some(("a".to_string(), None, None)),
            pi.as_expanded_name().unwrap()
        );
    }

    #[test]
    fn test_pi_as_string_value() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // AsStringValue
        assert_eq!("b", pi.as_string_value().unwrap());
    }

    #[test]
    fn test_pi_debug() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // fmt::Debug
        assert_eq!("XmlProcessingInstruction { a }", format!("{:?}", pi));
    }

    #[test]
    fn test_pi_display() {
        let (_, doc) = XmlDocument::from_raw("<root><?a b?></root>").unwrap();
        let root = doc.document_element().unwrap();
        let pi = root.child_nodes().item(0).unwrap().as_pi().unwrap();

        // fmt::Display
        assert_eq!("<?a b?>", format!("{}", pi));
    }

    #[test]
    fn test_namespace_node() {
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
    }

    #[test]
    fn test_namespace_as_node() {
        let (_, doc) = XmlDocument::from_raw("<root xmlns:a='http://test/a'></root>").unwrap();
        let root = doc.document_element().unwrap();
        let namespaces = root.in_scope_namespace().unwrap();
        let ns = namespaces.first().unwrap();

        // AsNode
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
    }

    #[test]
    fn test_namespace_as_expanded_name() {
        let (_, doc) = XmlDocument::from_raw("<root xmlns:a='http://test/a'></root>").unwrap();
        let root = doc.document_element().unwrap();
        let namespaces = root.in_scope_namespace().unwrap();
        let ns = namespaces.first().unwrap();

        // AsStringValue
        assert_eq!(
            Some(("a".to_string(), None, None)),
            ns.as_expanded_name().unwrap()
        );
    }

    #[test]
    fn test_namespace_as_string_value() {
        let (_, doc) = XmlDocument::from_raw("<root xmlns:a='http://test/a'></root>").unwrap();
        let root = doc.document_element().unwrap();
        let namespaces = root.in_scope_namespace().unwrap();
        let ns = namespaces.first().unwrap();

        // AsStringValue
        assert_eq!("http://test/a", ns.as_string_value().unwrap());
    }

    #[test]
    fn test_namespace_debug() {
        let (_, doc) = XmlDocument::from_raw("<root xmlns:a='http://test/a'></root>").unwrap();
        let root = doc.document_element().unwrap();
        let namespaces = root.in_scope_namespace().unwrap();
        let ns = namespaces.first().unwrap();

        // fmt::Debug
        assert_eq!("XmlNamespace { http://test/a }", format!("{:?}", ns));
    }

    #[test]
    fn test_namespace_display() {
        let (_, doc) = XmlDocument::from_raw("<root xmlns:a='http://test/a'></root>").unwrap();
        let root = doc.document_element().unwrap();
        let namespaces = root.in_scope_namespace().unwrap();
        let ns = namespaces.first().unwrap();

        // fmt::Display
        assert_eq!("xmlns:a=\"http://test/a\"", format!("{}", ns));
    }

    #[test]
    fn test_namespace_impl() {
        let (_, doc) = XmlDocument::from_raw("<root xmlns:a='http://test/a'></root>").unwrap();
        let root = doc.document_element().unwrap();
        let namespaces = root.in_scope_namespace().unwrap();
        let ns = namespaces.first().unwrap();

        // XmlNamespace
        assert!(!ns.implicit());
    }

    #[test]
    fn test_resolved_text_character_data() {
        let (_, doc) =
            XmlDocument::from_raw("<root>a<![CDATA[b]]>c<a />&#x3042;d&amp;d</root>").unwrap();
        let root = doc.document_element().unwrap();
        let text = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // CharacterData
        assert_eq!("abc", text.data().unwrap());
        assert_eq!(3, text.length());
        assert_eq!("bc", text.substring_data(1, 2));

        let text = root
            .child_nodes()
            .item(2)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // CharacterData
        assert_eq!(4, text.length());
        assert_eq!("d&", text.substring_data(1, 2));
    }

    #[test]
    fn test_resolved_text_node() {
        let (_, doc) =
            XmlDocument::from_raw("<root>a<![CDATA[b]]>c<a />&#x3042;d&amp;d</root>").unwrap();
        let root = doc.document_element().unwrap();
        let text = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();
        let a = root.child_nodes().item(1).unwrap().as_element().unwrap();

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

        let text = root
            .child_nodes()
            .item(2)
            .unwrap()
            .as_resolved_text()
            .unwrap();

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
    }

    #[test]
    fn test_resolved_text_as_node() {
        let (_, doc) =
            XmlDocument::from_raw("<root>a<![CDATA[b]]>c<a />&#x3042;d&amp;d</root>").unwrap();
        let root = doc.document_element().unwrap();
        let text = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();
        let a = root.child_nodes().item(1).unwrap().as_element().unwrap();

        // AsNode
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

        let text = root
            .child_nodes()
            .item(2)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // AsNode
        let node = text.as_node();
        assert_eq!("#text", node.node_name());
        assert_eq!(Some("あd&d".to_string()), node.node_value().unwrap());
        assert_eq!(NodeType::Text, node.node_type());
        assert_eq!(Some(root.as_node()), node.parent_node());
        assert_eq!(XmlNodeList::empty(), node.child_nodes());
        assert_eq!(None, node.first_child());
        assert_eq!(None, node.last_child());
        assert_eq!(Some(a.as_node()), node.previous_sibling());
        assert_eq!(None, node.next_sibling());
        assert_eq!(None, node.attributes());
        assert_eq!(Some(doc.clone()), node.owner_document());
        assert!(!node.has_child());
    }

    #[test]
    fn test_resolved_text_as_string_value() {
        let (_, doc) =
            XmlDocument::from_raw("<root>a<![CDATA[b]]>c<a />&#x3042;d&amp;d</root>").unwrap();
        let root = doc.document_element().unwrap();
        let text = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // AsStringValue
        assert_eq!("abc", text.as_string_value().unwrap());

        let text = root
            .child_nodes()
            .item(2)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // AsStringValue
        assert_eq!("あd&d", text.as_string_value().unwrap());
    }

    #[test]
    fn test_resolved_text_display() {
        let (_, doc) =
            XmlDocument::from_raw("<root>a<![CDATA[b]]>c<a />&#x3042;d&amp;d</root>").unwrap();
        let root = doc.document_element().unwrap();
        let text = root
            .child_nodes()
            .item(0)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // fmt::Display
        assert_eq!("a<![CDATA[b]]>c", format!("{}", text));

        let text = root
            .child_nodes()
            .item(2)
            .unwrap()
            .as_resolved_text()
            .unwrap();

        // fmt::Display
        assert_eq!("&#x3042;d&amp;d", format!("{}", text));
    }
}

// -----------------------------------------------------------------------------------------------
