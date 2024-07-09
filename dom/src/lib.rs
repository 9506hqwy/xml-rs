// TODO: read only.

// -----------------------------------------------------------------------------------------------

pub trait DomImplementation {
    fn has_feature(&self, feature: &str, version: Option<&str>) -> bool;
}

// -----------------------------------------------------------------------------------------------

pub trait DocumentFragment<'a>: Node<'a> {}

// -----------------------------------------------------------------------------------------------

pub trait Document<'a>: Node<'a> {
    fn doc_type(&self) -> Option<XmlDocumentType<'a>>;

    fn implementation(&self) -> XmlDomImplementation;

    fn element(&self) -> XmlElement<'a>;

    fn get_elements_by_tag_name(&self, tag_name: &str) -> XmlNodeList<'a>;
}

// -----------------------------------------------------------------------------------------------

pub trait Node<'a> {
    fn node_name(&self) -> String;

    fn node_value(&self) -> Option<String>;

    fn node_type(&self) -> NodeType;

    fn parent_node(&self) -> Option<XmlNode<'a>>;

    fn child_nodes(&self) -> XmlNodeList<'a>;

    fn first_child(&self) -> Option<XmlNode<'a>>;

    fn last_child(&self) -> Option<XmlNode<'a>>;

    fn previous_sibling(&self) -> Option<XmlNode<'a>>;

    fn next_sibling(&self) -> Option<XmlNode<'a>>;

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>>;

    fn owner_document(&self) -> Option<XmlDocument<'a>>;

    fn has_child(&self) -> bool;
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

pub trait NodeList<'a> {
    fn item(&self, index: usize) -> Option<XmlNode<'a>>;

    fn length(&self) -> usize;
}

// -----------------------------------------------------------------------------------------------

pub trait NamedNodeMap<'a> {
    fn get_named_item(&self, name: &str) -> Option<XmlNode<'a>>;

    fn item(&self, index: usize) -> Option<XmlNode<'a>>;

    fn length(&self) -> usize;
}

// -----------------------------------------------------------------------------------------------

pub trait CharacterData<'a>: Node<'a> {
    fn data(&self) -> String;

    fn length(&self) -> usize;

    fn substring(&self, offset: usize, count: usize) -> String;
}

// -----------------------------------------------------------------------------------------------

pub trait Attr<'a>: Node<'a> {
    fn name(&self) -> String;

    fn specified(&self) -> bool;

    fn value(&self) -> String;
}

// -----------------------------------------------------------------------------------------------

pub trait Element<'a>: Node<'a> {
    fn get_attribute(&self, name: &str) -> String;

    fn get_attribute_node(&self, name: &str) -> Option<XmlAttr<'a>>;

    fn get_elements_by_tag_name(&self, tag_name: &str) -> XmlNodeList<'a>;
}

// -----------------------------------------------------------------------------------------------

pub trait Text<'a>: CharacterData<'a> {}

// -----------------------------------------------------------------------------------------------

pub trait Comment<'a>: CharacterData<'a> {}

// -----------------------------------------------------------------------------------------------

pub trait CDataSection<'a>: Text<'a> {}

// -----------------------------------------------------------------------------------------------

pub trait DocumentType<'a>: Node<'a> {
    fn name(&self) -> String;

    fn entities(&self) -> XmlNamedNodeMap<'a>;

    fn notations(&self) -> XmlNamedNodeMap<'a>;
}

// -----------------------------------------------------------------------------------------------

pub trait Notation<'a>: Node<'a> {
    fn public_id(&self) -> Option<String>;

    fn system_id(&self) -> Option<String>;
}

// -----------------------------------------------------------------------------------------------

pub trait Entity<'a>: Node<'a> {
    fn public_id(&self) -> Option<String>;

    fn system_id(&self) -> Option<String>;

    fn notation_name(&self) -> Option<String>;
}

// -----------------------------------------------------------------------------------------------

pub trait EntityReference<'a>: Node<'a> {}

// -----------------------------------------------------------------------------------------------

pub trait ProcessingInstruction<'a>: Node<'a> {
    fn target(&self) -> String;

