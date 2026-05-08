use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use either::Either;
use lir::{MultiValInsn, ValType, slotmap::SlotMap, typed_index_collections::TiVec};
use smallvec::{SmallVec, smallvec};

use crate::{
    DynType, StaticType,
    mir::{self, analysis},
};

struct ProgramBuilder {
    host_functions: HashMap<&'static mir::HostFunctionInfo, lir::HostFunction>,
    user_function_stubs: BTreeMap<mir::FunctionId, UserFunctionStub>,
}

struct UserFunctionStub {
    lir_id: lir::FunctionId,
    parameter_types: Vec<ValType>,
    return_types: SmallVec<[(usize, ValType); 1]>,
}

struct FnBuilder<'a> {
    program_builder: &'a mut ProgramBuilder,
    /// The MIR function being lowered.
    mir: &'a mir::Function,
    fn_id: lir::FunctionId,
    local_storage: BTreeMap<mir::LocalId, LocalStorage>,
    /// LIR registers that we have created so far.
    lir_registers: TiVec<lir::Reg, lir::RegInfo>,
    /// The LIR instructions that have been built.
    insn_seqs: TiVec<lir::InsnSeqId, Vec<lir::InsnKind>>,
    /// Maps each MIR label to the instruction sequence that it belongs to.
    labels: BTreeMap<mir::Label, lir::InsnSeqId>,
}

enum LocalStorage {
    /// The variable is stored on the stack.
    Stack {
        /// The offset from the stack pointer to the variable.
        offset: usize,
    },
    /// The variable is stored in registers.
    Registers {
        /// Lists a flattened list of (offset, register) pairs where the
        /// register holds the value at the specified flattened offset.
        components: SmallVec<[(usize, lir::Reg); 1]>,
    },
}

impl<'a> FnBuilder<'a> {
    fn new_insn_seq(&mut self, label: Option<mir::Label>) -> InsnSeqBuilder<'_, 'a> {
        let seq_id = self.insn_seqs.push_and_get_key(Vec::new());
        if let Some(label) = label {
            self.labels.insert(label, seq_id);
        }
        InsnSeqBuilder { fn_builder: self, seq_id }
    }
}

struct InsnSeqBuilder<'a, 'b> {
    fn_builder: &'a mut FnBuilder<'b>,
    /// The index of the current instruction sequence being built.
    seq_id: lir::InsnSeqId,
}

impl<'a, 'b> InsnSeqBuilder<'a, 'b> {
    fn assign_reg_with_value(&mut self, out: lir::Reg, insn: lir::SingleValInsn) {
        let ty = insn.output_type(&self.fn_builder.lir_registers);
        assert_eq!(self.fn_builder.lir_registers[out].ty, ty);
        self.fn_builder.insn_seqs[self.seq_id].push(lir::InsnKind::SingleVal { out, insn });
    }

    fn add_reg_with_value(&mut self, name: Option<Arc<str>>, insn: lir::SingleValInsn) -> lir::Reg {
        let ty = insn.output_type(&self.fn_builder.lir_registers);
        let out = self.fn_builder.lir_registers.push_and_get_key(lir::RegInfo { ty, name });
        self.fn_builder.insn_seqs[self.seq_id].push(lir::InsnKind::SingleVal { out, insn });
        out
    }

    fn add_regs_with_value(
        &mut self,
        out_types: &[(usize, ValType)],
        insn: lir::MultiValInsn,
    ) -> SmallVec<[(usize, lir::Reg); 1]> {
        let out_regs: SmallVec<[(usize, lir::Reg); 1]> = out_types
            .iter()
            .map(|&(offset, ty)| {
                let reg =
                    self.fn_builder.lir_registers.push_and_get_key(lir::RegInfo { ty, name: None });
                (offset, reg)
            })
            .collect();
        self.fn_builder.insn_seqs[self.seq_id].push(lir::InsnKind::MultiVal {
            out: out_regs.iter().map(|(_, reg)| *reg).collect(),
            insn,
        });
        out_regs
    }

    fn add_insn(&mut self, insn: lir::InsnKind) {
        self.fn_builder.insn_seqs[self.seq_id].push(insn);
    }
}

