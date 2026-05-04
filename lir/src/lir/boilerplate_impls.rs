use std::fmt::{self, Debug, Write};

use super::*;

use pretty_print::PadAdapter;

impl Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Program { user_functions } = self;

        f.debug_struct("Lir")
            .field(
                "user_functions",
                &fmt::from_fn(|f| {
                    f.debug_map()
                        .entries(user_functions.iter().map(|(fn_id, function)| {
                            let print_key =
                                fmt::from_fn(move |f| fn_id.fmt_with_ctx(f, Some(user_functions)));
                            let print_value =
                                fmt::from_fn(|f| function.fmt_with_ctx(f, Some(self)));
                            (print_key, print_value)
                        }))
                        .finish()
                }),
            )
            .finish()
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None)
    }
}

impl Function {
    fn fmt_with_ctx(&self, f: &mut fmt::Formatter<'_>, program: Option<&Program>) -> fmt::Result {
        let Function {
            registers,
            parameters,
            return_values,
            stack_space,
            body,
            insn_seqs,
            debug_fn_name,
            is_entrypoint,
        } = self;

        f.debug_struct("Function")
            .field("debug_name", &debug_fn_name)
            .field("is_entrypoint", &is_entrypoint)
            .field("stack_space", &stack_space)
            .field(
                "return_values",
                &fmt::from_fn(|f| {
                    f.debug_list()
                        .entries(return_values.iter().map(|reg| {
                            fmt::from_fn(move |f| reg.fmt_with_ctx(f, Some(registers), true))
                        }))
                        .finish()
                }),
            )
            .field(
                "parameters",
                &fmt::from_fn(|f| {
                    f.debug_list()
                        .entries(parameters.iter().map(|param| {
                            fmt::from_fn(move |f| param.fmt_with_ctx(f, Some(registers), true))
                        }))
                        .finish()
                }),
            )
            .field(
                "body",
                &fmt::from_fn(|f| {
                    body.fmt_with_ctx(
                        f,
                        Some(registers),
                        Some(insn_seqs),
                        program.map(|p| &p.user_functions),
                    )
                }),
            )
            .finish()
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None, None, None)
    }
}

impl Block {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        registers: Option<&TiVec<Reg, RegInfo>>,
        insn_seqs: Option<&TiVec<InsnSeqId, Vec<InsnKind>>>,
        other_functions: Option<&SecondaryMap<FunctionId, Function>>,
    ) -> fmt::Result {
        let Block { body } = self;

        let Some(insn_seqs) = insn_seqs else {
            return write!(f, "<block {:?}>", body);
        };

        if f.alternate() {
            write!(f, "{:#?} ", body)?;
            fmt_insn_seq(f, &insn_seqs[*body], registers, Some(insn_seqs), other_functions)?;
        } else {
            write!(f, "<block {:?}>", body)?;
        }

        Ok(())
    }
}

impl Debug for IfElse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None, None, None)
    }
}

impl IfElse {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        registers: Option<&TiVec<Reg, RegInfo>>,
        insn_seqs: Option<&TiVec<InsnSeqId, Vec<InsnKind>>>,
        other_functions: Option<&SecondaryMap<FunctionId, Function>>,
    ) -> fmt::Result {
        let IfElse { condition, then_body, else_body } = self;
        let Some(insn_seqs) = insn_seqs else {
            return write!(f, "<ifelse {:?}>", condition);
        };

        write!(f, "if {}?", &fmt::from_fn(|f| condition.fmt_with_ctx(f, registers, false)),)?;
        fmt_insn_seq(f, &insn_seqs[*then_body], registers, Some(insn_seqs), other_functions)?;
        write!(f, " else ")?;
        fmt_insn_seq(f, &insn_seqs[*else_body], registers, Some(insn_seqs), other_functions)?;
        Ok(())
    }
}

impl Debug for Loop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None, None, None)
    }
}

impl Loop {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        registers: Option<&TiVec<Reg, RegInfo>>,
        insn_seqs: Option<&TiVec<InsnSeqId, Vec<InsnKind>>>,
        other_functions: Option<&SecondaryMap<FunctionId, Function>>,
    ) -> fmt::Result {
        let Loop { body } = self;
        let Some(insn_seqs) = insn_seqs else {
            return write!(f, "<loop {:?}>", body);
        };

        write!(f, "loop ")?;
        fmt_insn_seq(f, &insn_seqs[*body], registers, Some(insn_seqs), other_functions)?;

        Ok(())
    }
}