    fn data(&self) -> String;
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum XmlNode<'a> {
    Element(XmlElement<'a>),
    Attribute(XmlAttr<'a>),
    Text(XmlText<'a>),
    CData(XmlCDataSection<'a>),
    EntityReference(XmlEntityReference<'a>),
    Entity(XmlEntity<'a>),
    PI(XmlProcessingInstruction<'a>),
    Comment(XmlComment<'a>),
    Document(XmlDocument<'a>),
    DocumentType(XmlDocumentType<'a>),
    DocumentFragment(XmlDocumentFragment<'a>),
    Notation(XmlNotation<'a>),
}

impl<'a> XmlNode<'a> {
    fn previous_sibling_child(&self, node: XmlNode<'a>) -> Option<XmlNode<'a>> {
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
            .skip_while(|&v| *v != node)
            .nth(1)
            .cloned()
    }

    fn next_sibling_child(&self, node: XmlNode<'a>) -> Option<XmlNode<'a>> {
        let children = match &self {
            XmlNode::Element(v) => v.children(),
            XmlNode::Attribute(v) => v.children(),
            XmlNode::EntityReference(v) => v.children(),
            XmlNode::Entity(v) => v.children(),
            XmlNode::Document(v) => v.children(),
            XmlNode::DocumentFragment(v) => v.children(),
            _ => return None,
        };

        children.iter().skip_while(|&v| *v != node).nth(1).cloned()
    }
}

// -----------------------------------------------------------------------------------------------

pub trait AsNode<'a> {
    fn as_node(&self) -> XmlNode<'a>;

    fn as_boxed_node(&self) -> Box<XmlNode<'a>> {
        Box::new(self.as_node())
    }
}

// -----------------------------------------------------------------------------------------------

trait HasChild<'a> {
    fn children(&self) -> Vec<XmlNode<'a>>;

    fn first_child_node(&self) -> Option<XmlNode<'a>> {
        let mut children = self.children();
        if children.is_empty() {
            None
        } else {
            Some(children.remove(0))
        }
    }

    fn last_child_node(&self) -> Option<XmlNode<'a>> {
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

#[derive(Clone, Debug)]
pub struct XmlDocumentFragment<'a> {
    document: &'a xml_parser::model::Document<'a>,
    owner: XmlDocument<'a>,
}

impl<'a> DocumentFragment<'a> for XmlDocumentFragment<'a> {}

impl<'a> Node<'a> for XmlDocumentFragment<'a> {
    fn node_name(&self) -> String {
        "#document-fragment".to_string()
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::DocumentFragment
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl<'a> AsNode<'a> for XmlDocumentFragment<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::DocumentFragment(self.clone())
    }
}

impl<'a> HasChild<'a> for XmlDocumentFragment<'a> {
    fn children(&self) -> Vec<XmlNode<'a>> {
        let mut nodes: Vec<XmlNode> = vec![];

        add_prolog_to_nodes(&self.document.prolog, None, self.owner.clone(), &mut nodes);

        nodes.push(XmlNode::Element(self.root_element()));

        for tail in &self.document.miscs {
            add_misc_to_nodes(tail, None, self.owner.clone(), &mut nodes);
        }

        nodes
    }
}

impl<'a> PartialEq for XmlDocumentFragment<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.document == other.document
    }
}

impl<'a> XmlDocumentFragment<'a> {
    fn root_element(&self) -> XmlElement<'a> {
        XmlElement {
            element: &self.document.element,
            parent: None,
            owner: self.owner.clone(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlDocument<'a> {
    document: &'a xml_parser::model::Document<'a>,
}

impl<'a> Document<'a> for XmlDocument<'a> {
    fn doc_type(&self) -> Option<XmlDocumentType<'a>> {
        self.document
            .prolog
            .declaration_doc
            .map(|declaration| XmlDocumentType {
                declaration,
                parent: Some(self.as_boxed_node()),
                owner: self.clone(),
            })
    }

    fn implementation(&self) -> XmlDomImplementation {
        XmlDomImplementation {}
    }

    fn element(&self) -> XmlElement<'a> {
        self.root_element()
    }

    fn get_elements_by_tag_name(&self, tag_name: &str) -> XmlNodeList<'a> {
        let mut nodes: Vec<XmlNode> = vec![];

        for v in self.root_element().elements_by_tag_name(tag_name) {
            nodes.push(XmlNode::Element(v))
        }

        XmlNodeList { nodes }
    }
}

impl<'a> Node<'a> for XmlDocument<'a> {
    fn node_name(&self) -> String {
        "#document".to_string()
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::Document
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        None::<XmlDocument>
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl<'a> AsNode<'a> for XmlDocument<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::Document(self.clone())
    }
}

impl<'a> HasChild<'a> for XmlDocument<'a> {
    fn children(&self) -> Vec<XmlNode<'a>> {
        let mut nodes: Vec<XmlNode> = vec![];

        add_prolog_to_nodes(&self.document.prolog, None, self.clone(), &mut nodes);

        nodes.push(XmlNode::Element(self.root_element()));

        for tail in &self.document.miscs {
            add_misc_to_nodes(tail, None, self.clone(), &mut nodes);
        }

        nodes
    }
}