impl LocalStorage {
    fn get_reg_for_component_at_offset(&self, offset: usize) -> lir::Reg {
        let Self::Registers { components } = self else {
            panic!("expected local to be stored in registers");
        };
        components
            .iter()
            .find_map(|(o, r)| (*o == offset).then_some(*r))
            .expect("should be able to find component")
    }
}

pub fn translate_program_to_lir(program: &mir::Program) -> lir::Program {
    let mut lir_fn_id_allocator: SlotMap<lir::FunctionId, ()> = SlotMap::with_key();
    let mut program_builder =
        ProgramBuilder { host_functions: HashMap::new(), user_function_stubs: BTreeMap::new() };

    for (&mir_fn_id, mir_fn) in &program.functions {
        let lir_id = lir_fn_id_allocator.insert(());
        let parameter_types = mir_fn
            .parameters
            .iter()
            .flat_map(|param_local| {
                dyn_type_to_lir(&mir_fn.local_decls[param_local].ty)
                    .expect("function parameter type should be representable in LIR")
                    .into_iter()
                    .map(|(_, ty)| ty)
            })
            .collect();
        let return_types = dyn_type_to_lir(mir_fn.return_ty())
            .expect("function return type should be representable in LIR")
            .into_iter()
            .collect();

        program_builder
            .user_function_stubs
            .insert(mir_fn_id, UserFunctionStub { lir_id: lir_id, parameter_types, return_types });
    }

    let _ = program_builder;



    todo!("translate each MIR function body into LIR using the registered user-function stubs");
}

fn translate_function(function: &mir::Function) {
    let address_taken_locals = analysis::address_taken_locals(function);
    let mir::Function { debug_name, parameters, local_decls, return_local, body } = function;
}

fn translate_statements(
    fn_builder: &mut FnBuilder,
    statements: &[mir::Statement],
    label: Option<mir::Label>,
) -> lir::InsnSeqId {
    // "instruction sequence builder"
    let mut isb = fn_builder.new_insn_seq(label);
    for statement in statements {
        match statement {
            mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::Block(mir::Block {
                label,
                statements,
            })) => {
                let body = translate_statements(isb.fn_builder, statements, Some(*label));
                isb.add_insn(lir::InsnKind::Block(lir::Block { body }));
            }
            mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::IfElse(mir::IfElse {
                condition,
                then,
                r#else,
            })) => {
                // there is a better way to read the single register place than
                // to have to clone the place descriptor, but this works for now
                let condition = read_place_operand_into_new_reg(
                    &mut isb,
                    &mir::PlaceOperand::Direct(condition.clone()),
                );
                assert!(condition.len() == 1, "if-else condition must be a single boolean value");
                let condition = condition[0].1;

                let then_body = translate_statements(isb.fn_builder, to_statement_seq(then), None);
                let else_body =
                    translate_statements(isb.fn_builder, to_statement_seq(r#else), None);

                isb.add_insn(lir::InsnKind::IfElse(lir::IfElse { condition, then_body, else_body }))
            }
            mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::Loop(mir::Loop {
                num_repetitions,
                body,
            })) => {
                todo!()
            }
            mir::Statement::Elementary(mir::ElementaryStatement::Break { target }) => {
                let &target =
                    isb.fn_builder.labels.get(target).expect("break target must be a valid label");
                isb.add_insn(lir::InsnKind::Break { target })
            }
            mir::Statement::Elementary(mir::ElementaryStatement::Assign { dst, op }) => {
                execute_operation(&mut isb, dst, op);
            }
            mir::Statement::Elementary(mir::ElementaryStatement::Drop { src }) => {
                todo!()
            }
        }
    }

    todo!()
}

fn to_statement_seq(statement: &mir::Statement) -> &[mir::Statement] {
    match statement {
        mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::Block(mir::Block {
            statements, ..
        })) => statements,
        other => std::slice::from_ref(other),
    }
}

/// The LIR representation of a destination place.
enum LirPlaceRepr {
    Registers(SmallVec<[(usize, lir::Reg); 1]>),
    Memory {
        base_ptr: BasePtr,
        /// The offset of the entire place from the base pointer.
        total_offset: usize,
        /// The additional offsets of the fields of the place. Used when
        /// loading the value from memory. Not used when only a borrow of the
        /// place is needed.
        additional_field_offsets: SmallVec<[(usize, ValType); 1]>,
    },
}

