use std::collections::HashMap;

use lir::{
    BinaryOpcode, Block, FunctionId, IfElse, InsnIdx, InsnKind, InsnPc, InsnSeqId, Loop, MemOpType,
    Program, UnaryOpcode, ValRef, Value, VarId,
    smallvec::{SmallVec, smallvec},
    typed_index_collections::TiVec,
};
use tracing::warn;

pub struct Interpreter<'p> {
    // The program being interpreted.
    program: &'p Program,
    // A fake function table used to resolve function pointers.
    fn_table: Vec<FunctionId>,
    // Maps each function to its index in the function table, if it has one.
    fn_table_allocated_slots: HashMap<FunctionId, usize>,
}

#[derive(Default)]
pub struct CallFrame {
    local_vars: HashMap<VarId, Value>,
    insn_outputs: HashMap<InsnSeqId, TiVec<InsnIdx, SmallVec<[Value; 1]>>>,
}

impl CallFrame {
    fn get_insn_output(&self, val_ref: ValRef) -> Value {
        let ValRef(InsnPc(seq_id, insn_idx), output_idx) = val_ref;
        self.insn_outputs[&seq_id][insn_idx][usize::from(output_idx)]
    }
}

impl<'p> Interpreter<'p> {
    pub fn new(program: &'p Program) -> Self {
        todo!()
    }

    /// # Safety
    ///
    /// The program being interpreted can execute arbitrary memory operations,
    /// so the caller must ensure that it is safe to execute those operations as
    /// if it was calling some foreign function.
    pub unsafe fn interpret(&self, fn_id: FunctionId, args: &[Value]) -> SmallVec<[Value; 1]> {
        if self.program.entrypoints.iter().find(|&id| *id == fn_id).is_none() {
            warn!("function {:?} is not an entrypoint", fn_id);
        }
        let function = &self.program.user_functions[fn_id];

        let mut frame = CallFrame::default();
        assert_eq!(args.len(), function.num_parameters);
        for (var_id, arg) in args.iter().enumerate().map(|(i, arg)| (VarId(i), *arg)) {
            frame.local_vars.insert(var_id, arg);
        }

        // SAFETY: any unsafe operations specified in the program are the
        // liability of the caller
        let (outputs, break_target) =
            unsafe { self.interpret_insn_seq(&mut frame, fn_id, function.body.body) };
        assert_eq!(break_target, function.body.body);
        outputs
    }