impl<'a> PartialEq for XmlDocument<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.document == other.document
    }
}

impl<'a> From<&'a xml_parser::model::Document<'a>> for XmlDocument<'a> {
    fn from(value: &'a xml_parser::model::Document<'a>) -> Self {
        XmlDocument { document: value }
    }
}

impl<'a> XmlDocument<'a> {
    fn root_element(&self) -> XmlElement<'a> {
        XmlElement {
            element: &self.document.element,
            parent: None,
            owner: self.clone(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlNodeList<'a> {
    nodes: Vec<XmlNode<'a>>,
}

impl<'a> NodeList<'a> for XmlNodeList<'a> {
    fn item(&self, index: usize) -> Option<XmlNode<'a>> {
        let node = self.nodes.get(index);
        node.cloned()
    }

    fn length(&self) -> usize {
        self.nodes.len()
    }
}

impl<'a> XmlNodeList<'a> {
    fn empty() -> Self {
        XmlNodeList { nodes: vec![] }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct XmlNamedNodeMap<'a> {
    nodes: Vec<(String, XmlNode<'a>)>,
}

impl<'a> NamedNodeMap<'a> for XmlNamedNodeMap<'a> {
    fn get_named_item(&self, name: &str) -> Option<XmlNode<'a>> {
        let node = self.nodes.iter().find(|v| v.0 == name).map(|v| &v.1);
        node.cloned()
    }

    fn item(&self, index: usize) -> Option<XmlNode<'a>> {
        let node = self.nodes.get(index).map(|v| &v.1);
        node.cloned()
    }

    fn length(&self) -> usize {
        self.nodes.len()
    }
}

impl<'a> XmlNamedNodeMap<'a> {
    fn empty() -> Self {
        XmlNamedNodeMap { nodes: vec![] }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlAttr<'a> {
    attribute: &'a xml_parser::model::Attribute<'a>,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> Attr<'a> for XmlAttr<'a> {
    fn name(&self) -> String {
        match &self.attribute.name {
            xml_parser::model::AttributeName::QName(q) => match q {
                xml_nom::model::QName::Prefixed(n) => n.local_part.to_string(), // FIXME: namespace
                xml_nom::model::QName::Unprefixed(n) => n.to_string(),
            },
            _ => unreachable!(),
        }
    }

    fn specified(&self) -> bool {
        self.parent.is_some()
    }

    fn value(&self) -> String {
        let mut v = String::new();

        for value in &self.attribute.value {
            match value {
                xml_parser::model::AttributeValue::Reference(_) => {
                    unimplemented!("AttributeValue::Reference")
                }
                xml_parser::model::AttributeValue::Text(t) => v.push_str(t),
            }
        }

        v
    }
}

impl<'a> Node<'a> for XmlAttr<'a> {
    fn node_name(&self) -> String {
        self.name()
    }

    fn node_value(&self) -> Option<String> {
        Some(self.value())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Attribute
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl<'a> AsNode<'a> for XmlAttr<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::Attribute(self.clone())
    }
}

impl<'a> HasChild<'a> for XmlAttr<'a> {
    fn children(&self) -> Vec<XmlNode<'a>> {
        let mut nodes: Vec<XmlNode> = vec![];

        for v in &self.attribute.value {
            match v {
                xml_parser::model::AttributeValue::Reference(reference) => {
                    nodes.push(XmlNode::EntityReference(XmlEntityReference {
                        reference,
                        parent: Some(self.as_boxed_node()),
                        owner: self.owner.clone(),
                    }));
                }
                xml_parser::model::AttributeValue::Text(data) => {
                    if !data.is_empty() {
                        nodes.push(XmlNode::Text(XmlText {
                            data,
                            parent: Some(self.as_boxed_node()),
                            owner: self.owner.clone(),
                        }));
                    }
                }
            }
        }