enum LirSrcValue {
    /// The value we want is already in registers, no additional computation needed.
    Registers(SmallVec<[(usize, lir::Reg); 1]>),
    /// The value we want is not in registers, we need to compute it with the following instructions.
    SingleValInsns(SmallVec<[(usize, lir::SingleValInsn); 1]>),
    /// The value we want is not in registers, we need to compute it with the following instruction.
    MultiValInsn { insn: MultiValInsn, out_types: SmallVec<[(usize, ValType); 1]> },
}

fn write_dst_place(builder: &mut InsnSeqBuilder, dst_place_repr: LirPlaceRepr, src: LirSrcValue) {
    match (dst_place_repr, src) {
        (LirPlaceRepr::Registers(dst_regs), LirSrcValue::Registers(src_regs)) => {
            // insert move instructions to move values in between registers
        }
        (LirPlaceRepr::Memory {  }, LirSrcValue::Registers(src_regs)) => {
            // insert memstore instructions
            for
        }
        (LirPlaceRepr::Registers(dst_regs), LirSrcValue::SingleValInsns(src_insns)) => {
            // insert regular insn with the specified dsts
        }
        (LirPlaceRepr::Memory { .. }, LirSrcValue::SingleValInsns(src_insns)) => {
            // insert regular insns with temp dsts and then move those into memory
        }
        (LirPlaceRepr::Registers(small_vec), LirSrcValue::MultiValInsn { insn, out_types }) => {
            todo!()
        }
        (
            LirPlaceRepr::Memory { base_ptr, total_offset, additional_field_offsets },
            LirSrcValue::MultiValInsn { insn, out_types },
        ) => todo!(),
    }
}

fn write_new_place(
    builder: &mut InsnSeqBuilder,
    src: LirSrcValue,
) -> SmallVec<[(usize, lir::Reg); 1]> {
    match src {
        LirSrcValue::Registers(regs) => regs,
        LirSrcValue::SingleValInsns(insns) => insns
            .into_iter()
            .map(|(offset, insn)| {
                let reg = builder.add_reg_with_value(None, insn);
                (offset, reg)
            })
            .collect(),
        LirSrcValue::MultiValInsn { insn, out_types } => {
            builder.add_regs_with_value(&out_types, insn)
        }
    }
}

fn read_place_operand_into_new_reg(
    builder: &mut InsnSeqBuilder,
    place_operand: &mir::PlaceOperand,
) -> SmallVec<[(usize, lir::Reg); 1]> {
    let src = read_place_operand(builder, place_operand);
    write_new_place(builder, src)
}