    /// # Safety
    ///
    /// The program being interpreted can execute arbitrary memory operations,
    /// so the caller must ensure that it is safe to execute those operations as
    /// if it was calling some foreign function.
    unsafe fn interpret_insn_seq(
        &self,
        frame: &mut CallFrame,
        fn_id: FunctionId,
        insn_seq_id: InsnSeqId,
    ) -> (SmallVec<[Value; 1]>, InsnSeqId) {
        let function = &self.program.user_functions[fn_id];
        let insn_seq = &function.insn_seqs[insn_seq_id];

        // reset the table of outputs for instructions in this sequence
        frame.insn_outputs.insert(insn_seq_id, TiVec::new());

        for (insn_idx, insn) in insn_seq.iter_enumerated() {
            let output = match insn {
                InsnKind::LoopArg { initial_value } => todo!("TODO(mvp) implement loop arg"),
                InsnKind::Const(val) => {
                    smallvec![*val]
                }
                InsnKind::UserFunctionPtr { function } => {
                    let slot = self.fn_table_allocated_slots[function];
                    let val = Value::FnPtr(slot as *const u8);
                    smallvec![val]
                }
                InsnKind::DeriveField { ptr, offset } => {
                    let Value::Ptr(ptr) = frame.get_insn_output(*ptr) else {
                        panic!("expected a pointer value");
                    };
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    let val = Value::Ptr(unsafe { ptr.add(*offset) });
                    smallvec![val]
                }
                InsnKind::DeriveElement { ptr, index, element_size } => {
                    let Value::Ptr(ptr) = frame.get_insn_output(*ptr) else {
                        panic!("expected a pointer value");
                    };
                    let Value::I32(index) = frame.get_insn_output(*index) else {
                        panic!("expected an index value");
                    };
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    let val = Value::Ptr(unsafe {
                        ptr.add(usize::try_from(index).unwrap() * *element_size)
                    });
                    smallvec![val]
                }
                InsnKind::MemLoad { ptr, offset, r#type } => {
                    let Value::Ptr(ptr) = frame.get_insn_output(*ptr) else {
                        panic!("expected a pointer value");
                    };
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    let val = unsafe {
                        let addr = ptr.add(*offset);
                        match r#type {
                            MemOpType::I8 => Value::I32(u32::from(*addr.cast::<u8>())),
                            MemOpType::I32 => Value::I32(*addr.cast::<u32>()),
                            MemOpType::I64 => Value::I64(*addr.cast::<u64>()),
                            MemOpType::F64 => Value::F64(*addr.cast::<f64>()),
                            MemOpType::Ptr => Value::Ptr(*addr.cast::<*const u8>()),
                            MemOpType::FnPtr => Value::FnPtr(*addr.cast::<*const u8>()),
                        }
                    };
                    smallvec![val]
                }
                InsnKind::MemStore { r#type, ptr, offset, value } => {
                    let Value::Ptr(ptr) = frame.get_insn_output(*ptr) else {
                        panic!("expected a pointer value");
                    };
                    let value = frame.get_insn_output(*value);
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    unsafe {
                        let addr = ptr.add(*offset) as *mut u8;
                        match (r#type, value) {
                            (MemOpType::I8, Value::I32(value)) => *addr.cast::<u8>() = value as u8,
                            (MemOpType::I32, Value::I32(value)) => *addr.cast::<u32>() = value,
                            (MemOpType::I64, Value::I64(value)) => *addr.cast::<u64>() = value,
                            (MemOpType::F64, Value::F64(value)) => *addr.cast::<f64>() = value,
                            (MemOpType::Ptr, Value::Ptr(value)) => {
                                *addr.cast::<*const u8>() = value
                            }
                            (MemOpType::FnPtr, Value::FnPtr(value)) => {
                                *addr.cast::<*const u8>() = value
                            }
                            _ => panic!(
                                "invalid operand type for mem store operation {:?}: {:?}",
                                r#type, value
                            ),
                        }
                    }
                    smallvec![]
                }
                InsnKind::StackLoad { r#type, offset } => todo!("TODO(mvp) implement stack load"),
                InsnKind::StackStore { r#type, offset, value } => {
                    todo!("TODO(mvp) implement stack store")
                }
                InsnKind::VarLoad { var_id } => {
                    let val = frame.local_vars[&var_id];
                    smallvec![val]
                }
                InsnKind::VarStore { var_id, value } => {
                    frame.local_vars.insert(*var_id, frame.get_insn_output(*value));
                    smallvec![]
                }
                InsnKind::StackAddr { offset } => todo!("TODO(mvp) implement stack addr"),
                InsnKind::CallHostFunction { function, output_type, args } => {
                    let args: Vec<Value> =
                        args.iter().map(|arg| frame.get_insn_output(*arg)).collect();
                    todo!("TODO(mvp) call the host function and verify the output types")
                }
                InsnKind::CallUserFunction { function, output_type, args } => {
                    let args: Vec<Value> =
                        args.iter().map(|arg| frame.get_insn_output(*arg)).collect();
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    let return_vals = unsafe { self.interpret(*function, &args) };
                    // TODO verify the output types
                    return_vals
                }
                InsnKind::CallIndirectFunction { function, output_type, args } => {
                    let Value::FnPtr(fn_ptr) = frame.get_insn_output(*function) else {
                        panic!("expected a function pointer value");
                    };
                    let function_id = self.fn_table[fn_ptr as usize];
                    let args: Vec<Value> =
                        args.iter().map(|arg| frame.get_insn_output(*arg)).collect();
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    let return_vals = unsafe { self.interpret(function_id, &args) };
                    // TODO verify the output types
                    return_vals
                }
                InsnKind::UnaryOp { op, operand } => {
                    let operand = frame.get_insn_output(*operand);
                    let val = match (op, operand) {
                        (UnaryOpcode::I64ToI32, Value::I64(value)) => Value::I32(value as u32),
                        (UnaryOpcode::FNeg, Value::F64(value)) => Value::F64(-value),
                        (UnaryOpcode::Not, Value::I32(value)) => Value::I32(!value),
                        _ => panic!(
                            "invalid operand types for unary operation {:?}: {:?}",
                            op, operand
                        ),
                    };
                    smallvec![val]
                }
                InsnKind::BinaryOp { op, lhs, rhs } => {
                    let lhs = frame.get_insn_output(*lhs);
                    let rhs = frame.get_insn_output(*rhs);
                    let val = match (op, lhs, rhs) {
                        // IAdd, ISub, IMul: I32, I64, Ptr
                        (BinaryOpcode::IAdd, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32(lhs + rhs)
                        }
                        (BinaryOpcode::ISub, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32(lhs - rhs)
                        }
                        (BinaryOpcode::IMul, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32(lhs * rhs)
                        }
                        (BinaryOpcode::IAdd, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I64(lhs + rhs)
                        }
                        (BinaryOpcode::ISub, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I64(lhs - rhs)
                        }
                        (BinaryOpcode::IMul, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I64(lhs * rhs)
                        }
                        (BinaryOpcode::IAdd, Value::Ptr(lhs), Value::Ptr(rhs)) => {
                            Value::Ptr(((lhs as usize) + (rhs as usize)) as *const u8)
                        }
                        (BinaryOpcode::ISub, Value::Ptr(lhs), Value::Ptr(rhs)) => {
                            Value::Ptr(((lhs as usize).saturating_sub(rhs as usize)) as *const u8)
                        }
                        (BinaryOpcode::IMul, Value::Ptr(lhs), Value::Ptr(rhs)) => {
                            Value::Ptr(((lhs as usize) * (rhs as usize)) as *const u8)
                        }
                        // ULt, UGt, IEq, INeq: I32, I64, Ptr -> I32
                        (BinaryOpcode::ULt, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32((lhs < rhs) as u32)
                        }
                        (BinaryOpcode::UGt, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32((lhs > rhs) as u32)
                        }
                        (BinaryOpcode::IEq, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32((lhs == rhs) as u32)
                        }
                        (BinaryOpcode::INeq, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32((lhs != rhs) as u32)
                        }
                        (BinaryOpcode::ULt, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I32((lhs < rhs) as u32)
                        }
                        (BinaryOpcode::UGt, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I32((lhs > rhs) as u32)
                        }
                        (BinaryOpcode::IEq, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I32((lhs == rhs) as u32)
                        }
                        (BinaryOpcode::INeq, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I32((lhs != rhs) as u32)
                        }
                        (BinaryOpcode::ULt, Value::Ptr(lhs), Value::Ptr(rhs)) => {
                            Value::I32((lhs < rhs) as u32)
                        }
                        (BinaryOpcode::UGt, Value::Ptr(lhs), Value::Ptr(rhs)) => {
                            Value::I32((lhs > rhs) as u32)
                        }
                        (BinaryOpcode::IEq, Value::Ptr(lhs), Value::Ptr(rhs)) => {
                            Value::I32((lhs == rhs) as u32)
                        }
                        (BinaryOpcode::INeq, Value::Ptr(lhs), Value::Ptr(rhs)) => {
                            Value::I32((lhs != rhs) as u32)
                        }
                        // SLt, SGt: I32, I64 -> I32
                        (BinaryOpcode::SLt, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32(((lhs as i32) < (rhs as i32)) as u32)
                        }
                        (BinaryOpcode::SGt, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32(((lhs as i32) > (rhs as i32)) as u32)
                        }
                        (BinaryOpcode::SLt, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I32(((lhs as i64) < (rhs as i64)) as u32)
                        }
                        (BinaryOpcode::SGt, Value::I64(lhs), Value::I64(rhs)) => {
                            Value::I32(((lhs as i64) > (rhs as i64)) as u32)
                        }
                        // FAdd, FSub, FMul, FDiv: F64 -> F64
                        (BinaryOpcode::FAdd, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::F64(lhs + rhs)
                        }
                        (BinaryOpcode::FSub, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::F64(lhs - rhs)
                        }
                        (BinaryOpcode::FMul, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::F64(lhs * rhs)
                        }
                        (BinaryOpcode::FDiv, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::F64(lhs / rhs)
                        }
                        // FLt, FLte, FGt, FGte, FEq: F64 -> I32
                        (BinaryOpcode::FLt, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::I32((lhs < rhs) as u32)
                        }
                        (BinaryOpcode::FLte, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::I32((lhs <= rhs) as u32)
                        }
                        (BinaryOpcode::FGt, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::I32((lhs > rhs) as u32)
                        }
                        (BinaryOpcode::FGte, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::I32((lhs >= rhs) as u32)
                        }
                        (BinaryOpcode::FEq, Value::F64(lhs), Value::F64(rhs)) => {
                            Value::I32((lhs == rhs) as u32)
                        }
                        // And, Or: I32 -> I32
                        (BinaryOpcode::And, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32(lhs & rhs)
                        }
                        (BinaryOpcode::Or, Value::I32(lhs), Value::I32(rhs)) => {
                            Value::I32(lhs | rhs)
                        }
                        _ => panic!(
                            "Invalid operand types for operation {:?}: {:?} and {:?}",
                            op, lhs, rhs
                        ),
                    };
                    smallvec![val]
                }
                InsnKind::Break { target, values } => {
                    let values = values.iter().map(|value| frame.get_insn_output(*value)).collect();
                    return (values, *target);
                }
                InsnKind::ConditionalBreak { target, condition, values } => {
                    let Value::I32(condition) = frame.get_insn_output(*condition) else {
                        panic!("expected a boolean condition value");
                    };
                    match condition {
                        0 => smallvec![],
                        1 => {
                            let values =
                                values.iter().map(|value| frame.get_insn_output(*value)).collect();
                            return (values, *target);
                        }
                        _ => panic!("expected a boolean condition value, got {:?}", condition),
                    }
                }
                InsnKind::Block(Block { output_type, body }) => {
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    let (outputs, break_target) =
                        unsafe { self.interpret_insn_seq(frame, fn_id, *body) };

                    if break_target != *body {
                        return (outputs, break_target);
                    }
                    // TODO verify output types
                    outputs
                }
                InsnKind::IfElse(IfElse { output_type, condition, then_body, else_body }) => {
                    let Value::I32(condition) = frame.get_insn_output(*condition) else {
                        panic!("expected a boolean condition value");
                    };
                    // SAFETY: any unsafe operations specified in the program
                    // are the liability of the caller
                    let (outputs, break_target) = unsafe {
                        match condition {
                            0 => self.interpret_insn_seq(frame, fn_id, *then_body),
                            1 => self.interpret_insn_seq(frame, fn_id, *else_body),
                            _ => panic!("expected a boolean condition value, got {:?}", condition),
                        }
                    };
                    // TODO verify output types
                    outputs
                }
                InsnKind::Loop(Loop { inputs, output_type, body }) => {
                    todo!("TODO(mvp) implement loop")
                }
            };
            let output_insn_idx =
                frame.insn_outputs.get_mut(&insn_seq_id).unwrap().push_and_get_key(output);
            assert_eq!(output_insn_idx, insn_idx);
        }

        panic!(
            "reached the end of the instruction sequence without encountering a break instruction"
        );
    }
}
