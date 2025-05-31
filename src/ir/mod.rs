use crate::{
    checker::{
        CheckedExpression, CheckedExpressionData, CheckedProgram,
        types::{ScopedType, TypeKind},
    },
    parser::ast::{InfixOperator, PrefixOperator},
};
use tac::{BasicBlock, Entity, Instruction, Label, Type, Value};

pub mod tac;

type BlockId = usize;

pub struct IRGenerator {
    blocks: Vec<BasicBlock>,
    current_block: BlockId,
    temp_val_id: usize,
    variables: Vec<Variable>,
    types: Vec<Type>,
    main_label: Option<Label>,
    is_main: bool,
}

struct Variable {
    ty: Type,
    name: String,
}

impl IRGenerator {
    pub fn new(scoped_types: Vec<ScopedType>) -> Self {
        Self {
            temp_val_id: 0,
            blocks: vec![BasicBlock {
                label: "ENTRY".to_string(),
                instructions: vec![],
            }],
            current_block: 0,
            variables: vec![],
            types: type_kinds_to_types(scoped_types.into_iter().map(|t| t.kind).collect()),
            main_label: None,
            is_main: false,
        }
    }

    fn add_var(&mut self, name: &str, id: usize, ty: Type) {
        self.variables.push(Variable {
            ty,
            name: format!("{name}#{id}"),
        })
    }

    fn lookup_var(&self, name: &str, id: usize) -> Option<&Variable> {
        self.variables
            .iter()
            .find(|v| v.name == format!("{name}#{id}"))
    }

    fn get_id(&mut self) -> usize {
        let id = self.temp_val_id;
        self.temp_val_id += 1;
        id
    }

    fn add_block(&mut self, name: &str) -> BlockId {
        let new_block = BasicBlock {
            label: name.to_string(),
            instructions: vec![],
        };
        self.blocks.push(new_block);
        self.blocks.len() - 1
    }

    fn add_instruction(&mut self, instr: Instruction) {
        self.blocks[self.current_block].instructions.push(instr);
    }

    pub fn program_ir(mut self, checked_program: &CheckedProgram) -> Vec<BasicBlock> {
        for expr in &checked_program.exprs {
            self.expression_ir(expr);
        }
        if let Some(label) = &self.main_label {
            self.add_instruction(Instruction::Goto(label.to_string()));
        }
        self.blocks
    }