fn execute_operation(builder: &mut InsnSeqBuilder, dst: &mir::Place, operation: &mir::Operation) {
    let dst_place_repr = translate_place(builder, dst);

    let src_values = match operation {
        mir::Operation::Operand(place_operand) => {
            let (is_borrow, place) = match place_operand {
                mir::PlaceOperand::Borrow(place) => (true, place),
                mir::PlaceOperand::Direct(place) => (false, place),
            };

            read_from_lir_place(translate_place(builder, place), is_borrow)
        }
        mir::Operation::Const(mir::PodValue { ty, bytes }) => {
            let components =
                dyn_type_to_lir(ty).expect("should be able to load type into LIR registers");

            LirSrcValue::SingleValInsns(
                components
                    .into_iter()
                    .map(|(offset, ty)| {
                        let lir_val = match ty {
                            ValType::I8 => lir::Value::I8(bytes[offset]),
                            ValType::I32 => {
                                lir::Value::I32(*bytemuck::from_bytes(&bytes[offset..offset + 4]))
                            }
                            ValType::I64 => {
                                lir::Value::I64(*bytemuck::from_bytes(&bytes[offset..offset + 8]))
                            }
                            ValType::F64 => {
                                lir::Value::F64(*bytemuck::from_bytes(&bytes[offset..offset + 8]))
                            }
                            ValType::Ptr => {
                                unimplemented!(
                                    "pointer values are not stable enough to store in a const"
                                )
                            }
                            ValType::FnPtr => unimplemented!(
                                "function pointer values are not stable enough to store in a const"
                            ),
                        };

                        let insn = lir::SingleValInsn::Const { val: lir_val };
                        (offset, insn)
                    })
                    .collect(),
            )
        }
        mir::Operation::FunctionPtr { function } => {
            let lir_fn_id = builder.fn_builder.program_builder.user_function_stubs[function].lir_id;
            let insn = lir::SingleValInsn::UserFunctionPtr { function: lir_fn_id };
            LirSrcValue::SingleValInsns(smallvec![(0, insn)])
        },
        mir::Operation::BinaryOp { lhs, rhs, opcode } => {
            // materialize the operands into registers
            let lhs = read_place_operand_into_new_reg(builder, lhs);
            assert!(lhs.len() == 1, "binary operation lhs must be a single value");
            let rhs = read_place_operand_into_new_reg(builder, rhs);
            assert!(rhs.len() == 1, "binary operation rhs must be a single value");

            let insn = lir::SingleValInsn::BinaryOp { op: *opcode, lhs: lhs[0].1, rhs: rhs[0].1 };
            LirSrcValue::SingleValInsns(smallvec![(0, insn)])
        }
        mir::Operation::UnaryOp { operand, opcode } => {
            let operand = read_place_operand_into_new_reg(builder, operand);
            assert!(operand.len() == 1, "unary operation operand must be a single value");

            let insn = lir::SingleValInsn::UnaryOp { op: *opcode, operand: operand[0].1 };
            LirSrcValue::SingleValInsns(smallvec![(0, insn)])
        }
        mir::Operation::CallUserFunction { function, args } => {
            let lir_args: Box<[lir::Reg]> = args
                .iter()
                .flat_map(|arg| {
                    read_place_operand_into_new_reg(builder, arg).into_iter().map(|(_, reg)| reg)
                })
                .collect();

            let lir_fn_stub = &builder.fn_builder.program_builder.user_function_stubs[function];
            let lir_fn_id = lir_fn_stub.lir_id;
            let return_tys = lir_fn_stub.return_types.clone();

            let insn = lir::MultiValInsn::CallUserFunction { function: lir_fn_id, args: lir_args };
            LirSrcValue::MultiValInsn { insn, out_types: return_tys }
        }
        mir::Operation::CallHostFunction { function, args } => {
            let lir_args: Box<[lir::Reg]> = args
                .iter()
                .flat_map(|arg| {
                    read_place_operand_into_new_reg(builder, arg).into_iter().map(|(_, reg)| reg)
                })
                .collect();

            let lir_fn = builder
                .fn_builder
                .program_builder
                .host_functions
                .entry(*function)
                .or_insert_with_key(|function| translate_host_function_info(function))
                .clone();
            let return_tys = static_type_to_lir(&function.return_type);

            let insn = lir::MultiValInsn::CallHostFunction { function: lir_fn, args: lir_args };
            LirSrcValue::MultiValInsn { insn, out_types: return_tys }
        }
    };
    write_dst_place(builder, dst_place_repr, src_values);
}

fn translate_place(builder: &mut InsnSeqBuilder, place: &mir::Place) -> LirPlaceRepr {
    let mir::Place { local, projections } = place;
    let local_ty = &builder.fn_builder.mir.local_decls[local].ty;
    match &builder.fn_builder.local_storage[local] {
        LocalStorage::Registers { .. } => {
            translate_place_in_regs(builder, *local, 0, local_ty, projections)
        }
        LocalStorage::Stack { offset } => {
            translate_place_in_mem(builder, BasePtr::StackPtr, *offset, local_ty, projections)
        }
    }
}

fn read_place_operand(
    builder: &mut InsnSeqBuilder,
    place_operand: &mir::PlaceOperand,
) -> LirSrcValue {
    let (is_borrow, place) = match place_operand {
        mir::PlaceOperand::Borrow(place) => (true, place),
        mir::PlaceOperand::Direct(place) => (false, place),
    };

    let lir_place_repr = translate_place(builder, place);
    read_from_lir_place(lir_place_repr, is_borrow)
}

