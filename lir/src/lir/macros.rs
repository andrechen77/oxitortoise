pub use crate::lir_function;

#[macro_export]
macro_rules! next_val_ref {
    ($ctx:expr) => {{
        let val_ref = *$ctx.next_val_ref;
        $ctx.next_val_ref.0 += 1;
        val_ref
    }};
}

#[macro_export]
macro_rules! push_node {
    ($ctx:expr; % $val_ref:pat = $name:ident) => {
        let $val_ref = $name;
    };
    ($ctx:expr; % $val_ref:pat = constant($ty:ident, $value:expr)) => {
        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::Const {
            r#type: $crate::lir::ValType::$ty,
            value: $value,
        });
    };
    ($ctx:expr; % $val_ref:pat = argument($ty:ident, $index:expr)) => {
        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::Argument {
            r#type: $crate::lir::ValType::$ty,
            index: $index,
        });
    };
    ($ctx:expr; % $val_ref:pat = loop_argument($initial_value:ident)) => {
        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::LoopArgument {
            initial_value: $initial_value,
        });
    };
    ($ctx:expr; % $val_ref:pat = derive_field($offset:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; %ptr = $ptr_ident $(($($ptr_param)*))*);

        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::DeriveField {
            offset: $offset,
            ptr: $ptr,
        });
    };
    ($ctx:expr; % $val_ref:pat = derive_element($element_size:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))*,
        $index_ident:ident $(($($index_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; %ptr = $ptr_ident $(($($ptr_param)*))*);
        $crate::push_node!($ctx; %index = $index_ident $(($($index_param)*))*);

        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::DeriveElement {
            element_size: $element_size,
            ptr,
            index,
        });
    };
    ($ctx:expr; % $val_ref:pat = mem_load($ty:ident, $offset:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; %ptr = $ptr_ident $(($($ptr_param)*))*);

        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::MemLoad {
            r#type: $crate::lir::ValType::$ty,
            offset: $offset,
            ptr,
        });
    };
    ($ctx:expr; mem_store($offset:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))*,
        $value_ident:ident $(($($value_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; %ptr = $ptr_ident $(($($ptr_param)*))*);
        $crate::push_node!($ctx; %value = $value_ident $(($($value_param)*))*);

        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::MemStore {
            offset: $offset,
            ptr,
            value,
        });
    };
    ($ctx:expr; % $val_ref:pat = stack_load($ty:ident, $offset:expr)) => {
        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::StackLoad {
            r#type: $crate::lir::ValType::$ty,
            offset: $offset,
        });
    };
    ($ctx:expr; stack_store($offset:expr)(
        $value_ident:ident $(($($value_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; %value = $value_ident $(($($value_param)*))*);

        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::StackStore {
            offset: $offset,
            value: $value,
        });
    };
    ($ctx:expr; % $val_ref:pat = call_imported_function($function:ident -> ($($return_ty:ident),*))(
        $(
            $arg_ident:ident $(($($arg_param:tt)*))*
        ),* $(,)?
    )) => {
        let mut args = Vec::new();
        $(
            $crate::push_node!($ctx; %arg = $arg_ident $(($($arg_param)*))*);
            args.push(arg);
        )*

        let $val_ref = ($({
            $crate::lir::ValType::$return_ty;
            $crate::next_val_ref!($ctx)
        }),*);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::CallImportedFunction {
            function: $crate::lir::ImportedFunctionId { name: stringify!($function) },
            args: args.into_boxed_slice(),
        });
    };
    ($ctx:expr; % $val_ref:pat = call_user_function($function:ident -> ($($return_ty:ident),*))(
        $(
            $arg_ident:ident $(($($arg_param:tt)*))*
        ),* $(,)?
    )) => {
        let mut args = Vec::new();
        $(
            $crate::push_node!($ctx; %arg = $arg_ident $(($($arg_param)*))*);
            args.push(arg);
        )*
        let $val_ref = ($({
            $crate::lir::ValType::$return_ty;
            $crate::next_val_ref!($ctx)
        }),*);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::CallUserFunction {
            function: $crate::lir::FunctionId,
            args: args.into_boxed_slice(),
        });
    };
    ($ctx:expr; % $val_ref:pat = $unary_op:ident(
        $operand_ident:ident $(($($operand_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; %operand = $operand_ident $(($($operand_param)*))*);

        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::UnaryOp {
            op: $crate::lir::UnaryOpcode::$unary_op,
            operand,
        });
    };
    ($ctx:expr; % $val_ref:pat = $binary_op:ident(
        $lhs_ident:ident $(($($lhs_param:tt)*))*,
        $rhs_ident:ident $(($($rhs_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; %lhs = $lhs_ident $(($($lhs_param)*))*);
        $crate::push_node!($ctx; %rhs = $rhs_ident $(($($rhs_param)*))*);

        let $val_ref = $crate::next_val_ref!($ctx);
        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::BinaryOp {
            op: $crate::lir::BinaryOpcode::$binary_op,
            lhs,
            rhs,
        });
    };
    ($ctx:expr; break_($target:expr)(
        $($value_ident:ident $(($($value_param:tt)*))*),* $(,)?
    )) => {
        let mut values = Vec::new();
        $(
            $crate::push_node!($ctx; %value = $value_ident $(($($value_param)*))*);
            values.push(value);
        )*

        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::Break {
            target: $target,
            values: values.into_boxed_slice(),
        });
    };
    ($ctx:expr; break_if($target:expr)(
        $condition_ident:ident $(($($condition_param:tt)*))*,
        $($value_ident:ident $(($($value_param:tt)*))*),* $(,)?
    )) => {
        $crate::push_node!($ctx; %condition = $condition_ident $(($($condition_param)*))*);

        let mut values = Vec::new();
        $(
            $crate::push_node!($ctx; %value = $value_ident $(($($value_param)*))*);
            values.push(value);
        )*

        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::ConditionalBreak {
            target: $target,
            condition,
            values: values.into_boxed_slice(),
        });
    };
    ($ctx:expr; % $val_ref:pat = block(-> ($($return_ty:ident),*)) $label:ident: {$($inner:tt)*}) => {
        let $val_ref = ($({
            #[allow(unused)]
            $crate::lir::ValType::$return_ty;
            $crate::next_val_ref!($ctx)
        }),*);

        $crate::instruction_seq!($ctx; $label: { $($inner)* });

        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::Block($crate::lir::Block {
            output_type: $crate::lir::InsnOutput::from_types_array([$($crate::lir::ValType::$return_ty),*]),
            body: $label,
        }));
    };
    ($ctx:expr; % $val_ref:pat = if_else(-> ($($return_ty:ident),*))(
        $condition_ident:ident $(($($condition_param:tt)*))*
    ) $then_label:ident: {$($then_inner:tt)*} $else_label:ident: {$($else_inner:tt)*}) => {
        $crate::push_node!($ctx; %condition = $condition_ident $(($($condition_param)*))*);

        let $val_ref = ($({
            #[allow(unused)]
            $crate::lir::ValType::$return_ty;
            $crate::next_val_ref!($ctx)
        }),*);

        $crate::instruction_seq!($ctx; $then_label: { $($then_inner)* });
        $crate::instruction_seq!($ctx; $else_label: { $($else_inner)* });

        $ctx.insn_seqs[$ctx.curr_seq_id].push($crate::lir::InsnKind::IfElse($crate::lir::IfElse {
            output_type: $crate::lir::InsnOutput::from_types_array([$($crate::lir::ValType::$return_ty),*]),
            condition,
            then_body: $then_label,
            else_body: $else_label,
        }));
    };
}

