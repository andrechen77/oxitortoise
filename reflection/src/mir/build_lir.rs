use std::{collections::BTreeMap, sync::Arc};

use lir::{ValType, typed_index_collections::TiVec};
use smallvec::{SmallVec, smallvec};

use crate::{
    DynType,
    mir::{self, analysis},
};

struct MirToLirBuilder {
    /// Local variables that are stored in registers
    locals_in_registers: BTreeMap<(mir::LocalId, usize), lir::Reg>,
    lir_registers: TiVec<lir::Reg, lir::RegInfo>,
    current_insn_seq_id: lir::InsnSeqId,
    lir_insns: Vec<lir::InsnKind>,
}

impl MirToLirBuilder {
    fn add_reg_with_value(&mut self, name: Option<Arc<str>>, insn: lir::SingleValInsn) -> lir::Reg {
        let ty = insn.output_type(&self.lir_registers);
        let out = self.lir_registers.push_and_get_key(lir::RegInfo { ty, name });
        self.lir_insns.push(lir::InsnKind::SingleVal { out, insn });
        out
    }
}

fn translate_function(function: &mir::Function) {
    let address_taken_locals = analysis::address_taken_locals(function);
    let mir::Function { debug_name, parameters, local_decls, return_local, body } = function;
}

/// Given a local variable stored in registers, produces LIR instructions that
/// produce the given place operand.
fn translate_place_operand(
    place_operand: &mir::PlaceOperand,
    locals: &BTreeMap<(mir::LocalId, usize), lir::Reg>,
) {
    let (is_borrow, place) = match place_operand {
        mir::PlaceOperand::Borrow(place) => (true, place),
        mir::PlaceOperand::Direct(place) => (false, place),
    };
}

fn translate_place_from_regs(
    builder: &mut MirToLirBuilder,
    base_local: mir::LocalId,
    current_offset: usize,
    current_type: &DynType,
    projections: &[mir::Projection],
    is_borrow: bool,
) -> SmallVec<[lir::Reg; 1]> {
    if let Some((first_proj, remaining_proj)) = projections.split_first() {
        let projected_type =
            current_type.project(*first_proj).expect("should be able to project type");
        match first_proj {
            mir::Projection::Field { byte_offset: field_offset } => translate_place_from_regs(
                builder,
                base_local,
                current_offset + field_offset,
                projected_type,
                remaining_proj,
                is_borrow,
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
                let base_ptr = builder.locals_in_registers[&(base_local, current_offset)];
                translate_place_from_addr(
                    builder,
                    base_ptr,
                    0,
                    projected_type,
                    remaining_proj,
                    is_borrow,
                )
            }
        }
    } else {
        if is_borrow {
            panic!("cannot take the address of a local variable stored in registers");
        }
        let components =
            mir_type_to_lir(current_type).expect("should be able to load type into LIR registers");
        components
            .into_iter()
            .map(|(inner_offset, _val_type)| {
                builder.locals_in_registers[&(base_local, current_offset + inner_offset)]
            })
            .collect()
    }
}

/// Given a pointer to a place in memory and a list of additional projections
/// to apply to the place, produces LIR instructions that apply the projections.
fn translate_place_from_addr(
    builder: &mut MirToLirBuilder,
    base_ptr: lir::Reg,
    current_offset: usize,
    current_type: &DynType,
    projections: &[mir::Projection],
    is_borrow: bool,
) -> SmallVec<[lir::Reg; 1]> {
    if let Some((first_proj, remaining_proj)) = projections.split_first() {
        let projected_type =
            current_type.project(*first_proj).expect("should be able to project type");
        match first_proj {
            mir::Projection::Field { byte_offset: field_offset } => translate_place_from_addr(
                builder,
                base_ptr,
                current_offset + field_offset,
                projected_type,
                remaining_proj,
                is_borrow,
            ),
            mir::Projection::StaticIndex(index) => translate_place_from_addr(
                builder,
                base_ptr,
                current_offset + index * projected_type.layout().size(),
                projected_type,
                remaining_proj,
                is_borrow,
            ),
            mir::Projection::Deref => {
                // to add one level of dereferencing, we have to load from the
                // place we're currently pointing to and use that as the base
                // pointer for future projections
                let new_base_ptr = builder.add_reg_with_value(
                    None,
                    lir::SingleValInsn::MemLoad {
                        r#type: ValType::Ptr,
                        offset: current_offset,
                        ptr: base_ptr,
                    },
                );
                translate_place_from_addr(
                    builder,
                    new_base_ptr,
                    0,
                    projected_type,
                    remaining_proj,
                    is_borrow,
                )
            }
            mir::Projection::DynamicIndex(index) => {
                let index_val = builder.locals_in_registers[&(*index, 0)];
                let new_base_ptr = builder.add_reg_with_value(
                    None,
                    lir::SingleValInsn::DeriveElement {
                        element_size: projected_type.layout().size(),
                        ptr: base_ptr,
                        index: index_val,
                    },
                );

                translate_place_from_addr(
                    builder,
                    new_base_ptr,
                    0,
                    projected_type,
                    remaining_proj,
                    is_borrow,
                )
            }
        }
    } else {
        if is_borrow {
            if current_offset == 0 {
                // just return the pointer itself
                smallvec![base_ptr]
            } else {
                // return the pointer with some offset
                let ptr = builder.add_reg_with_value(
                    None,
                    lir::SingleValInsn::DeriveField { offset: current_offset, ptr: base_ptr },
                );
                smallvec![ptr]
            }
        } else {
            let components = mir_type_to_lir(current_type)
                .expect("should be able to load type into LIR registers");

            // dereference the pointer for each primitive in the type
            components
                .into_iter()
                .map(|(inner_offset, val_type)| {
                    builder.add_reg_with_value(
                        None,
                        lir::SingleValInsn::MemLoad {
                            r#type: val_type,
                            offset: current_offset + inner_offset,
                            ptr: base_ptr,
                        },
                    )
                })
                .collect()
        }
    }
}

/// Decomposes a MIR type into its primitive components.
fn mir_type_to_lir(mir_ty: &DynType) -> Option<SmallVec<[(usize, ValType); 1]>> {
    match mir_ty {
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
                    let field_results = mir_type_to_lir(field_ty);
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