fn fmt_insn_seq(
    f: &mut fmt::Formatter<'_>,
    insn_seq: &[InsnKind],
    registers: Option<&TiVec<Reg, RegInfo>>,
    other_insn_seqs: Option<&TiVec<InsnSeqId, Vec<InsnKind>>>,
    other_functions: Option<&SecondaryMap<FunctionId, Function>>,
) -> fmt::Result {
    if f.alternate() {
        write!(f, "{{\n")?;
        let mut writer = PadAdapter::new(&mut *f);
        for insn in insn_seq {
            write!(
                writer,
                "{};\n",
                &fmt::from_fn(|f| insn.fmt_with_ctx(
                    f,
                    registers,
                    other_insn_seqs,
                    other_functions
                ))
            )?;
        }
        write!(f, "}}")
    } else {
        write!(f, "{{ ")?;
        for insn in insn_seq {
            write!(
                f,
                "{}; ",
                &fmt::from_fn(|f| insn.fmt_with_ctx(
                    f,
                    registers,
                    other_insn_seqs,
                    other_functions
                ))
            )?;
        }
        write!(f, "}}")
    }
}

impl Debug for InsnKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None, None, None)
    }
}

impl InsnKind {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        registers: Option<&TiVec<Reg, RegInfo>>,
        insn_seqs: Option<&TiVec<InsnSeqId, Vec<InsnKind>>>,
        other_functions: Option<&SecondaryMap<FunctionId, Function>>,
    ) -> fmt::Result {
        match self {
            InsnKind::SingleVal { out, insn } => {
                write!(
                    f,
                    "{} = {}",
                    &fmt::from_fn(|f| out.fmt_with_ctx(f, registers, false)),
                    &fmt::from_fn(|f| insn.fmt_with_ctx(f, registers, other_functions)),
                )
            }
            InsnKind::MultiVal { out, insn } => {
                write!(
                    f,
                    "[{:?}] = {}",
                    &fmt::from_fn(|f| {
                        f.debug_list()
                            .entries(out.iter().map(|reg| {
                                fmt::from_fn(move |f| reg.fmt_with_ctx(f, registers, false))
                            }))
                            .finish()
                    }),
                    &fmt::from_fn(|f| insn.fmt_with_ctx(f, registers, other_functions)),
                )
            }
            InsnKind::MemStore { r#type, ptr, value, offset } => {
                write!(
                    f,
                    "store({:?})({} + {}, {})",
                    r#type,
                    &fmt::from_fn(|f| ptr.fmt_with_ctx(f, registers, false)),
                    *offset,
                    &fmt::from_fn(|f| value.fmt_with_ctx(f, registers, false)),
                )
            }
            InsnKind::StackStore { r#type, offset, value } => {
                write!(
                    f,
                    "store({:?})(sp + {}, {})",
                    r#type,
                    *offset,
                    &fmt::from_fn(|f| value.fmt_with_ctx(f, registers, false)),
                )
            }
            InsnKind::Break { target } => {
                write!(f, "break({:?})", target)
            }
            InsnKind::ConditionalBreak { target, condition } => {
                write!(
                    f,
                    "break_if({:?})({:?})",
                    target,
                    &fmt::from_fn(|f| condition.fmt_with_ctx(f, registers, false)),
                )
            }
            InsnKind::Block(block) => block.fmt_with_ctx(f, registers, insn_seqs, other_functions),
            InsnKind::IfElse(if_else) => {
                if_else.fmt_with_ctx(f, registers, insn_seqs, other_functions)
            }
            InsnKind::Loop(r#loop) => r#loop.fmt_with_ctx(f, registers, insn_seqs, other_functions),
        }
    }
}

impl Debug for SingleValInsn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None, None)
    }
}

