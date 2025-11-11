#[macro_export]
macro_rules! push_node {
    ($ctx:expr; [$val_ref:ident] = $name:ident) => {
        let $val_ref = $name;
    };
    ($ctx:expr; [$val_ref:ident] = constant($ty:ident, $value:expr)) => {
        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::Const($crate::Const {
                ty: $crate::ValType::$ty,
                value: $value,
            })
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; [$val_ref:ident] = user_fn_ptr($function:ident)) => {
        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::UserFunctionPtr {
                function: $function,
            }
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; [$val_ref:ident] = loop_argument($initial_value:ident)) => {
        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::LoopArgument {
                initial_value: $initial_value,
            }
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; [$val_ref:ident] = derive_field($offset:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; [ptr] = $ptr_ident $(($($ptr_param)*))*);

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::DeriveField {
                offset: $offset,
                ptr,
            }
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; [$val_ref:ident] = derive_element($element_size:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))*,
        $index_ident:ident $(($($index_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; [ptr] = $ptr_ident $(($($ptr_param)*))*);
        $crate::push_node!($ctx; [index] = $index_ident $(($($index_param)*))*);

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::DeriveElement {
                element_size: $element_size,
                ptr,
                index,
            }
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; [$val_ref:ident] = var_load($var_id:expr)) => {
        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::VarLoad {
                var_id: $var_id,
            },
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; var_store($var_id:expr)(
        $value_ident:ident $(($($value_param:tt)*))*
    )) => {
        $crate::push_node!($ctx; [value] = $value_ident $(($($value_param)*))*);

        $ctx.0[$ctx.1].push($crate::InsnKind::VarStore {
            var_id: $var_id,
            value,
        });
    };
    ($ctx:expr; [$val_ref:ident] = mem_load($ty:ident, $offset:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; [ptr] = $ptr_ident $(($($ptr_param)*))*);

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::MemLoad {
                r#type: $crate::ValType::$ty,
                offset: $offset,
                ptr,
            }
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; mem_store($offset:expr)(
        $ptr_ident:ident $(($($ptr_param:tt)*))*,
        $value_ident:ident $(($($value_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; [ptr] = $ptr_ident $(($($ptr_param)*))*);
        $crate::push_node!($ctx; [value] = $value_ident $(($($value_param)*))*);

        $ctx.0[$ctx.1].push($crate::InsnKind::MemStore {
            offset: $offset,
            ptr,
            value,
        });
    };
    ($ctx:expr; [$val_ref:ident] = stack_load($ty:ident, $offset:expr)) => {
        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::StackLoad {
                r#type: $crate::ValType::$ty,
                offset: $offset,
            }
        );
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; stack_store($offset:expr)(
        $value_ident:ident $(($($value_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; [value] = $value_ident $(($($value_param)*))*);

        $ctx.0[$ctx.1].push($crate::InsnKind::StackStore {
            offset: $offset,
            value,
        });
    };
    ($ctx:expr; [$val_ref:ident] = stack_addr($offset:expr)) => {
        let insn_idx = $ctx.0[$ctx.1].push_and_get_key($crate::InsnKind::StackAddr { offset: $offset });
        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; [$($val_ref:ident),*] = call_host_fn($function:ident -> [$($return_ty:ident),*])(
        $(
            $arg_ident:ident $(($($arg_param:tt)*))*
        ),* $(,)?
    )) => {
        let mut args = Vec::new();
        $(
            $crate::push_node!($ctx; [arg] = $arg_ident $(($($arg_param)*))*);
            args.push(arg);
        )*

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::CallHostFunction {
                function: $function,
                output_type: $crate::smallvec::smallvec![$($crate::ValType::$return_ty),*],
                args: args.into_boxed_slice(),
            }
        );

        let mut i = 0;
        let ($($val_ref),*) = ($({
            // include the return type here to ensure we get the right number of
            // repetitions
            $crate::ValType::$return_ty;
            let j = i;
            i += 1;
            let v = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), j);
            $ctx.2.insert(v, stringify!($val_ref).into());
            v
        }),*);
    };
    ($ctx:expr; [$($val_ref:ident),*] = call_user_function($function:ident -> [$($return_ty:ident),*])(
        $(
            $arg_ident:ident $(($($arg_param:tt)*))*
        ),* $(,)?
    )) => {
        let mut args = Vec::new();
        $(
            $crate::push_node!($ctx; [arg] = $arg_ident $(($($arg_param)*))*);
            args.push(arg);
        )*
        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::CallUserFunction {
                function: $function,
                output_type: $crate::smallvec::smallvec![$($crate::ValType::$return_ty),*],
                args: args.into_boxed_slice(),
            }
        );

        let mut i = 0;
        let ($($val_ref),*) = ($({
            // include the return type here to ensure we get the right number of
            // repetitions
            $crate::ValType::$return_ty;
            let j = i;
            i += 1;
            let v = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), j);
            $ctx.2.insert(v, stringify!($val_ref).into());
            v
        }),*);
    };
    ($ctx:expr; [$val_ref:ident] = $unary_op:ident(
        $operand_ident:ident $(($($operand_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; [operand] = $operand_ident $(($($operand_param)*))*);

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::UnaryOp {
                op: $crate::UnaryOpcode::$unary_op,
                operand,
            }
        );

        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; [$val_ref:ident] = $binary_op:ident(
        $lhs_ident:ident $(($($lhs_param:tt)*))*,
        $rhs_ident:ident $(($($rhs_param:tt)*))* $(,)?
    )) => {
        $crate::push_node!($ctx; [lhs] = $lhs_ident $(($($lhs_param)*))*);
        $crate::push_node!($ctx; [rhs] = $rhs_ident $(($($rhs_param)*))*);

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::BinaryOp {
                op: $crate::BinaryOpcode::$binary_op,
                lhs,
                rhs,
            }
        );

        let $val_ref = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), 0);
        $ctx.2.insert($val_ref, stringify!($val_ref).into());
    };
    ($ctx:expr; break_($target:expr)(
        $($value_ident:ident $(($($value_param:tt)*))*),* $(,)?
    )) => {
        let mut values = Vec::new();
        $(
            $crate::push_node!($ctx; [value] = $value_ident $(($($value_param)*))*);
            values.push(value);
        )*

        $ctx.0[$ctx.1].push($crate::InsnKind::Break {
            target: $target,
            values: values.into_boxed_slice(),
        });
    };
    ($ctx:expr; break_if($target:expr)(
        $condition_ident:ident $(($($condition_param:tt)*))*,
        $($value_ident:ident $(($($value_param:tt)*))*),* $(,)?
    )) => {
        $crate::push_node!($ctx; [condition] = $condition_ident $(($($condition_param)*))*);

        let mut values = Vec::new();
        $(
            $crate::push_node!($ctx; [value] = $value_ident $(($($value_param)*))*);
            values.push(value);
        )*

        $ctx.0[$ctx.1].push($crate::InsnKind::ConditionalBreak {
            target: $target,
            condition,
            values: values.into_boxed_slice(),
        });
    };
    ($ctx:expr; [$($val_ref:ident),*] = block(-> [$($return_ty:ident),*]) $label:ident: {$($inner:tt)*}) => {
        $crate::instruction_seq!($ctx; $label: { $($inner)* });

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::Block($crate::Block {
                output_type: $crate::smallvec::smallvec![$($crate::ValType::$return_ty),*],
                body: $label,
            })
        );

        let mut i = 0;
        let [$($val_ref),*] = [$({
            // include the return type here to ensure we get the right number of
            // repetitions
            $crate::ValType::$return_ty;
            let j = i;
            i += 1;
            let v = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), j);
            $ctx.2.insert(v, stringify!($val_ref).into());
            v
        }),*];
    };
    ($ctx:expr; [$($val_ref:ident),*] = if_else(-> [$($return_ty:ident),*])(
        $condition_ident:ident $(($($condition_param:tt)*))*
    ) $then_label:ident: {$($then_inner:tt)*} $else_label:ident: {$($else_inner:tt)*}) => {
        $crate::push_node!($ctx; [condition] = $condition_ident $(($($condition_param)*))*);

        $crate::instruction_seq!($ctx; $then_label: { $($then_inner)* });
        $crate::instruction_seq!($ctx; $else_label: { $($else_inner)* });

        let insn_idx = $ctx.0[$ctx.1].push_and_get_key(
            $crate::InsnKind::IfElse($crate::IfElse {
                output_type: $crate::smallvec::smallvec![$($crate::ValType::$return_ty),*],
                condition,
                then_body: $then_label,
                else_body: $else_label,
            })
        );

        let mut i = 0;
        let ($($val_ref),*) = ($({
            // include the return type here to ensure we get the right number of
            // repetitions
            $crate::ValType::$return_ty;
            let j = i;
            i += 1;
            let v = $crate::ValRef($crate::InsnPc($ctx.1, insn_idx), j);
            $ctx.2.insert(v, stringify!($val_ref).into());
            v
        }),*);
    };
}