        nodes
    }
}

impl<'a> PartialEq for XmlAttr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.attribute == other.attribute
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlElement<'a> {
    element: &'a xml_parser::model::Element<'a>,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> Element<'a> for XmlElement<'a> {
    fn get_attribute(&self, name: &str) -> String {
        let attr = self.get_attribute_node(name);
        attr.map(|v| v.value()).unwrap_or_default()
    }

    fn get_attribute_node(&self, name: &str) -> Option<XmlAttr<'a>> {
        self.element
            .attributes
            .iter()
            .find(|v| {
                match &v.name {
                    xml_parser::model::AttributeName::DefaultNamespace => false,
                    xml_parser::model::AttributeName::Namespace(_) => false,
                    xml_parser::model::AttributeName::QName(q) => match q {
                        xml_nom::model::QName::Prefixed(n) => n.local_part == name, // FIXME: namespace
                        xml_nom::model::QName::Unprefixed(n) => *n == name,
                    },
                }
            })
            .map(|v| XmlAttr {
                attribute: v,
                parent: Some(self.as_boxed_node()),
                owner: self.owner.clone(),
            })
    }

    fn get_elements_by_tag_name(&self, tag_name: &str) -> XmlNodeList<'a> {
        let nodes = self
            .elements_by_tag_name(tag_name)
            .iter()
            .map(|v| v.as_node())
            .collect();
        XmlNodeList { nodes }
    }
}

impl<'a> Node<'a> for XmlElement<'a> {
    fn node_name(&self) -> String {
        match &self.element.name {
            xml_nom::model::QName::Prefixed(n) => n.local_part.to_string(), // FIXME: namespace
            xml_nom::model::QName::Unprefixed(n) => n.to_string(),
        }
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::Element
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList {
            nodes: self.children(),
        }
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        self.first_child_node()
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        self.last_child_node()
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        let nodes = self
            .element
            .attributes
            .iter()
            .map(|attribute| XmlAttr {
                attribute,
                parent: Some(self.as_boxed_node()),
                owner: self.owner.clone(),
            })
            .map(|v| (v.name(), v.as_node()))
            .collect();

        Some(XmlNamedNodeMap { nodes })
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        self.has_child_node()
    }
}

impl<'a> AsNode<'a> for XmlElement<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::Element(self.clone())
    }
}

impl<'a> HasChild<'a> for XmlElement<'a> {
    fn children(&self) -> Vec<XmlNode<'a>> {
        let mut nodes: Vec<XmlNode<'a>> = vec![];

        if let Some(content) = &self.element.content {
            if let Some(data) = content.head {
                if !data.is_empty() {
                    nodes.push(XmlNode::Text(XmlText {
                        data,
                        parent: Some(self.as_boxed_node()),
                        owner: self.owner.clone(),
                    }))
                }
            }

            for cell in &content.children {
                match &cell.child {
                    xml_parser::model::Contents::Element(element) => {
                        nodes.push(XmlNode::Element(XmlElement {
                            element,
                            parent: Some(self.as_boxed_node()),
                            owner: self.owner.clone(),
                        }))
                    }
                    xml_parser::model::Contents::Reference(reference) => {
                        nodes.push(XmlNode::EntityReference(XmlEntityReference {
                            reference,
                            parent: Some(self.as_boxed_node()),
                            owner: self.owner.clone(),
                        }))
                    }
                    xml_parser::model::Contents::CData(data) => {
                        nodes.push(XmlNode::CData(XmlCDataSection {
                            data: data.value,
                            parent: Some(self.as_boxed_node()),
                            owner: self.owner.clone(),
                        }))
                    }
                    xml_parser::model::Contents::PI(pi) => {
                        nodes.push(XmlNode::PI(XmlProcessingInstruction {
                            pi,
                            parent: Some(self.as_boxed_node()),
                            owner: self.owner.clone(),
                        }))
                    }
                    xml_parser::model::Contents::Comment(data) => {
                        nodes.push(XmlNode::Comment(XmlComment {
                            data: data.value,
                            parent: Some(self.as_boxed_node()),
                            owner: self.owner.clone(),
                        }))
                    }
                }

                if let Some(data) = cell.tail {
                    if !data.is_empty() {
                        nodes.push(XmlNode::Text(XmlText {
                            data,
                            parent: Some(self.as_boxed_node()),
                            owner: self.owner.clone(),
                        }))
                    }
                }
            }
        }

        nodes
    }
}

