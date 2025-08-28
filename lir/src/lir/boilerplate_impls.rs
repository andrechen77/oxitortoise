use super::*;

impl Debug for InsnSeqId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InsnSeqId({})", self.0)
    }
}

impl Debug for InsnIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for InsnPc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0.0, self.1.0)
    }
}

impl Debug for ValRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{}", self.0, self.1)
    }
}

impl Debug for InsnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InsnKind::FunctionArgs { output_type } => {
                write!(f, "arguments(-> {:?})", output_type)
            }
            InsnKind::LoopArg { initial_value } => {
                write!(f, "loop_args(-> {:?})", initial_value)
            }
            InsnKind::Const(Const { r#type, value }) => {
                write!(f, "constant({:?}, {:?})", r#type, value)
            }
            InsnKind::UserFunctionPtr { function } => {
                write!(f, "user_fn_ptr({:?})", function)
            }
            InsnKind::DeriveField { offset, ptr } => {
                write!(f, "derive_field({:?}, {:?})", offset, ptr)
            }
            InsnKind::DeriveElement { element_size, ptr, index } => {
                write!(f, "derive_element({:?}, {:?}, {:?})", element_size, ptr, index)
            }
            InsnKind::MemLoad { r#type, offset, ptr } => {
                write!(f, "mem_load({:?}, {:?}, {:?})", r#type, offset, ptr)
            }
            InsnKind::MemStore { offset, ptr, value } => {
                write!(f, "mem_store({:?}, {:?}, {:?})", offset, ptr, value)
            }
            InsnKind::StackLoad { r#type, offset } => {
                write!(f, "stack_load({:?}, {:?})", r#type, offset)
            }
            InsnKind::StackStore { offset, value } => {
                write!(f, "stack_store({:?}, {:?})", offset, value)
            }
            InsnKind::StackAddr { offset } => {
                write!(f, "stack_addr({:?})", offset)
            }
            InsnKind::CallHostFunction { function, output_type, args } => {
                write!(f, "call_host_fn({:?}, {:?}, {:?})", function, output_type, args)
            }
            InsnKind::CallUserFunction { function, output_type, args } => {
                write!(f, "call_user_function({:?}, {:?}, {:?})", function, output_type, args)
            }
            InsnKind::UnaryOp { op, operand } => {
                write!(f, "{:?}({:?})", op, operand)
            }
            InsnKind::BinaryOp { op, lhs, rhs } => {
                write!(f, "{:?}({:?}, {:?})", op, lhs, rhs)
            }
            InsnKind::Break { target, values } => {
                write!(f, "break({:?}, {:?})", target, values)
            }
            InsnKind::ConditionalBreak { target, condition, values } => {
                write!(f, "conditional_break({:?}, {:?}, {:?})", target, condition, values)
            }
            InsnKind::Block(block) => {
                write!(f, "{:?}", block)
            }
            InsnKind::IfElse(if_else) => {
                write!(f, "{:?}", if_else)
            }
            InsnKind::Loop(loop_) => {
                write!(f, "{:?}", loop_)
            }
        }
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Block { output_type, body } = self;
        write!(f, "block(-> {:?}) {{{:?}}}", output_type, body)
    }
}

impl Debug for IfElse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let IfElse { output_type, condition, then_body, else_body } = self;
        write!(
            f,
            "if_else(-> {:?})({:?}) {{{:?}}} {{{:?}}}",
            output_type, condition, then_body, else_body
        )
    }
}

impl Debug for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Loop { inputs, output_type, body } = self;
        write!(f, "loop(-> {:?})({:?}) {{{:?}}}", output_type, inputs, body)
    }
}

impl Step for InsnIdx {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        (end.0 - start.0, Some(end.0 - start.0))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(InsnIdx(start.0.checked_add(count)?))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(InsnIdx(start.0.checked_sub(count)?))
    }
}