#[macro_export]
macro_rules! instruction_seq {
    ($ctx:expr; $label:ident: { $($([$($val_ref:ident),*] =)? $node_ident:ident $(($($param:tt)*))* $($body_label:ident: {$($body:tt)*})*; )* }) => {
        let $label = $ctx.0.next_key();
        $ctx.0.push($crate::typed_index_collections::TiVec::new());
        let new_ctx = (&mut *$ctx.0, $label, &mut *$ctx.2);
        $(
            $crate::push_node!(new_ctx; $([$($val_ref),*] =)? $node_ident $(($($param)*))* $($body_label: {$($body)*})*);
        )*
    }
}

// FIXME fix macro to work with new function parameter passing (i.e. they are
// used to initialize first X local variables rather than coming from some
// "arguments" instruction)
#[macro_export]
macro_rules! lir_function {
    (
        fn $func:ident($($param_ty:ident $param_name:ident),*) -> [$($return_ty:ident),*],
        vars: [$($var_ty:ident $var_name:ident),*],
        stack_space: $stack_space:expr,
        $label:ident: { $($inner:tt)* }
    ) => {
        let mut i = 0;
        let mut debug_var_names: std::collections::HashMap<$crate::VarId, std::rc::Rc<str>> = std::collections::HashMap::new();
        $(
            let $param_name: $crate::VarId = i.into();
            debug_var_names.insert($param_name, stringify!($param_name).into());
            i += 1;
        )*
        let num_parameters = i;
        $(
            let $var_name: $crate::VarId = i.into();
            debug_var_names.insert($var_name, stringify!($var_name).into());
            i += 1;
        )*

        let mut insn_seqs: $crate::typed_index_collections::TiVec<$crate::InsnSeqId, $crate::typed_index_collections::TiVec<$crate::InsnIdx, $crate::InsnKind>> = $crate::typed_index_collections::TiVec::new();
        let mut debug_val_names = std::collections::HashMap::new();
        let ctx = (&mut insn_seqs, $crate::InsnSeqId(0), &mut debug_val_names);


        $crate::instruction_seq!(ctx; $label: { $($inner)* });

        let $func = $crate::Function {
            local_vars: $crate::typed_index_collections::ti_vec![
                $($crate::ValType::$param_ty,)*
                $($crate::ValType::$var_ty,)*
            ],
            num_parameters,
            stack_space: $stack_space,
            body: $crate::Block {
                output_type: $crate::smallvec::smallvec![$($crate::ValType::$return_ty),*],
                body: $crate::InsnSeqId(0),
            },
            insn_seqs,
            debug_fn_name: Some(stringify!($func).into()),
            debug_val_names,
            debug_var_names,
        };
    }
}