/// Given a place representation, returns the registers that contain the values
/// of that place. If the place is already in registers, this will emit no
/// operations and simply return those register ids. If the place is is memory,
/// this will emit load operations to get the current values.
fn read_from_lir_place(place_repr: LirPlaceRepr, is_borrow: bool) -> LirSrcValue {
    match (is_borrow, place_repr) {
        (false, LirPlaceRepr::Registers(registers)) => LirSrcValue::Registers(registers),
        (true, LirPlaceRepr::Memory { base_ptr, total_offset, additional_field_offsets: _ }) => {
            match base_ptr {
                BasePtr::Reg(base_ptr) => {
                    if total_offset == 0 {
                        // just return the pointer itself
                        LirSrcValue::Registers(smallvec![(0, base_ptr)])
                    } else {
                        // return the pointer with some offset
                        LirSrcValue::SingleValInsns(smallvec![(
                            0,
                            lir::SingleValInsn::DeriveField { offset: total_offset, ptr: base_ptr }
                        )])
                    }
                }
                BasePtr::StackPtr => LirSrcValue::SingleValInsns(smallvec![(
                    0,
                    lir::SingleValInsn::StackAddr { offset: total_offset }
                )]),
            }
        }
        (false, LirPlaceRepr::Memory { base_ptr, total_offset, additional_field_offsets }) => {
            // dereference the pointer for each primitive in the type
            LirSrcValue::SingleValInsns(
                additional_field_offsets
                    .into_iter()
                    .map(|(inner_offset, val_type)| {
                        let insn = insn_to_load(base_ptr, val_type, total_offset + inner_offset);
                        (inner_offset, insn)
                    })
                    .collect(),
            )
        }
        (true, LirPlaceRepr::Registers(_)) => {
            panic!("cannot take the address of a local variable stored in registers");
        }
    }
}

fn translate_place_in_regs(
    builder: &mut InsnSeqBuilder,
    base_local: mir::LocalId,
    current_offset: usize,
    current_type: &DynType,
    projections: &[mir::Projection],
) -> LirPlaceRepr {
    if let Some((first_proj, remaining_proj)) = projections.split_first() {
        let projected_type =
            current_type.project(*first_proj).expect("should be able to project type");
        match first_proj {
            mir::Projection::Field { byte_offset: field_offset } => translate_place_in_regs(
                builder,
                base_local,
                current_offset + field_offset,
                projected_type,
                remaining_proj,
            ),
            mir::Projection::StaticIndex(_) => {
                unimplemented!("did not expect that we would have an array stored in registers")
            }
            mir::Projection::DynamicIndex(_) => {
                unimplemented!("did not expect that we would have an array stored in registers")
            }
            mir::Projection::Deref => {
                // load thes value from the current place and use that as the
                // base pointer for future projections
                let base_ptr = builder.fn_builder.local_storage[&base_local]
                    .get_reg_for_component_at_offset(current_offset);
                translate_place_in_mem(
                    builder,
                    BasePtr::Reg(base_ptr),
                    0,
                    projected_type,
                    remaining_proj,
                )
            }
        }
    } else {
        let components =
            dyn_type_to_lir(current_type).expect("should be able to load type into LIR registers");
        let registers = components
            .into_iter()
            .map(|(inner_offset, _val_type)| {
                let reg = builder.fn_builder.local_storage[&base_local]
                    .get_reg_for_component_at_offset(current_offset + inner_offset);
                (inner_offset, reg)
            })
            .collect();
        LirPlaceRepr::Registers(registers)
    }
}

#[derive(Debug, Clone, Copy)]
enum BasePtr {
    Reg(lir::Reg),
    StackPtr,
}

