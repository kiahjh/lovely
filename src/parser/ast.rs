use crate::span::Span;

#[derive(PartialEq, Eq, Debug)]
pub struct Program(pub Vec<Expression>);

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Expression {
    pub span: Span,
    pub kind: ExpressionKind,
}
impl Expression {
    pub fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { span, kind }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ExpressionKind {
    BoolLiteral(bool),
    IntLiteral(isize),
    Ident(String),

    Block(Vec<Expression>),

    Prefix {
        operator: PrefixOperator,
        expression: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: InfixOperator,
        right: Box<Expression>,
    },

    VariableDecl {
        name: String,
        value: Box<Expression>,
        mutable: bool,
        ty: Option<Type>,
    },

    Function {
        parameters: Vec<FunctionParameter>,
        return_type: Option<Type>,
        body: Box<Expression>,
    },

    FunctionCall {
        name: String,
        arguments: Vec<FunctionArgument>,
    },
}

impl ExpressionKind {
    pub fn is_const(&self) -> bool {
        use crate::parser::ast::ExpressionKind::*;

        match self {
            BoolLiteral(_) | IntLiteral(_) | Ident(_) | Function { .. } => true,
            Block(expressions) => expressions.iter().all(|expr| expr.kind.is_const()),
            Prefix { expression, .. } => expression.kind.is_const(),
            Infix { left, right, .. } => left.kind.is_const() && right.kind.is_const(),
            VariableDecl { value, mutable, .. } => !mutable && value.kind.is_const(),
            FunctionCall { .. } => false,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PrefixOperator {
    LogicalNot,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum InfixOperator {
    Plus,
    Minus,
    Divide,
    Multiply,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FunctionArgument {
    pub label: Option<String>,
    pub value: Expression,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FunctionParameter {
    pub modifier: ParameterModifier,
    pub labeled_at_callsite: bool,
    pub internal_name: String,
    pub external_name: Option<String>,
    pub ty: Type,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ParameterModifier {
    Read,
    Mut,
    Take,
}

#[derive(PartialEq, Eq, Debug)]
pub struct VariableDecl {
    pub name: String,
    pub value: Expression,
    pub mutable: bool,
    pub ty: Option<Type>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Type {
    Ident(String),
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    Lowest,
    Equality,   // == or !=
    Comparison, // <, <=, >, >=
    Sum,        // + or -
    Product,    // * or /
    Group,      // ( )
    Prefix,     // -X or !X
}
