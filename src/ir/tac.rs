use std::fmt::Display;

pub type Label = String;

pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

pub struct BasicBlock {
    pub label: Label,
    pub parameters: Vec<Parameter>,
    pub instructions: Vec<Instruction>,
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}{}:",
            self.label,
            if self.parameters.is_empty() {
                "".to_string()
            } else {
                format!(
                    "({})",
                    self.parameters
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.ty))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        )?;
        for instruction in &self.instructions {
            writeln!(f, "  {}", instruction)?;
        }
        Ok(())
    }
}

pub enum Instruction {
    Add {
        dest: Entity,
        lhs: Entity,
        rhs: Entity,
    },
    Sub {
        dest: Entity,
        lhs: Entity,
        rhs: Entity,
    },
    Mul {
        dest: Entity,
        lhs: Entity,
        rhs: Entity,
    },
    Div {
        dest: Entity,
        lhs: Entity,
        rhs: Entity,
    },
    Not {
        dest: Entity,
        src: Entity,
    },
    Assign {
        dest: Entity,
        src: Entity,
    },
    Conditional {
        lhs: Entity,
        rhs: Entity,
        operator: Operator,
        true_label: Label,
        false_label: Label,
    },
    Call {
        dest: Entity,
        callee: Entity,
        args: Vec<Entity>,
    },
    Goto(Label),
    Ret(Entity),
    Exit(Entity),
}

impl Instruction {
    pub fn assign(dest: Entity, src: Entity) -> Self {
        Self::Assign { dest, src }
    }
    pub fn add(dest: Entity, lhs: Entity, rhs: Entity) -> Self {
        Self::Add { dest, lhs, rhs }
    }
    pub fn sub(dest: Entity, lhs: Entity, rhs: Entity) -> Self {
        Self::Sub { dest, lhs, rhs }
    }
    pub fn mul(dest: Entity, lhs: Entity, rhs: Entity) -> Self {
        Self::Mul { dest, lhs, rhs }
    }
    pub fn div(dest: Entity, lhs: Entity, rhs: Entity) -> Self {
        Self::Div { dest, lhs, rhs }
    }
    pub fn not(dest: Entity, src: Entity) -> Self {
        Self::Not { dest, src }
    }
    pub fn call(dest: Entity, callee: Entity, args: Vec<Entity>) -> Self {
        Self::Call { dest, callee, args }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Add { dest, lhs, rhs } => write!(f, "{dest} := {lhs} + {rhs}"),
            Instruction::Sub { dest, lhs, rhs } => write!(f, "{dest} := {lhs} - {rhs}"),
            Instruction::Mul { dest, lhs, rhs } => write!(f, "{dest} := {lhs} * {rhs}"),
            Instruction::Div { dest, lhs, rhs } => write!(f, "{dest} := {lhs} / {rhs}"),
            Instruction::Not { dest, src } => write!(f, "{dest} := !{src}"),
            Instruction::Assign { dest, src } => write!(f, "{dest} := {src}"),
            Instruction::Conditional {
                lhs,
                rhs,
                operator,
                true_label,
                false_label,
            } => write!(
                f,
                "if {lhs} {operator} {rhs} then goto {true_label} else goto {false_label}"
            ),
            Instruction::Call { dest, callee, args } => write!(
                f,
                "{dest} := {callee}({})",
                args.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Instruction::Goto(label) => write!(f, "goto {}", label),
            Instruction::Ret(entity) => write!(f, "ret {entity}"),
            Instruction::Exit(code) => write!(f, "exit {code}"),
        }
    }
}

pub type TempId = usize;

#[derive(Clone, Debug)]
pub struct Entity {
    pub value: Value,
    pub ty: Type,
}

impl Entity {
    pub fn new(value: Value, ty: Type) -> Self {
        Self { value, ty }
    }
    pub fn variable(name: &str, ty: Type) -> Self {
        Self {
            value: Value::Variable(name.to_string()),
            ty,
        }
    }
    pub fn temp_val(id: usize, ty: Type) -> Self {
        Self {
            value: Value::Temp(id),
            ty,
        }
    }
    pub fn int_literal(num: isize) -> Self {
        Self {
            value: Value::Int(num),
            ty: Type::Int,
        }
    }
    pub fn unit() -> Self {
        Self {
            value: Value::Unit,
            ty: Type::Unit,
        }
    }
    pub fn bool(val: bool) -> Self {
        Self {
            value: Value::Bool(val),
            ty: Type::Bool,
        }
    }
    pub fn function(label: &str, return_type: Type) -> Self {
        Self {
            value: Value::FunctionPointer(label.to_string()),
            ty: Type::Function {
                return_type: Box::new(return_type),
            },
        }
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Value::Temp(id) => write!(f, "t{id}"),
            Value::Unit => write!(f, "unit"),
            Value::Int(val) => write!(f, "{val}"),
            Value::Bool(val) => write!(f, "{val}"),
            Value::Variable(name) => write!(f, "{name}"),
            Value::FunctionPointer(label) => write!(f, "{label}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Bool,
    Unit,
    Function { return_type: Box<Type> },
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Unit => write!(f, "Unit"),
            Type::Function { .. } => write!(f, "TODO(function types)"),
        }
    }
}

impl Type {
    pub fn size_in_bytes(&self) -> usize {
        match &self {
            Type::Int => 8,
            Type::Bool => 1,
            Type::Unit => 0,
            Type::Function { .. } => 8,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Temp(TempId),
    Unit,
    Int(isize),
    Bool(bool),
    Variable(String),
    FunctionPointer(Label),
}

pub enum Operator {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Less => write!(f, "<"),
            Operator::Greater => write!(f, ">"),
            Operator::LessEqual => write!(f, "<="),
            Operator::GreaterEqual => write!(f, ">="),
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),
        }
    }
}
