use xml_nom::model::QName;

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AdditiveExpr<'a> {
    operand: MultiplicativeExpr<'a>,
    operations: Vec<(AdditiveOperator, MultiplicativeExpr<'a>)>,
}

impl<'a> From<MultiplicativeExpr<'a>> for AdditiveExpr<'a> {
    fn from(value: MultiplicativeExpr<'a>) -> Self {
        AdditiveExpr::from((value, vec![]))
    }
}

impl<'a>
    From<(
        MultiplicativeExpr<'a>,
        Vec<(AdditiveOperator, MultiplicativeExpr<'a>)>,
    )> for AdditiveExpr<'a>
{
    fn from(
        value: (
            MultiplicativeExpr<'a>,
            Vec<(AdditiveOperator, MultiplicativeExpr<'a>)>,
        ),
    ) -> Self {
        let (operand, operations) = value;
        AdditiveExpr {
            operand,
            operations,
        }
    }
}

impl<'a> AdditiveExpr<'a> {
    pub fn operand(&self) -> &MultiplicativeExpr<'a> {
        &self.operand
    }

    pub fn operations(&self) -> &[(AdditiveOperator, MultiplicativeExpr<'a>)] {
        self.operations.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AdditiveOperator {
    #[default]
    Add,
    Sub,
}

impl From<&str> for AdditiveOperator {
    fn from(value: &str) -> Self {
        match value {
            "+" => AdditiveOperator::Add,
            "-" => AdditiveOperator::Sub,
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AndExpr<'a> {
    operands: Vec<EqualityExpr<'a>>,
}

impl<'a> From<EqualityExpr<'a>> for AndExpr<'a> {
    fn from(value: EqualityExpr<'a>) -> Self {
        AndExpr::from(vec![value])
    }
}

impl<'a> From<Vec<EqualityExpr<'a>>> for AndExpr<'a> {
    fn from(value: Vec<EqualityExpr<'a>>) -> Self {
        AndExpr { operands: value }
    }
}

impl<'a> AndExpr<'a> {
    pub fn operands(&self) -> &[EqualityExpr<'a>] {
        self.operands.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

pub type Argument<'a> = Expr<'a>;

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AxisName {
    Ancestor,
    AncestorOrSelf,
    Attribute,
    Child,
    Descendant,
    DescendantOrSelf,
    Following,
    FollowingSibling,
    Namespace,
    Parent,
    Preceding,
    PrecedingSibling,
    #[default]
    Current, // Self
}

impl From<&str> for AxisName {
    fn from(value: &str) -> Self {
        match value {
            "ancestor" => AxisName::Ancestor,
            "ancestor-or-self" => AxisName::AncestorOrSelf,
            "attribute" => AxisName::Attribute,
            "child" => AxisName::Child,
            "descendant" => AxisName::Descendant,
            "descendant-or-self" => AxisName::DescendantOrSelf,
            "following" => AxisName::Following,
            "following-sibling" => AxisName::FollowingSibling,
            "namespace" => AxisName::Namespace,
            "parent" => AxisName::Parent,
            "preceding" => AxisName::Preceding,
            "preceding-sibling" => AxisName::PrecedingSibling,
            "self" => AxisName::Current,
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum AxisSpecifier {
    Name(AxisName),
    Abbreviated(String),
}

impl Default for AxisSpecifier {
    fn default() -> AxisSpecifier {
        AxisSpecifier::Name(AxisName::Child)
    }
}

impl From<AxisName> for AxisSpecifier {
    fn from(value: AxisName) -> Self {
        AxisSpecifier::Name(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EqualityExpr<'a> {
    operand: RelationalExpr<'a>,
    operations: Vec<(EqualityOperator, RelationalExpr<'a>)>,
}

impl<'a> From<RelationalExpr<'a>> for EqualityExpr<'a> {
    fn from(value: RelationalExpr<'a>) -> Self {
        EqualityExpr::from((value, vec![]))
    }
}

impl<'a>
    From<(
        RelationalExpr<'a>,
        Vec<(EqualityOperator, RelationalExpr<'a>)>,
    )> for EqualityExpr<'a>
{
    fn from(
        value: (
            RelationalExpr<'a>,
            Vec<(EqualityOperator, RelationalExpr<'a>)>,
        ),
    ) -> Self {
        let (operand, operations) = value;
        EqualityExpr {
            operand,
            operations,
        }
    }
}

impl<'a> EqualityExpr<'a> {
    pub fn operand(&self) -> &RelationalExpr<'a> {
        &self.operand
    }

    pub fn operations(&self) -> &[(EqualityOperator, RelationalExpr<'a>)] {
        self.operations.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum EqualityOperator {
    #[default]
    Equal,
    NotEqual,
}

impl From<&str> for EqualityOperator {
    fn from(value: &str) -> Self {
        match value {
            "=" => EqualityOperator::Equal,
            "!=" => EqualityOperator::NotEqual,
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

pub type Expr<'a> = OrExpr<'a>;

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilterExpr<'a> {
    primary: PrimaryExpr<'a>,
    predicates: Vec<PredicateExpr<'a>>,
}

impl<'a> From<PrimaryExpr<'a>> for FilterExpr<'a> {
    fn from(value: PrimaryExpr<'a>) -> Self {
        FilterExpr::from((value, vec![]))
    }
}

impl<'a> From<(PrimaryExpr<'a>, Vec<PredicateExpr<'a>>)> for FilterExpr<'a> {
    fn from(value: (PrimaryExpr<'a>, Vec<PredicateExpr<'a>>)) -> Self {
        let (primary, predicates) = value;
        FilterExpr {
            primary,
            predicates,
        }
    }
}

impl<'a> FilterExpr<'a> {
    pub fn primary(&self) -> &PrimaryExpr<'a> {
        &self.primary
    }

    pub fn predicates(&self) -> &[PredicateExpr<'a>] {
        self.predicates.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FunctionCall<'a> {
    name: QName<'a>,
    args: Vec<Argument<'a>>,
}

impl<'a> From<QName<'a>> for FunctionCall<'a> {
    fn from(value: QName<'a>) -> Self {
        FunctionCall::from((value, vec![]))
    }
}

impl<'a> From<(QName<'a>, Vec<Argument<'a>>)> for FunctionCall<'a> {
    fn from(value: (QName<'a>, Vec<Argument<'a>>)) -> Self {
        let (name, args) = value;
        FunctionCall { name, args }
    }
}

impl<'a> FunctionCall<'a> {
    pub fn name(&self) -> &QName<'a> {
        &self.name
    }

    pub fn args(&self) -> &[Argument<'a>] {
        self.args.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelativeLocationPath<'a> {
    operand: Step<'a>,
    operations: Vec<(LocationPathOperator, Step<'a>)>,
}

impl<'a> From<Step<'a>> for RelativeLocationPath<'a> {
    fn from(value: Step<'a>) -> Self {
        RelativeLocationPath::from((value, vec![]))
    }
}

impl<'a> From<(Step<'a>, Vec<(LocationPathOperator, Step<'a>)>)> for RelativeLocationPath<'a> {
    fn from(value: (Step<'a>, Vec<(LocationPathOperator, Step<'a>)>)) -> Self {
        let (operand, operations) = value;
        RelativeLocationPath {
            operand,
            operations,
        }
    }
}

impl<'a> RelativeLocationPath<'a> {
    pub fn operand(&self) -> &Step<'a> {
        &self.operand
    }

    pub fn operations(&self) -> &[(LocationPathOperator, Step<'a>)] {
        self.operations.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LocationPathOperator {
    #[default]
    Current,
    DescendantOrSelfNode,
}

impl From<&str> for LocationPathOperator {
    fn from(value: &str) -> Self {
        match value {
            "/" => LocationPathOperator::Current,
            "//" => LocationPathOperator::DescendantOrSelfNode,
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiplicativeExpr<'a> {
    operand: UnaryExpr<'a>,
    operations: Vec<(MultiplicativeOperator, UnaryExpr<'a>)>,
}

impl<'a> From<UnaryExpr<'a>> for MultiplicativeExpr<'a> {
    fn from(value: UnaryExpr<'a>) -> Self {
        MultiplicativeExpr::from((value, vec![]))
    }
}

impl<'a> From<(UnaryExpr<'a>, Vec<(MultiplicativeOperator, UnaryExpr<'a>)>)>
    for MultiplicativeExpr<'a>
{
    fn from(value: (UnaryExpr<'a>, Vec<(MultiplicativeOperator, UnaryExpr<'a>)>)) -> Self {
        let (operand, operations) = value;
        MultiplicativeExpr {
            operand,
            operations,
        }
    }
}

impl<'a> MultiplicativeExpr<'a> {
    pub fn operand(&self) -> &UnaryExpr<'a> {
        &self.operand
    }

    pub fn operations(&self) -> &[(MultiplicativeOperator, UnaryExpr<'a>)] {
        self.operations.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum MultiplicativeOperator {
    #[default]
    Mul,
    Div,
    Mod,
}

impl From<&str> for MultiplicativeOperator {
    fn from(value: &str) -> Self {
        match value {
            "*" => MultiplicativeOperator::Mul,
            "div" => MultiplicativeOperator::Div,
            "mod" => MultiplicativeOperator::Mod,
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum NameTest<'a> {
    #[default]
    All,
    Namespace(&'a str),
    QName(QName<'a>),
}

impl<'a> From<&'a str> for NameTest<'a> {
    fn from(value: &'a str) -> Self {
        NameTest::Namespace(value)
    }
}

impl<'a> From<QName<'a>> for NameTest<'a> {
    fn from(value: QName<'a>) -> Self {
        NameTest::QName(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum NodeTest<'a> {
    Name(NameTest<'a>),
    Type(NodeType),
    PI(&'a str),
}

impl<'a> Default for NodeTest<'a> {
    fn default() -> Self {
        NodeTest::Name(NameTest::default())
    }
}

impl<'a> From<NameTest<'a>> for NodeTest<'a> {
    fn from(value: NameTest<'a>) -> Self {
        NodeTest::Name(value)
    }
}

impl<'a> From<NodeType> for NodeTest<'a> {
    fn from(value: NodeType) -> Self {
        NodeTest::Type(value)
    }
}

impl<'a> From<&'a str> for NodeTest<'a> {
    fn from(value: &'a str) -> Self {
        NodeTest::PI(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum NodeType {
    #[default]
    Comment,
    Text,
    PI,
    Node,
}

impl From<&str> for NodeType {
    fn from(value: &str) -> Self {
        match value {
            "comment" => NodeType::Comment,
            "text" => NodeType::Text,
            "processing-instruction" => NodeType::PI,
            "node" => NodeType::Node,
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrExpr<'a> {
    operands: Vec<AndExpr<'a>>,
}

impl<'a> From<AndExpr<'a>> for OrExpr<'a> {
    fn from(value: AndExpr<'a>) -> Self {
        OrExpr::from(vec![value])
    }
}

impl<'a> From<Vec<AndExpr<'a>>> for OrExpr<'a> {
    fn from(value: Vec<AndExpr<'a>>) -> Self {
        OrExpr { operands: value }
    }
}

impl<'a> OrExpr<'a> {
    pub fn operands(&self) -> &[AndExpr<'a>] {
        self.operands.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum PathExpr<'a> {
    #[default]
    Root,
    Filter(FilterExpr<'a>),
    Path(
        Option<(Option<FilterExpr<'a>>, LocationPathOperator)>,
        RelativeLocationPath<'a>,
    ),
}

impl<'a> From<FilterExpr<'a>> for PathExpr<'a> {
    fn from(value: FilterExpr<'a>) -> Self {
        PathExpr::Filter(value)
    }
}

impl<'a> From<RelativeLocationPath<'a>> for PathExpr<'a> {
    fn from(value: RelativeLocationPath<'a>) -> Self {
        PathExpr::Path(None, value)
    }
}

impl<'a>
    From<(
        Option<(Option<FilterExpr<'a>>, LocationPathOperator)>,
        RelativeLocationPath<'a>,
    )> for PathExpr<'a>
{
    fn from(
        value: (
            Option<(Option<FilterExpr<'a>>, LocationPathOperator)>,
            RelativeLocationPath<'a>,
        ),
    ) -> Self {
        let (op, path) = value;
        PathExpr::Path(op, path)
    }
}

// -----------------------------------------------------------------------------------------------

pub type PredicateExpr<'a> = Expr<'a>;

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum PrimaryExpr<'a> {
    Variable(QName<'a>),
    Expr(Box<Expr<'a>>),
    Literal(&'a str),
    Number(&'a str),
    Function(FunctionCall<'a>),
}

impl<'a> Default for PrimaryExpr<'a> {
    fn default() -> Self {
        PrimaryExpr::Literal("")
    }
}

impl<'a> From<QName<'a>> for PrimaryExpr<'a> {
    fn from(value: QName<'a>) -> Self {
        PrimaryExpr::Variable(value)
    }
}

impl<'a> From<Expr<'a>> for PrimaryExpr<'a> {
    fn from(value: Expr<'a>) -> Self {
        PrimaryExpr::Expr(Box::new(value))
    }
}

impl<'a> From<&'a str> for PrimaryExpr<'a> {
    fn from(value: &'a str) -> Self {
        PrimaryExpr::Literal(value)
    }
}

impl<'a> From<FunctionCall<'a>> for PrimaryExpr<'a> {
    fn from(value: FunctionCall<'a>) -> Self {
        PrimaryExpr::Function(value)
    }
}

impl<'a> PrimaryExpr<'a> {
    pub fn number(value: &'a str) -> Self {
        PrimaryExpr::Number(value)
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelationalExpr<'a> {
    operand: AdditiveExpr<'a>,
    operations: Vec<(RelationalOperator, AdditiveExpr<'a>)>,
}

impl<'a> From<AdditiveExpr<'a>> for RelationalExpr<'a> {
    fn from(value: AdditiveExpr<'a>) -> Self {
        RelationalExpr::from((value, vec![]))
    }
}

impl<'a>
    From<(
        AdditiveExpr<'a>,
        Vec<(RelationalOperator, AdditiveExpr<'a>)>,
    )> for RelationalExpr<'a>
{
    fn from(
        value: (
            AdditiveExpr<'a>,
            Vec<(RelationalOperator, AdditiveExpr<'a>)>,
        ),
    ) -> Self {
        let (operand, operations) = value;
        RelationalExpr {
            operand,
            operations,
        }
    }
}

impl<'a> RelationalExpr<'a> {
    pub fn operand(&self) -> &AdditiveExpr<'a> {
        &self.operand
    }

    pub fn operations(&self) -> &[(RelationalOperator, AdditiveExpr<'a>)] {
        self.operations.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum RelationalOperator {
    #[default]
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
}

impl From<&str> for RelationalOperator {
    fn from(value: &str) -> Self {
        match value {
            "<" => RelationalOperator::LessThan,
            ">" => RelationalOperator::GreaterThan,
            "<=" => RelationalOperator::LessEqual,
            ">=" => RelationalOperator::GreaterEqual,
            _ => unreachable!(),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Step<'a> {
    Test(AxisSpecifier, NodeTest<'a>, Vec<Expr<'a>>),
    #[default]
    Current,
    Parent,
}

impl<'a> From<(AxisSpecifier, NodeTest<'a>, Vec<Expr<'a>>)> for Step<'a> {
    fn from(value: (AxisSpecifier, NodeTest<'a>, Vec<Expr<'a>>)) -> Self {
        let (axis, node, predicate) = value;
        Step::Test(axis, node, predicate)
    }
}

impl<'a> Step<'a> {
    pub fn parent() -> Self {
        Step::Parent
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnaryExpr<'a> {
    inv: Vec<&'a str>,
    value: UnionExpr<'a>,
}

impl<'a> From<UnionExpr<'a>> for UnaryExpr<'a> {
    fn from(value: UnionExpr<'a>) -> Self {
        UnaryExpr::from((vec![], value))
    }
}

impl<'a> From<(Vec<&'a str>, UnionExpr<'a>)> for UnaryExpr<'a> {
    fn from(value: (Vec<&'a str>, UnionExpr<'a>)) -> Self {
        let (inv, v) = value;
        UnaryExpr { inv, value: v }
    }
}

impl<'a> UnaryExpr<'a> {
    pub fn inv(&self) -> &[&'a str] {
        self.inv.as_slice()
    }

    pub fn value(&self) -> &UnionExpr<'a> {
        &self.value
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnionExpr<'a> {
    operands: Vec<PathExpr<'a>>,
}

impl<'a> From<PathExpr<'a>> for UnionExpr<'a> {
    fn from(value: PathExpr<'a>) -> Self {
        UnionExpr::from(vec![value])
    }
}

impl<'a> From<Vec<PathExpr<'a>>> for UnionExpr<'a> {
    fn from(value: Vec<PathExpr<'a>>) -> Self {
        UnionExpr { operands: value }
    }
}

impl<'a> UnionExpr<'a> {
    pub fn operands(&self) -> &[PathExpr<'a>] {
        self.operands.as_slice()
    }
}

// -----------------------------------------------------------------------------------------------