impl<'a> PartialEq for XmlElement<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

impl<'a> XmlElement<'a> {
    fn elements_by_tag_name(&self, tag_name: &str) -> Vec<XmlElement<'a>> {
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

#[derive(Clone, Debug)]
pub struct XmlText<'a> {
    data: &'a str,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> Text<'a> for XmlText<'a> {}

impl<'a> CharacterData<'a> for XmlText<'a> {
    fn data(&self) -> String {
        self.data.to_string()
    }

    fn length(&self) -> usize {
        self.data.len()
    }

    fn substring(&self, offset: usize, count: usize) -> String {
        self.data[offset..(offset + count)].to_string()
    }
}

impl<'a> Node<'a> for XmlText<'a> {
    fn node_name(&self) -> String {
        "#text".to_string()
    }

    fn node_value(&self) -> Option<String> {
        Some(self.data())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Text
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl<'a> AsNode<'a> for XmlText<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::Text(self.clone())
    }
}

impl<'a> PartialEq for XmlText<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlComment<'a> {
    data: &'a str,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> Comment<'a> for XmlComment<'a> {}

impl<'a> CharacterData<'a> for XmlComment<'a> {
    fn data(&self) -> String {
        self.data.to_string()
    }

    fn length(&self) -> usize {
        self.data.len()
    }

    fn substring(&self, offset: usize, count: usize) -> String {
        self.data[offset..(offset + count)].to_string()
    }
}

impl<'a> Node<'a> for XmlComment<'a> {
    fn node_name(&self) -> String {
        "#comment".to_string()
    }

    fn node_value(&self) -> Option<String> {
        Some(self.data())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Comment
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl<'a> AsNode<'a> for XmlComment<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::Comment(self.clone())
    }
}

impl<'a> PartialEq for XmlComment<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlCDataSection<'a> {
    data: &'a str,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> CDataSection<'a> for XmlCDataSection<'a> {}

impl<'a> Text<'a> for XmlCDataSection<'a> {}

impl<'a> CharacterData<'a> for XmlCDataSection<'a> {
    fn data(&self) -> String {
        self.data.to_string()
    }

    fn length(&self) -> usize {
        self.data.len()
    }

    fn substring(&self, offset: usize, count: usize) -> String {
        self.data[offset..(offset + count)].to_string()
    }
}

impl<'a> Node<'a> for XmlCDataSection<'a> {
    fn node_name(&self) -> String {
        "#cdata-section".to_string()
    }

    fn node_value(&self) -> Option<String> {
        Some(self.data())
    }

    fn node_type(&self) -> NodeType {
        NodeType::CData
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl<'a> AsNode<'a> for XmlCDataSection<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::CData(self.clone())
    }
}

impl<'a> PartialEq for XmlCDataSection<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlDocumentType<'a> {
    declaration: &'a str,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> DocumentType<'a> for XmlDocumentType<'a> {
    fn name(&self) -> String {
        // TODO:
        "".to_string()
    }

    fn entities(&self) -> XmlNamedNodeMap<'a> {
        // TODO:
        XmlNamedNodeMap::empty()
    }

    fn notations(&self) -> XmlNamedNodeMap<'a> {
        // TODO:
        XmlNamedNodeMap::empty()
    }
}

impl<'a> Node<'a> for XmlDocumentType<'a> {
    fn node_name(&self) -> String {
        self.name()
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::DocumentType
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl<'a> AsNode<'a> for XmlDocumentType<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::DocumentType(self.clone())
    }
}

impl<'a> PartialEq for XmlDocumentType<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlNotation<'a> {
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> Notation<'a> for XmlNotation<'a> {
    fn public_id(&self) -> Option<String> {
        // TODO:
        None
    }

    fn system_id(&self) -> Option<String> {
        // TODO:
        None
    }
}

impl<'a> Node<'a> for XmlNotation<'a> {
    fn node_name(&self) -> String {
        // TODO:
        "".to_string()
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::Notation
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl<'a> AsNode<'a> for XmlNotation<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::Notation(self.clone())
    }
}

