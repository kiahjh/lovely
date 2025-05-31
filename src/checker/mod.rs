#![allow(dead_code)]

use crate::{
    parser::ast::{
        Expression, ExpressionKind,
        InfixOperator::{self, *},
        ParameterModifier,
        PrefixOperator::{self, *},
        Program, Type,
    },
    span::Span,
};
use error::Error;
use types::{FunctionParameterType, FunctionParameterTypeKind, ScopedType, TypeKind};
use vars::{BorrowStatus, ScopedVariable};

mod error;
pub mod types;
mod vars;

type TypeId = usize;
type VariableId = usize;

const INT_ID: usize = 0;
const BOOL_ID: usize = 1;
const UNIT_ID: usize = 2;

pub type ScopeId = usize;

pub struct Scope {
    pub parent_scope: Option<ScopeId>,
}

pub struct Checker {
    cur_scope: ScopeId,
    scopes: Vec<Scope>,
    pub types: Vec<ScopedType>,
    variables: Vec<ScopedVariable>,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            cur_scope: 0,
            scopes: vec![Scope { parent_scope: None }],
            types: vec![
                // builtin types
                ScopedType::named("Int", 0),
                ScopedType::named("Bool", 0),
                ScopedType::named("Unit", 0),
            ],
            variables: vec![],
        }
    }

    fn create_scope(&mut self, parent_id: Option<ScopeId>) -> ScopeId {
        self.scopes.push(Scope {
            parent_scope: parent_id,
        });
        self.scopes.len() - 1
    }

    fn lookup_type(&self, ty: &Type, scope_id: ScopeId) -> Option<TypeId> {
        let cur_scope = &self.scopes[scope_id];
        let Type::Ident(name) = ty;
        if let Some(type_id) = self.types.iter().position(|t| match &t.kind {
            TypeKind::Name(type_name) => name == type_name,
            TypeKind::Function { .. } => false, // TODO: handle function types
        } && t.scope_id == scope_id) {
            Some(type_id)
        } else if let Some(parent_id) = cur_scope.parent_scope {
            self.lookup_type(ty, parent_id)
        } else {
            None
        }
    }

    fn add_type(&mut self, kind: TypeKind) -> TypeId {
        let type_id = self.types.len();
        self.types.push(ScopedType {
            kind,
            scope_id: self.cur_scope,
        });
        type_id
    }

    fn lookup_variable(&self, var_name: &str, scope_id: ScopeId) -> Option<(VariableId, TypeId)> {
        let cur_scope = &self.scopes[scope_id];
        if let Some((index, variable)) = self
            .variables
            .iter()
            .enumerate()
            .rev()
            .find(|t| t.1.scope_id == scope_id && t.1.name == var_name)
        {
            Some((index, variable.type_id))
        } else if let Some(parent_id) = cur_scope.parent_scope {
            self.lookup_variable(var_name, parent_id)
        } else {
            None
        }
    }

    fn lookup_variable_all(
        &self,
        var_name: &str,
        scope_id: ScopeId,
    ) -> Option<Vec<(VariableId, TypeId)>> {
        let cur_scope = &self.scopes[scope_id];
        let mut vec = self
            .variables
            .iter()
            .enumerate()
            .rev()
            .filter(|t| t.1.scope_id == scope_id && t.1.name == var_name)
            .map(|t| (t.0, t.1.type_id))
            .collect::<Vec<_>>();
        if let Some(parent_id) = cur_scope.parent_scope {
            vec.append(
                &mut self
                    .lookup_variable_all(var_name, parent_id)
                    .unwrap_or_default(),
            );
        }
        if vec.is_empty() { None } else { Some(vec) }
    }

    fn add_variable(
        &mut self,
        var_name: &str,
        var_type: TypeId,
        borrow_status: BorrowStatus,
    ) -> VariableId {
        let variable_id = self.variables.len();
        self.variables.push(ScopedVariable::new(
            var_name,
            self.cur_scope,
            var_type,
            borrow_status,
        ));
        variable_id
    }

    pub fn check(&mut self, ast: &Program) -> Result<CheckedProgram, Vec<Error>> {
        let mut checked_exprs = vec![];
        let mut errors = vec![];
        for expr in &ast.0 {
            match self.check_expression(expr, None) {
                Ok(checked_expr) => checked_exprs.push(checked_expr),
                Err(err) => errors.push(err),
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(CheckedProgram {
            exprs: checked_exprs,
        })
    }

    fn check_expression(
        &mut self,
        expr: &Expression,
        type_hint: Option<TypeId>,
    ) -> Result<CheckedExpression, Error> {
        match &expr.kind {
            ExpressionKind::BoolLiteral(value) => self.typed_expression(
                CheckedExpressionData::BoolLiteral(*value),
                expr.span,
                BOOL_ID,
                type_hint,
            ),
            ExpressionKind::IntLiteral(value) => self.typed_expression(
                CheckedExpressionData::IntLiteral(*value),
                expr.span,
                INT_ID,
                type_hint,
            ),
            ExpressionKind::Block(exprs) => {
                // create a new scope
                let new_scope = self.create_scope(Some(self.cur_scope));
                let original_scope = self.cur_scope;
                self.cur_scope = new_scope;

                let mut checked_exprs = vec![];
                let mut type_id = UNIT_ID;
                for (index, expr) in exprs.iter().enumerate() {
                    let checked_expr;
                    if index == exprs.len() - 1 {
                        checked_expr = self.check_expression(expr, type_hint)?;
                        type_id = checked_expr.type_id;
                    } else {
                        checked_expr = self.check_expression(expr, None)?;
                    }
                    checked_exprs.push(checked_expr);
                }

                self.cur_scope = original_scope;

                self.typed_expression(
                    CheckedExpressionData::Block(checked_exprs),
                    expr.span,
                    type_id,
                    type_hint,
                )
            }
            ExpressionKind::Prefix {
                operator,
                expression,
            } => match operator {
                LogicalNot => {
                    let res = self.check_expression(expression, Some(BOOL_ID))?;
                    self.typed_expression(
                        CheckedExpressionData::Prefix {
                            operator: *operator,
                            expression: Box::new(res),
                        },
                        expr.span,
                        BOOL_ID,
                        type_hint,
                    )
                }
            },
            ExpressionKind::Infix {
                left,
                operator,
                right,
            } => match operator {
                // ints
                Plus | Minus | Divide | Multiply => {
                    let left = self.check_expression(left, Some(INT_ID))?;
                    let right = self.check_expression(right, Some(INT_ID))?;
                    self.typed_expression(
                        CheckedExpressionData::Infix {
                            left: Box::new(left),
                            operator: *operator,
                            right: Box::new(right),
                        },
                        expr.span,
                        INT_ID,
                        type_hint,
                    )
                }
                LessThan | GreaterThan | LessThanOrEqual | GreaterThanOrEqual => {
                    let left = self.check_expression(left, Some(INT_ID))?;
                    let right = self.check_expression(right, Some(INT_ID))?;
                    self.typed_expression(
                        CheckedExpressionData::Infix {
                            left: Box::new(left),
                            operator: *operator,
                            right: Box::new(right),
                        },
                        expr.span,
                        BOOL_ID,
                        type_hint,
                    )
                }

                // anything
                Equal | NotEqual => {
                    let left = self.check_expression(left, None)?;
                    let right = self.check_expression(right, None)?;
                    if left.type_id != right.type_id {
                        Err(Error::type_mismatch(left.type_id, right.type_id, expr.span))
                    } else {
                        self.typed_expression(
                            CheckedExpressionData::Infix {
                                left: Box::new(left),
                                operator: *operator,
                                right: Box::new(right),
                            },
                            expr.span,
                            BOOL_ID,
                            type_hint,
                        )
                    }
                }
            },
            ExpressionKind::VariableDecl {
                name,
                value,
                mutable,
                ty,
            } => {
                let r_value = if let Some(ty) = ty {
                    self.check_expression(value, self.lookup_type(ty, self.cur_scope))?
                } else {
                    self.check_expression(value, None)?
                };
                let id = self.add_variable(
                    name,
                    r_value.type_id,
                    if *mutable {
                        BorrowStatus::MutablyOwned
                    } else {
                        BorrowStatus::ImmutableOwned
                    },
                );
                self.typed_expression(
                    CheckedExpressionData::VariableDecl {
                        name: name.to_string(),
                        value: Box::new(r_value),
                        mutable: *mutable,
                        variable_id: id,
                    },
                    expr.span,
                    UNIT_ID,
                    type_hint,
                )
            }
            ExpressionKind::Ident(name) => {
                if let Some((var_id, var_type)) = self.lookup_variable(name, self.cur_scope) {
                    self.typed_expression(
                        CheckedExpressionData::Ident {
                            name: name.to_string(),
                            variable_id: var_id,
                        },
                        expr.span,
                        var_type,
                        type_hint,
                    )
                } else {
                    Err(Error::variable_not_found(name, expr.span))
                }
            }
            ExpressionKind::Function {
                parameters,
                return_type,
                body,
            } => {
                // create a new scope
                let new_scope = self.create_scope(Some(self.cur_scope));
                let original_scope = self.cur_scope;
                self.cur_scope = new_scope;

                let mut checked_params = vec![];

                // add params as local variables in said scope
                for param in parameters {
                    if let Some(type_id) = self.lookup_type(&param.ty, self.cur_scope) {
                        let param_var_id = self.add_variable(
                            &param.internal_name,
                            type_id,
                            match param.modifier {
                                ParameterModifier::Read => BorrowStatus::ImmutablyBorrowed,
                                ParameterModifier::Mut => BorrowStatus::MutablyBorrowed,
                                ParameterModifier::Take => BorrowStatus::ImmutableOwned,
                            },
                        );
                        checked_params.push(CheckedFunctionParameter {
                            modifier: param.modifier,
                            internal_name: param.internal_name.clone(),
                            external_name: param.external_name.clone(),
                            labeled_at_callsite: param.labeled_at_callsite,
                            type_id,
                            variable_id: param_var_id,
                        });
                    } else {
                        return Err(Error::type_not_found(param.ty.clone(), expr.span));
                    }
                }

                // get return type
                let return_type = if let Some(ty) = return_type {
                    ty
                } else {
                    &Type::Ident("Unit".to_string())
                };
                let Some(return_type_id) = self.lookup_type(return_type, self.cur_scope) else {
                    return Err(Error::type_not_found(return_type.clone(), expr.span));
                };

                let body = self.check_expression(body, Some(return_type_id))?;

                let ty = self.add_type(TypeKind::Function {
                    parameters: checked_params
                        .iter()
                        .map(|p| FunctionParameterType {
                            kind: match p.modifier {
                                ParameterModifier::Read => FunctionParameterTypeKind::ReadOnlyRef,
                                ParameterModifier::Mut => FunctionParameterTypeKind::MutableRef,
                                ParameterModifier::Take => FunctionParameterTypeKind::Owned,
                            },
                            callsite_label: p.external_name.clone(),
                            ty: p.type_id,
                        })
                        .collect(),
                    return_type: return_type_id,
                });

                self.cur_scope = original_scope;

                self.typed_expression(
                    CheckedExpressionData::Function {
                        parameters: checked_params,
                        return_type: return_type_id,
                        body: Box::new(body),
                    },
                    expr.span,
                    ty,
                    type_hint,
                )
            }
            ExpressionKind::FunctionCall { name, arguments } => {
                let mut args = vec![];

                for arg in arguments {
                    args.push(CheckedFunctionArgument {
                        label: arg.label.clone(),
                        value: self.check_expression(&arg.value, None)?,
                    });
                }

                let num_args = arguments.len();

                let mut first = true;
                let mut arg_count_errors = vec![];
                let mut type_errors = vec![];
                let mut label_errors = vec![];

                'outer: for (var_id, type_id) in self
                    .lookup_variable_all(name, self.cur_scope)
                    .ok_or(Error::variable_not_found(name, expr.span))?
                {
                    let ScopedType {
                        scope_id: _,
                        kind:
                            TypeKind::Function {
                                parameters,
                                return_type,
                            },
                    } = &self.types[type_id].clone()
                    else {
                        if first {
                            Err(Error::variable_is_not_a_function(name, expr.span))?
                        } else {
                            // Don't assume shadowing
                            continue 'outer;
                        }
                    };

                    first = false;

                    let num_params = parameters.len();
                    if num_params != num_args {
                        arg_count_errors
                            .push(Error::incorrect_arg_count(num_params, num_args, expr.span));
                        continue;
                    }

                    for ((param, arg), old_arg) in parameters.iter().zip(&args).zip(arguments) {
                        if param.ty != arg.value.type_id {
                            type_errors.push(Error::type_mismatch(
                                param.ty,
                                arg.value.type_id,
                                old_arg.value.span,
                            ));
                            continue 'outer;
                        }
                        if param.callsite_label != arg.label {
                            label_errors.push(Error::incorrect_arg_label(
                                param.callsite_label.clone(),
                                arg.label.clone(),
                                expr.span,
                            ));
                            continue 'outer;
                        }
                    }

                    return self.typed_expression(
                        CheckedExpressionData::FunctionCall {
                            name: name.to_string(),
                            arguments: args,
                            variable_id: var_id,
                        },
                        expr.span,
                        *return_type,
                        type_hint,
                    );
                }

                if let Some(error) = label_errors.into_iter().next() {
                    return Err(error);
                }
                if let Some(error) = type_errors.into_iter().next() {
                    return Err(error);
                }
                if let Some(error) = arg_count_errors.into_iter().next() {
                    return Err(error);
                }

                unreachable!("No error, no missing function and no success???")
            }
        }
    }

    fn typed_expression(
        &self,
        res: CheckedExpressionData,
        span: Span,
        res_type: TypeId,
        type_hint: Option<TypeId>,
    ) -> Result<CheckedExpression, Error> {
        if let Some(type_hint) = type_hint {
            if type_hint == res_type {
                Ok(CheckedExpression::new(res, res_type))
            } else {
                Err(Error::type_mismatch(type_hint, res_type, span))
            }
        } else {
            Ok(CheckedExpression::new(res, res_type))
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct CheckedProgram {
    pub exprs: Vec<CheckedExpression>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct CheckedExpression {
    pub type_id: TypeId,
    pub data: CheckedExpressionData,
}
impl CheckedExpression {
    pub fn new(data: CheckedExpressionData, type_id: TypeId) -> Self {
        Self { type_id, data }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum CheckedExpressionData {
    BoolLiteral(bool),
    IntLiteral(isize),
    Ident {
        name: String,
        variable_id: VariableId,
    },

    Block(Vec<CheckedExpression>),

    Prefix {
        operator: PrefixOperator,
        expression: Box<CheckedExpression>,
    },
    Infix {
        left: Box<CheckedExpression>,
        operator: InfixOperator,
        right: Box<CheckedExpression>,
    },

    VariableDecl {
        name: String,
        value: Box<CheckedExpression>,
        mutable: bool,
        variable_id: VariableId,
    },

    Function {
        parameters: Vec<CheckedFunctionParameter>,
        return_type: TypeId,
        body: Box<CheckedExpression>,
    },

    FunctionCall {
        name: String,
        arguments: Vec<CheckedFunctionArgument>,
        variable_id: VariableId,
    },
}

#[derive(PartialEq, Eq, Debug)]
pub struct CheckedFunctionArgument {
    pub label: Option<String>,
    pub value: CheckedExpression,
}

#[derive(PartialEq, Eq, Debug)]
pub struct CheckedFunctionParameter {
    pub modifier: ParameterModifier,
    pub labeled_at_callsite: bool,
    pub internal_name: String,
    pub external_name: Option<String>,
    pub variable_id: VariableId,
    pub type_id: TypeId,
}