    fn expression_ir(&mut self, expr: &CheckedExpression) -> Entity {
        match &expr.data {
            CheckedExpressionData::BoolLiteral(val) => Entity::bool(*val),
            CheckedExpressionData::IntLiteral(val) => Entity::int_literal(*val),
            CheckedExpressionData::Ident { name, variable_id } => {
                let Some(var) = self.lookup_var(name, *variable_id) else {
                    unreachable!("unidentified variable: {name}#{variable_id}")
                };
                Entity::variable(&var.name, var.ty.clone())
            }
            CheckedExpressionData::Block(exprs) => {
                let mut ret_value = Entity {
                    value: Value::Unit,
                    ty: Type::Unit,
                };
                for expr in exprs {
                    ret_value = self.expression_ir(expr);
                }
                ret_value
            }
            CheckedExpressionData::Prefix {
                operator,
                expression,
            } => {
                let entity = self.expression_ir(expression);
                match operator {
                    PrefixOperator::LogicalNot => {
                        let temp = self.get_id();
                        self.add_instruction(Instruction::not(
                            Entity::temp_val(temp, Type::Bool),
                            entity,
                        ));
                        Entity::temp_val(temp, Type::Bool)
                    }
                }
            }
            CheckedExpressionData::Infix {
                left,
                operator,
                right,
            } => {
                let lhs = self.expression_ir(left);
                let rhs = self.expression_ir(right);
                match operator {
                    InfixOperator::Plus => {
                        let t1 = self.get_id();
                        self.add_instruction(Instruction::add(
                            Entity::temp_val(t1, Type::Int),
                            lhs,
                            rhs,
                        ));
                        Entity::temp_val(t1, Type::Int)
                    }
                    InfixOperator::Minus => {
                        let t1 = self.get_id();
                        self.add_instruction(Instruction::sub(
                            Entity::temp_val(t1, Type::Int),
                            lhs,
                            rhs,
                        ));
                        Entity::temp_val(t1, Type::Int)
                    }
                    InfixOperator::Multiply => {
                        let t1 = self.get_id();
                        self.add_instruction(Instruction::mul(
                            Entity::temp_val(t1, Type::Int),
                            lhs,
                            rhs,
                        ));
                        Entity::temp_val(t1, Type::Int)
                    }
                    InfixOperator::Divide => {
                        let t1 = self.get_id();
                        self.add_instruction(Instruction::div(
                            Entity::temp_val(t1, Type::Int),
                            lhs,
                            rhs,
                        ));
                        Entity::temp_val(t1, Type::Int)
                    }
                    _ => todo!(),
                }
            }
            CheckedExpressionData::VariableDecl {
                name,
                value,
                variable_id,
                ..
            } => {
                let this_is_main = if let CheckedExpressionData::Function { .. } = value.data {
                    name == "main" && self.current_block == 0
                } else {
                    false
                };

                if this_is_main {
                    self.is_main = true;
                }

                let val = self.expression_ir(value);

                if this_is_main {
                    self.main_label = Some(val.to_string());
                    self.is_main = false;
                }

                let var_name = format!("{name}#{variable_id}");

                self.add_instruction(Instruction::assign(
                    Entity::variable(&var_name, val.ty.clone()),
                    val.clone(),
                ));
                self.add_var(name, *variable_id, val.ty);
                Entity::unit()
            }
            CheckedExpressionData::Function {
                body, parameters, ..
            } => {
                for param in parameters {
                    self.add_var(
                        &param.internal_name,
                        param.variable_id,
                        self.types[param.type_id].clone(),
                    );
                }

                let id = self.get_id();
                let label = format!("FUN#{}", id);
                let prev_block = self.current_block;
                self.current_block = self.add_block(&label);

                let entity = self.expression_ir(body);

                if self.is_main {
                    match &entity.ty {
                        Type::Unit => self.add_instruction(Instruction::Exit(entity.clone())),
                        Type::Int => self.add_instruction(Instruction::Exit(entity.clone())),
                        _ => panic!("main must return Unit or Int"),
                    }
                } else {
                    self.add_instruction(Instruction::Ret(entity.clone()));
                }

                self.current_block = prev_block;

                Entity::function(&label, entity.ty)
            }
            CheckedExpressionData::FunctionCall {
                name,
                arguments,
                variable_id,
            } => {
                let args = arguments
                    .iter()
                    .map(|a| self.expression_ir(&a.value))
                    .collect::<Vec<_>>();

                let id = self.get_id();

                let Some(Variable {
                    ty: Type::Function { return_type },
                    name: name_with_id,
                }) = self.lookup_var(name, *variable_id)
                else {
                    unreachable!(
                        "tried to call a variable that either isn't a function, or doesn't exist: {name}()"
                    );
                };

                let return_type = return_type.clone();
                self.add_instruction(Instruction::call(
                    Entity::temp_val(id, *return_type.clone()),
                    Entity::variable(name_with_id, *return_type.clone()),
                    args,
                ));
                Entity::temp_val(id, *return_type)
            }
        }
    }
}

fn type_kinds_to_types(type_kinds: Vec<TypeKind>) -> Vec<Type> {
    let mut types = vec![];

    for kind in &type_kinds {
        match kind {
            TypeKind::Name(s) if s == "Unit" => types.push(Type::Unit),
            TypeKind::Name(s) if s == "Bool" => types.push(Type::Bool),
            TypeKind::Name(s) if s == "Int" => types.push(Type::Int),
            TypeKind::Name(_) => todo!(),
            TypeKind::Function { return_type, .. } => {
                let Some(type_kind) = type_kinds.get(*return_type) else {
                    unreachable!("pretty sure this should never happen...")
                };
                types.append(&mut type_kinds_to_types(vec![type_kind.clone()]))
            }
        };
    }

    types
}