fn insn_to_load(base_ptr: BasePtr, r#type: ValType, offset: usize) -> lir::SingleValInsn {
    match base_ptr {
        BasePtr::Reg(ptr) => lir::SingleValInsn::MemLoad { r#type, offset, ptr },
        BasePtr::StackPtr => lir::SingleValInsn::StackLoad { r#type, offset },
    }
}

/// Given a pointer to a place in memory and a list of additional projections
/// to apply to the place, produces LIR instructions that apply the projections.
fn translate_place_in_mem(
    builder: &mut InsnSeqBuilder,
    base_ptr: BasePtr,
    current_offset: usize,
    current_type: &DynType,
    projections: &[mir::Projection],
) -> LirPlaceRepr {
    if let Some((first_proj, remaining_proj)) = projections.split_first() {
        let projected_type =
            current_type.project(*first_proj).expect("should be able to project type");
        match first_proj {
            mir::Projection::Field { byte_offset: field_offset } => translate_place_in_mem(
                builder,
                base_ptr,
                current_offset + field_offset,
                projected_type,
                remaining_proj,
            ),
            mir::Projection::StaticIndex(index) => translate_place_in_mem(
                builder,
                base_ptr,
                current_offset + index * projected_type.layout().size(),
                projected_type,
                remaining_proj,
            ),
            mir::Projection::Deref => {
                // to add one level of dereferencing, we have to load from the
                // place we're currently pointing to and use that as the base
                // pointer for future projections
                let new_base_ptr = builder
                    .add_reg_with_value(None, insn_to_load(base_ptr, ValType::Ptr, current_offset));
                translate_place_in_mem(
                    builder,
                    BasePtr::Reg(new_base_ptr),
                    0,
                    projected_type,
                    remaining_proj,
                )
            }
            mir::Projection::DynamicIndex(index) => {
                let index_val =
                    builder.fn_builder.local_storage[index].get_reg_for_component_at_offset(0);
                let BasePtr::Reg(base_ptr) = base_ptr else {
                    unimplemented!(
                        "did not expect that we would have a stack pointer as the base pointer of any array"
                    );
                };
                let new_base_ptr = builder.add_reg_with_value(
                    None,
                    lir::SingleValInsn::DeriveElement {
                        element_size: projected_type.layout().size(),
                        ptr: base_ptr,
                        index: index_val,
                    },
                );

                translate_place_in_mem(
                    builder,
                    BasePtr::Reg(new_base_ptr),
                    0,
                    projected_type,
                    remaining_proj,
                )
            }
        }
    } else {
        let additional_field_offsets =
            dyn_type_to_lir(current_type).expect("should be able to load type into LIR registers");
        LirPlaceRepr::Memory { base_ptr, total_offset: current_offset, additional_field_offsets }
    }
}

/// Decomposes a MIR type into its primitive components.
fn dyn_type_to_lir(ty: &DynType) -> Option<SmallVec<[(usize, ValType); 1]>> {
    match ty {
        DynType::Ref(_) => {
            // just a pointer
            Some(smallvec![(0, ValType::Ptr)])
        }
        DynType::StaticStruct(struct_def) => {
            if let Some(prim) = struct_def.static_ty.primitive_type {
                Some(smallvec![(0, prim)])
            } else if struct_def.exhaustive_fields {
                // drill down into the struct to get the LIR types
                let mut total = smallvec![];
                for (field_offset, field_ty) in &struct_def.fields {
                    let field_results = dyn_type_to_lir(field_ty);
                    total.extend(
                        field_results?.into_iter().map(|(offset, ty)| (*field_offset + offset, ty)),
                    )
                }
                Some(total)
            } else {
                None
            }
        }
        DynType::Array(_) => {
            // we could decompose, but it's better to just leave it in memory
            None
        }
        DynType::Struct(_) => {
            // we could decompose, but it's better to just leave the struct in memory
            None
        }
        DynType::None => None,
    }
}

fn static_type_to_lir(static_ty: &StaticType) -> SmallVec<[(usize, ValType); 1]> {
    dyn_type_to_lir(static_ty.dyn_type).expect("should be able to load type into LIR registers")
}

fn translate_host_function_info(info: &mir::HostFunctionInfo) -> lir::HostFunction {
    let mir::HostFunctionInfo { debug_name, parameter_types, return_type, link_name, link_addr } =
        info;
    let parameter_types = parameter_types
        .iter()
        .flat_map(|ty| static_type_to_lir(ty).into_iter().map(|(_, ty)| ty))
        .collect();
    let return_type = static_type_to_lir(return_type).into_iter().map(|(_, ty)| ty).collect();
    lir::HostFunction::new(lir::HostFunctionInfo {
        parameter_types,
        return_type,
        addr: *link_addr,
        name: *link_name,
    })
}