impl<'a> PartialEq for XmlNotation<'a> {
    fn eq(&self, _other: &Self) -> bool {
        // TODO:
        false
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlEntity<'a> {
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> Entity<'a> for XmlEntity<'a> {
    fn public_id(&self) -> Option<String> {
        // TODO:
        None
    }

    fn system_id(&self) -> Option<String> {
        // TODO:
        None
    }

    fn notation_name(&self) -> Option<String> {
        // TODO:
        None
    }
}

impl<'a> Node<'a> for XmlEntity<'a> {
    fn node_name(&self) -> String {
        // TODO:
        "".to_string()
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::Entity
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        // TODO:
        XmlNodeList { nodes: vec![] }
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        // TODO:
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        // TODO:
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        // TODO:
        false
    }
}

impl<'a> AsNode<'a> for XmlEntity<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::Entity(self.clone())
    }
}

impl<'a> HasChild<'a> for XmlEntity<'a> {
    fn children(&self) -> Vec<XmlNode<'a>> {
        // TODO:
        vec![]
    }
}

impl<'a> PartialEq for XmlEntity<'a> {
    fn eq(&self, _other: &Self) -> bool {
        // TODO:
        false
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlEntityReference<'a> {
    reference: &'a xml_parser::model::Reference<'a>,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> EntityReference<'a> for XmlEntityReference<'a> {}

impl<'a> Node<'a> for XmlEntityReference<'a> {
    fn node_name(&self) -> String {
        // TODO:
        "".to_string()
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::EntityReference
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        // TODO
        XmlNodeList { nodes: vec![] }
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        // TODO:
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        // TODO:
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        // TODO:
        false
    }
}

impl<'a> AsNode<'a> for XmlEntityReference<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::EntityReference(self.clone())
    }
}

impl<'a> HasChild<'a> for XmlEntityReference<'a> {
    fn children(&self) -> Vec<XmlNode<'a>> {
        // TODO:
        vec![]
    }
}

impl<'a> PartialEq for XmlEntityReference<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct XmlProcessingInstruction<'a> {
    pi: &'a xml_parser::model::PI<'a>,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
}

impl<'a> ProcessingInstruction<'a> for XmlProcessingInstruction<'a> {
    fn target(&self) -> String {
        self.pi.target.to_string()
    }

    fn data(&self) -> String {
        self.pi.value.map(|v| v.to_string()).unwrap_or_default()
    }
}

impl<'a> Node<'a> for XmlProcessingInstruction<'a> {
    fn node_name(&self) -> String {
        self.target()
    }

    fn node_value(&self) -> Option<String> {
        None
    }

    fn node_type(&self) -> NodeType {
        NodeType::PI
    }

    fn parent_node(&self) -> Option<XmlNode<'a>> {
        self.parent.as_ref().map(|v| *v.clone())
    }

    fn child_nodes(&self) -> XmlNodeList<'a> {
        XmlNodeList::empty()
    }

    fn first_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn last_child(&self) -> Option<XmlNode<'a>> {
        None
    }

    fn previous_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.previous_sibling_child(self.as_node()))
    }

    fn next_sibling(&self) -> Option<XmlNode<'a>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.next_sibling_child(self.as_node()))
    }

    fn attributes(&self) -> Option<XmlNamedNodeMap<'a>> {
        None::<XmlNamedNodeMap>
    }

    fn owner_document(&self) -> Option<XmlDocument<'a>> {
        Some(self.owner.clone())
    }

    fn has_child(&self) -> bool {
        false
    }
}

impl<'a> AsNode<'a> for XmlProcessingInstruction<'a> {
    fn as_node(&self) -> XmlNode<'a> {
        XmlNode::PI(self.clone())
    }
}

impl<'a> PartialEq for XmlProcessingInstruction<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.pi == other.pi
    }
}

// -----------------------------------------------------------------------------------------------

fn add_misc_to_nodes<'a>(
    misc: &'a xml_parser::model::Misc<'a>,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
    nodes: &mut Vec<XmlNode<'a>>,
) {
    match misc {
        xml_parser::model::Misc::Comment(c) => {
            nodes.push(XmlNode::Comment(XmlComment {
                data: c.value,
                parent,
                owner,
            }));
        }
        xml_parser::model::Misc::PI(p) => {
            nodes.push(XmlNode::PI(XmlProcessingInstruction {
                pi: p,
                parent,
                owner,
            }));
        }
        _ => {}
    }
}