#[macro_export]
macro_rules! instruction_seq {
    ($ctx:expr; $label:ident: { $($(% $val_ref:pat =)? $node_ident:ident $(($($param:tt)*))* $($body_label:ident: {$($body:tt)*})*; )* }) => {
        let $label = $ctx.insn_seqs.next_key();
        $ctx.insn_seqs.push(typed_index_collections::TiVec::new());
        let new_ctx = FnBuilderCtx {
            insn_seqs: $ctx.insn_seqs,
            curr_seq_id: $label,
            next_val_ref: $ctx.next_val_ref,
        };
        $(
            $crate::push_node!(new_ctx; $(% $val_ref =)? $node_ident $(($($param)*))* $($body_label: {$($body)*})*);
        )*
    }
}

#[macro_export]
macro_rules! lir_function {
    (fn $func:ident($($param_ty:ident),*) -> ($($return_ty:ident),*) $label:ident: { $($inner:tt)* }) => {
        struct FnBuilderCtx<'a> {
            insn_seqs: &'a mut typed_index_collections::TiVec<$crate::lir::InsnSeqId, typed_index_collections::TiVec<$crate::lir::InsnPc, $crate::lir::InsnKind>>,
            curr_seq_id: $crate::lir::InsnSeqId,
            next_val_ref: &'a mut $crate::lir::ValRef,
        }
        let mut insn_seqs = typed_index_collections::TiVec::new();
        let mut next_val_ref = $crate::lir::ValRef(0);

        let ctx = FnBuilderCtx {
            insn_seqs: &mut insn_seqs,
            curr_seq_id: $crate::lir::InsnSeqId(0),
            next_val_ref: &mut next_val_ref,
        };
        $crate::instruction_seq!(ctx; $label: { $($inner)* });

        let $func = $crate::lir::Function {
            parameter_types: vec![$($crate::lir::ValType::$param_ty),*],
            body: $crate::lir::Block {
                output_type: $crate::lir::InsnOutput::from_types_array([$($crate::lir::ValType::$return_ty),*]),
                body: $crate::lir::InsnSeqId(0),
            },
            insn_seqs,
        };
    }
}