impl SingleValInsn {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        registers: Option<&TiVec<Reg, RegInfo>>,
        other_functions: Option<&SecondaryMap<FunctionId, Function>>,
    ) -> fmt::Result {
        match self {
            SingleValInsn::Const { val } => write!(f, "{:?}", val),
            SingleValInsn::UserFunctionPtr { function } => {
                function.fmt_with_ctx(f, other_functions)
            }
            SingleValInsn::DeriveField { offset, ptr } => {
                write!(
                    f,
                    "lea({} + {})",
                    &fmt::from_fn(|f| ptr.fmt_with_ctx(f, registers, false)),
                    *offset,
                )
            }
            SingleValInsn::DeriveElement { ptr, element_size, index } => {
                write!(
                    f,
                    "lea({} + {} * {})",
                    &fmt::from_fn(|f| ptr.fmt_with_ctx(f, registers, false)),
                    *element_size,
                    &fmt::from_fn(|f| index.fmt_with_ctx(f, registers, false)),
                )
            }
            SingleValInsn::MemLoad { r#type, ptr, offset } => {
                write!(
                    f,
                    "load({:?})({} + {})",
                    r#type,
                    &fmt::from_fn(|f| ptr.fmt_with_ctx(f, registers, false)),
                    *offset,
                )
            }
            SingleValInsn::StackLoad { r#type, offset } => {
                write!(f, "load({:?})(sp + {})", r#type, *offset,)
            }
            SingleValInsn::StackAddr { offset } => {
                write!(f, "lea(sp + {})", *offset)
            }
            SingleValInsn::UnaryOp { op, operand } => {
                write!(
                    f,
                    "{:?}({})",
                    op,
                    &fmt::from_fn(|f| operand.fmt_with_ctx(f, registers, false))
                )
            }
            SingleValInsn::BinaryOp { op, lhs, rhs } => {
                write!(
                    f,
                    "{:?}({}, {})",
                    op,
                    &fmt::from_fn(|f| lhs.fmt_with_ctx(f, registers, false)),
                    &fmt::from_fn(|f| rhs.fmt_with_ctx(f, registers, false)),
                )
            }
        }
    }
}

impl Debug for MultiValInsn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None, None)
    }
}

impl MultiValInsn {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        registers: Option<&TiVec<Reg, RegInfo>>,
        other_functions: Option<&SecondaryMap<FunctionId, Function>>,
    ) -> fmt::Result {
        match self {
            MultiValInsn::CallHostFunction { function, args } => {
                write!(
                    f,
                    "call({})({:?})",
                    function,
                    &fmt::from_fn(|f| {
                        f.debug_list()
                            .entries(args.iter().map(|reg| {
                                fmt::from_fn(move |f| reg.fmt_with_ctx(f, registers, false))
                            }))
                            .finish()
                    }),
                )
            }
            MultiValInsn::CallUserFunction { function, args } => {
                write!(
                    f,
                    "call({})({:?})",
                    fmt::from_fn(|f| function.fmt_with_ctx(f, other_functions)),
                    &fmt::from_fn(|f| {
                        f.debug_list()
                            .entries(args.iter().map(|reg| {
                                fmt::from_fn(move |f| reg.fmt_with_ctx(f, registers, false))
                            }))
                            .finish()
                    }),
                )
            }
            MultiValInsn::CallIndirectFunction { function, args } => {
                write!(
                    f,
                    "call({:?})({:?})",
                    function,
                    &fmt::from_fn(|f| {
                        f.debug_list()
                            .entries(args.iter().map(|reg| {
                                fmt::from_fn(move |f| reg.fmt_with_ctx(f, registers, false))
                            }))
                            .finish()
                    }),
                )
            }
        }
    }
}

impl FunctionId {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        other_functions: Option<&SecondaryMap<FunctionId, Function>>,
    ) -> fmt::Result {
        if let Some(other_functions) = other_functions
            && let Some(debug_fn_name) = &other_functions[*self].debug_fn_name
        {
            write!(f, "F{:?}#{}", self.0, debug_fn_name)
        } else {
            write!(f, "F{:?}", self.0)
        }
    }
}

impl Debug for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_ctx(f, None, true)
    }
}

impl Reg {
    fn fmt_with_ctx(
        &self,
        f: &mut fmt::Formatter<'_>,
        reg_info: Option<&TiVec<Reg, RegInfo>>,
        declare: bool,
    ) -> fmt::Result {
        let reg_info = reg_info.map(|reg_info| &reg_info[*self]);

        if let Some(reg_info) = reg_info
            && let Some(name) = &reg_info.name
        {
            write!(f, "%{}#{}", self.0, name)?;
        } else {
            write!(f, "%{}", self.0)?;
        }

        if declare && let Some(reg_info) = reg_info {
            write!(f, ": {:?}", reg_info.ty)?;
        }

        Ok(())
    }
}