fn add_prolog_to_nodes<'a>(
    prolog: &'a xml_parser::model::Prolog<'a>,
    parent: Option<Box<XmlNode<'a>>>,
    owner: XmlDocument<'a>,
    nodes: &mut Vec<XmlNode<'a>>,
) {
    for head in &prolog.heads {
        add_misc_to_nodes(head, parent.clone(), owner.clone(), nodes);
    }

    if let Some(declaration) = prolog.declaration_doc {
        let parent = parent.clone();
        let owner = owner.clone();
        nodes.push(XmlNode::DocumentType(XmlDocumentType {
            declaration,
            parent,
            owner,
        }));
    }

    for tail in &prolog.tails {
        add_misc_to_nodes(tail, parent.clone(), owner.clone(), nodes);
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
        let (_, document) = xml_parser::document("<root></root>").unwrap();

        let root = XmlNode::Element(XmlElement {
            element: &document.element,
            parent: Some(XmlDocument::from(&document).as_boxed_node()),
            owner: XmlDocument::from(&document),
        });

        let m = XmlDocumentFragment {
            document: &document,
            owner: XmlDocument::from(&document),
        };

        // Node
        assert_eq!("#document-fragment", m.node_name());
        assert_eq!(None, m.node_value());
        assert_eq!(NodeType::DocumentFragment, m.node_type());
        assert_eq!(None, m.parent_node());
        assert_eq!(
            XmlNodeList {
                nodes: vec![root.clone()]
            },
            m.child_nodes()
        );
        assert_eq!(Some(root.clone()), m.first_child());
        assert_eq!(Some(root.clone()), m.last_child());
        assert_eq!(None, m.previous_sibling());
        assert_eq!(None, m.next_sibling());
        assert_eq!(None, m.attributes());
        assert_eq!(Some(XmlDocument::from(&document)), m.owner_document());
        assert!(m.has_child());
    }

    #[test]
    fn test_document() {
        let (_, document) = xml_parser::document("<root></root>").unwrap();

        let elem = XmlElement {
            element: &document.element,
            parent: Some(XmlDocument::from(&document).as_boxed_node()),
            owner: XmlDocument::from(&document),
        };
        let root = XmlNode::Element(elem.clone());

        let m = XmlDocument::from(&document);

        // Document
        assert_eq!(None, m.doc_type());
        assert_eq!(XmlDomImplementation {}, m.implementation());
        assert_eq!(elem, m.element());
        assert_eq!(
            XmlNodeList {
                nodes: vec![root.clone()]
            },
            m.get_elements_by_tag_name("root")
        );

        // Node
        assert_eq!("#document", m.node_name());
        assert_eq!(None, m.node_value());
        assert_eq!(NodeType::Document, m.node_type());
        assert_eq!(None, m.parent_node());
        assert_eq!(
            XmlNodeList {
                nodes: vec![root.clone()]
            },
            m.child_nodes()
        );
        assert_eq!(Some(root.clone()), m.first_child());
        assert_eq!(Some(root.clone()), m.last_child());
        assert_eq!(None, m.previous_sibling());
        assert_eq!(None, m.next_sibling());
        assert_eq!(None, m.attributes());
        assert_eq!(None, m.owner_document());
        assert!(m.has_child());
    }

    #[test]
    fn test_attr() {
        let (_, document) = xml_parser::document("<root a='b'></root>").unwrap();
        let doc = XmlDocument::from(&document);
        let elem = doc.element();
        let attr = elem.get_attribute_node("a").unwrap();
        let text = XmlNode::Text(XmlText {
            data: "b",
            parent: Some(attr.as_boxed_node()),
            owner: doc.clone(),
        });

        // Attr
        assert!(attr.specified());

        // Node
        assert_eq!("a", attr.node_name());
        assert_eq!(Some("b".to_string()), attr.node_value());
        assert_eq!(NodeType::Attribute, attr.node_type());
        assert_eq!(None, attr.parent_node());
        assert_eq!(
            XmlNodeList {
                nodes: vec![text.clone()]
            },
            attr.child_nodes()
        );
        assert_eq!(Some(text.clone()), attr.first_child());
        assert_eq!(Some(text.clone()), attr.last_child());
        assert_eq!(None, attr.previous_sibling());
        assert_eq!(None, attr.next_sibling());
        assert_eq!(None, attr.attributes());
        assert_eq!(Some(doc.clone()), attr.owner_document());
        assert!(attr.has_child());
    }

    #[test]
    fn test_element() {
        let (_, document) = xml_parser::document(
            "<root><elem1 a=\"b\">data1</elem1><elem2 c=\"d\"></elem2></root>",
        )
        .unwrap();
        let doc = XmlDocument::from(&document);
        let root = doc.element();
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
        let data1 = XmlNode::Text(XmlText {
            data: "data1",
            parent: Some(elem1.as_boxed_node()),
            owner: doc.clone(),
        });

        // Element
        assert_eq!("b", elem1.get_attribute("a"));
        assert_eq!(Some(attra.clone()), elem1.get_attribute_node("a"));

        // Node (elem1)
        assert_eq!("elem1", elem1.node_name());
        assert_eq!(None, elem1.node_value());
        assert_eq!(NodeType::Element, elem1.node_type());
        assert_eq!(Some(root.as_node()), elem1.parent_node());
        assert_eq!(
            XmlNodeList {
                nodes: vec![data1.clone()]
            },
            elem1.child_nodes()
        );
        assert_eq!(Some(data1.clone()), elem1.first_child());
        assert_eq!(Some(data1.clone()), elem1.last_child());
        assert_eq!(None, elem1.previous_sibling());
        assert_eq!(Some(elem2.as_node()), elem1.next_sibling());
        assert_eq!(
            Some(XmlNamedNodeMap {
                nodes: vec![("a".to_string(), attra.clone().as_node())]
            }),
            elem1.attributes()
        );
        assert_eq!(Some(doc.clone()), elem1.owner_document());
        assert!(elem1.has_child());

        // Node (elem2)
        assert_eq!("elem2", elem2.node_name());
        assert_eq!(None, elem2.node_value());
        assert_eq!(NodeType::Element, elem2.node_type());
        assert_eq!(Some(root.as_node()), elem2.parent_node());
        assert_eq!(XmlNodeList { nodes: vec![] }, elem2.child_nodes());
        assert_eq!(None, elem2.first_child());
        assert_eq!(None, elem2.last_child());
        assert_eq!(Some(elem1.as_node()), elem2.previous_sibling());
        assert_eq!(None, elem2.next_sibling());
        assert_eq!(
            Some(XmlNamedNodeMap {
                nodes: vec![("c".to_string(), attrc.clone().as_node())]
            }),
            elem2.attributes()
        );
        assert_eq!(Some(doc.clone()), elem2.owner_document());
        assert!(!elem2.has_child());
    }

    #[test]
    fn test_text() {
        let (_, document) = xml_parser::document("<root>text</root>").unwrap();
        let doc = XmlDocument::from(&document);
        let root = doc.element();
        let text = if let XmlNode::Text(e) = root.child_nodes().item(0).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(4, text.length());
        assert_eq!("ex", text.substring(1, 2));

        // Node
        assert_eq!("#text", text.node_name());
        assert_eq!(Some("text".to_string()), text.node_value());
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
    }

    #[test]
    fn test_comment() {
        let (_, document) = xml_parser::document("<root><!-- comment --></root>").unwrap();
        let doc = XmlDocument::from(&document);
        let root = doc.element();
        let comment = if let XmlNode::Comment(e) = root.child_nodes().item(0).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(9, comment.length());
        assert_eq!("co", comment.substring(1, 2));

        // Node
        assert_eq!("#comment", comment.node_name());
        assert_eq!(Some(" comment ".to_string()), comment.node_value());
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
    fn test_cdata() {
        let (_, document) = xml_parser::document("<root><![CDATA[&<>\"]]></root>").unwrap();
        let doc = XmlDocument::from(&document);
        let root = doc.element();
        let cdata = if let XmlNode::CData(e) = root.child_nodes().item(0).unwrap() {
            e.clone()
        } else {
            unreachable!()
        };

        // CharacterData
        assert_eq!(4, cdata.length());
        assert_eq!("<>", cdata.substring(1, 2));

        // Node
        assert_eq!("#cdata-section", cdata.node_name());
        assert_eq!(Some("&<>\"".to_string()), cdata.node_value());
        assert_eq!(NodeType::CData, cdata.node_type());
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
}

// -----------------------------------------------------------------------------------------------
