use std::fmt::Display;

use crate::ir::tac::{BasicBlock, Entity, Instruction, TempId, Value};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Register {
    Rax, // return values
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp, // frame pointer
    Rsp, // stack pointer
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Rax => write!(f, "rax"),
            Register::Rbx => write!(f, "rbx"),
            Register::Rcx => write!(f, "rcx"),
            Register::Rdx => write!(f, "rdx"),
            Register::Rsi => write!(f, "rsi"),
            Register::Rdi => write!(f, "rdi"),
            Register::Rbp => write!(f, "rbp"),
            Register::Rsp => write!(f, "rsp"),
            Register::R8 => write!(f, "r8"),
            Register::R9 => write!(f, "r9"),
            Register::R10 => write!(f, "r10"),
            Register::R11 => write!(f, "r11"),
            Register::R12 => write!(f, "r12"),
            Register::R13 => write!(f, "r13"),
            Register::R14 => write!(f, "r14"),
            Register::R15 => write!(f, "r15"),
        }
    }
}

enum RegisterItem {
    TempValue(TempId),
    Variable { name: String, id: usize },
    FunctionPointer(String),
}

pub struct Codegen {
    register_stack: Vec<(Register, Option<RegisterItem>)>,
    register_stack_index: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            register_stack: vec![
                (Register::Rbx, None),
                (Register::Rcx, None),
                (Register::Rdx, None),
                (Register::Rsi, None),
                (Register::Rdi, None),
                (Register::R8, None),
                (Register::R9, None),
                (Register::R10, None),
                (Register::R11, None),
                (Register::R12, None),
                (Register::R13, None),
                (Register::R14, None),
                (Register::R15, None),
            ],
            register_stack_index: usize::MAX,
        }
    }

    fn shift_stack_index_right(&mut self) {
        if self.register_stack_index == usize::MAX {
            self.register_stack_index = 0;
        } else {
            self.register_stack_index += 1;
        }
        if self.register_stack_index > self.register_stack.len() - 1 {
            panic!("not enough temp registers")
        }
    }

    fn shift_stack_index_left(&mut self) {
        if self.register_stack_index == 0 {
            panic!("nuthin' in the stack you silly!!")
        }
        self.register_stack_index -= 1;
    }

    fn get_temp_register(&mut self, item: RegisterItem) -> Register {
        if let Some(register) = self
            .register_stack
            .iter()
            .find(|r| match (&item, &r.1) {
                (RegisterItem::TempValue(item_id), Some(RegisterItem::TempValue(reg_id)))
                    if item_id == reg_id =>
                {
                    true
                }
                (
                    RegisterItem::FunctionPointer(item_label),
                    Some(RegisterItem::FunctionPointer(reg_label)),
                ) if item_label == reg_label => true,
                (
                    RegisterItem::Variable {
                        name: item_name,
                        id: item_id,
                    },
                    Some(RegisterItem::Variable {
                        name: reg_name,
                        id: reg_id,
                    }),
                ) if item_id == reg_id && item_name == reg_name => true,
                _ => false,
            })
            .map(|tup| tup.0)
        {
            register
        } else {
            self.shift_stack_index_right();
            self.register_stack[self.register_stack_index].1 = Some(item);
            self.register_stack[self.register_stack_index].0
        }
    }

    fn entity_to_asm(&mut self, entity: &Entity) -> String {
        match &entity.value {
            Value::Temp(id) => self
                .get_temp_register(RegisterItem::TempValue(*id))
                .to_string(),
            Value::Unit => "todo".to_string(),
            Value::Int(val) => val.to_string(),
            Value::Bool(val) => {
                if *val {
                    "-1".to_string()
                } else {
                    "0".to_string()
                }
            }
            Value::Variable(var) => {
                let items = var.split("#").collect::<Vec<_>>();
                let name = *items.first().expect("uh there's like no name");
                let id = items
                    .get(1)
                    .expect("uh there's like no id")
                    .parse::<usize>()
                    .expect("uh the id wasn't like a string");

                self.get_temp_register(RegisterItem::Variable {
                    name: name.to_string(),
                    id,
                })
                .to_string()
            }
            Value::FunctionPointer(label) => self
                .get_temp_register(RegisterItem::FunctionPointer(label.to_string()))
                .to_string(),
        }
    }

    pub fn gen_asm(&mut self, ir: &[BasicBlock]) -> String {
        let mut lines = vec![];
        lines.push("global _start\n\nsection .text\n".to_string());
        for block in ir {
            lines.push(self.basic_block_to_asm(block));
        }
        lines.join("\n")
    }

    fn basic_block_to_asm(&mut self, block: &BasicBlock) -> String {
        let mut lines = vec![];
        lines.push(format!(
            "{}:",
            match block.label.as_str() {
                "ENTRY" => "_start",
                other => other,
            }
        ));
        for instr in &block.instructions {
            lines.push(self.instruction_to_asm(instr));
        }
        lines.push(String::new());
        lines.join("\n")
    }

    fn instruction_to_asm(&mut self, instr: &Instruction) -> String {
        let mut lines = vec![];
        lines.push(format!("\n  ; {instr}"));
        match instr {
            Instruction::Add { dest, lhs, rhs } => {
                lines.push(format!(
                    "  mov {}, {}",
                    self.entity_to_asm(dest),
                    self.entity_to_asm(lhs)
                ));
                lines.push(format!(
                    "  add {}, {}",
                    self.entity_to_asm(dest),
                    self.entity_to_asm(rhs)
                ));
            }
            Instruction::Sub { dest, lhs, rhs } => {
                lines.push(format!(
                    "  mov {}, {}",
                    self.entity_to_asm(dest),
                    self.entity_to_asm(lhs)
                ));
                lines.push(format!(
                    "  sub {}, {}",
                    self.entity_to_asm(dest),
                    self.entity_to_asm(rhs)
                ));
            }
            Instruction::Mul { dest, lhs, rhs } => {
                lines.push(format!("  mov rax, {}", self.entity_to_asm(rhs)));
                lines.push("  push rdx".to_string());
                lines.push("  push r15".to_string());
                lines.push(format!("  mov r15, {}", self.entity_to_asm(lhs)));
                lines.push("  mul r15".to_string());
                lines.push("  pop r15".to_string());
                lines.push("  pop rdx".to_string());
                lines.push(format!("  mov {}, rax", self.entity_to_asm(dest)));
            }
            Instruction::Div { dest, lhs, rhs } => {
                lines.push(format!("  mov rax, {}", self.entity_to_asm(lhs)));
                lines.push("  push rdx".to_string());
                lines.push("  push r15".to_string());
                lines.push(format!("  mov r15, {}", self.entity_to_asm(rhs)));
                lines.push("  xor rdx, rdx".to_string());
                lines.push("  div r15".to_string());
                lines.push("  pop r15".to_string());
                lines.push("  pop rdx".to_string());
                lines.push(format!("  mov {}, rax", self.entity_to_asm(dest)));
            }
            Instruction::Not { dest, src } => {
                let mut register = self.entity_to_asm(dest);
                lines.push(format!("  mov {}, {}", register, self.entity_to_asm(src)));
                lines.push(format!("  not {}", register));
            }
            Instruction::Assign { dest, src } => {
                match &src.value {
                    Value::FunctionPointer(label) => {
                        lines.push(format!("  lea {}, [{label}]", self.entity_to_asm(dest)))
                    }
                    _ => {
                        lines.push(format!(
                            "  mov {}, {}",
                            self.entity_to_asm(dest),
                            self.entity_to_asm(src)
                        ));
                    }
                };
            }
            Instruction::Conditional { .. } => todo!("conditional"),
            Instruction::Call { dest, callee, .. } => {
                lines.push("  ; TODO: function arguments".to_string());
                lines.push(format!("  call {}", self.entity_to_asm(callee)));
                lines.push(format!("  mov {}, rax", self.entity_to_asm(dest)));
            }
            Instruction::Goto(label) => {
                lines.push(format!("  jmp {label}"));
            }
            Instruction::Ret(entity) => {
                lines.push(format!("  mov rax, {}", self.entity_to_asm(entity)));
                lines.push("  ret".to_string());
            }
            Instruction::Exit(entity) => {
                lines.push("  mov rax, 60".to_string());
                lines.push(format!("  mov rdi, {}", self.entity_to_asm(entity)));
                lines.push("  syscall".to_string());
            }
        };
        lines.join("\n")
    }
}
