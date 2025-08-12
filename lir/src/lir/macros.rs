pub use crate::instructions;

#[macro_export]
macro_rules! make_node {
    ($insns:expr; constant($ty:ident, $value:expr)) => {
        InsnKind::Const {
            r#type: ValType::$ty,
            value: $value,
        }
    };
    ($insns:expr; argument($ty:ident, $index:expr)) => {
        InsnKind::Argument {
            r#type: ValType::$ty,
            index: $index,
        }
    };
    ($insns:expr; project($index:expr)($multivalue:tt)) => {
        {
            let multivalue = $crate::push_node!($insns; $multivalue);
            InsnKind::Project {
                multivalue,
                index: $index,
            }
        }
    };
    ($insns:expr; derive_field($offset:expr)($ptr:tt)) => {
        {
            let ptr = $crate::push_node!($insns; $ptr);
            InsnKind::DeriveField {
                offset: $offset,
                ptr,
            }
        }
    };
    ($insns:expr; derive_element($element_size:expr)($ptr:tt, $index:tt)) => {
        {
            let ptr = $crate::push_node!($insns; $ptr);
            let index = $crate::push_node!($insns; $index);
            InsnKind::DeriveElement {
                element_size: $element_size,
                ptr,
                index,
            }
        }
    };
    ($insns:expr; mem_load($ty:ident, $offset:expr)($ptr:tt)) => {
        {
            let ptr = $crate::push_node!($insns; $ptr);
            InsnKind::MemLoad {
                r#type: ValType::$ty,
                offset: $offset,
                ptr,
            }
        }
    };
    ($insns:expr; mem_store($offset:expr)($ptr:tt, $value:tt)) => {
        {
            let ptr = $crate::push_node!($insns; $ptr);
            let value = $crate::push_node!($insns; $value);
            InsnKind::MemStore {
                offset: $offset,
                ptr,
                value,
            }
        }
    };
    ($insns:expr; stack_load($ty:ident, $offset:expr)) => {
        InsnKind::StackLoad {
            r#type: ValType::$ty,
            offset: $offset,
        }
    };
    ($insns:expr; stack_store($offset:expr)($value:tt)) => {
        {
            let value = $crate::push_node!($insns; $value);
            InsnKind::StackStore {
                offset: $offset,
                value,
            }
        }
    };
    ($insns:expr; call_imported_function($function:expr)($($args:tt),*)) => {
        {
            let args = vec![$(
                $crate::push_node!($insns; $args)
            ),*].into_boxed_slice();
            InsnKind::CallImportedFunction {
                function: $function,
                args,
            }
        }
    };
    ($insns:expr; call_user_function($function:expr)($($args:tt),*)) => {
        {
            let args = vec![$(
                $crate::push_node!($insns; $args)
            ),*].into_boxed_slice();
            InsnKind::CallUserFunction {
                function: $function,
                args,
            }
        }
    };
    ($insns:expr; break_($depth:expr)($($values:tt),*)) => {
        {
            let values = Box::new([$(
                $crate::push_node!($insns; $values)
            ),*]);
            InsnKind::Break {
                depth: $depth,
                values,
            }
        }
    };
    ($insns:expr; break_if($depth:expr)($condition:tt, $($values:tt),*)) => {
        {
            let condition = $crate::push_node!($insns; $condition);
            let values = Box::new([$(
                $crate::push_node!($insns; $values)
            ),*]);
            InsnKind::ConditionalBreak {
                condition,
                depth: $depth,
                values,
            }
        }
    };
    ($insns:expr; loop_arg($initial_value:tt)) => {
        {
            let initial_value = $crate::push_node!($insns; $initial_value);
            InsnKind::LoopArgument {
                initial_value,
            }
        }
    };
    ($insns:expr; add($lhs:tt, $rhs:tt)) => {
        {
            let lhs = $crate::push_node!($insns; $lhs);
            let rhs = $crate::push_node!($insns; $rhs);
            InsnKind::BinaryOp {
                op: BinaryOpcode::Add,
                lhs,
                rhs,
            }
        }
    }
}

#[macro_export]
macro_rules! push_node {
    ($insns:expr; [$node_kind:ident $($node_params:tt)*]) => {
        {
            let node = $crate::make_node!($insns; $node_kind $($node_params)*);
            let pc = $insns.len();
            $insns.push(node);
            $crate::lir::InsnPc::from(pc)
        }
    };
    ($insns:expr; $value:expr) => {
        $value
    };
}

#[macro_export]
macro_rules! instructions {
    (let $insns:ident; $($node_label:ident = $node:tt; )*) => {
        let mut $insns: TiVec<InsnPc, InsnKind> = TiVec::new();
        $(
            let $node_label = $crate::push_node!($insns; $node);
        )*
    }
}

#[cfg(test)]
mod tests {
    use typed_index_collections::ti_vec;

    use super::super::*;
    use super::*;

    #[test]
    fn test_macro() {
        instructions! {
            let insns;
            a = [constant(I32, 10)];
            b = [constant(I32, 20)];
            _c = [project(0)(b)];
            _d = [add(a, [constant(I32, 30)])];
        };
        println!("{:?}", insns);
    }

    #[test]
    #[allow(dead_code)]
    fn test_nested_nodes() {
        instructions! {
            let insns;
            // Basic constants
            _const_10 = [constant(I32, 10)];
            _const_20 = [constant(I32, 20)];
            _const_ptr = [constant(Ptr, 0x1000)];

            // Nested operations - add with inline constants
            _sum = [add([constant(I32, 5)], [constant(I32, 15)])];

            // Complex nested operations - add with inline add
            _complex_sum = [add(
                [add([constant(I32, 1)], [constant(I32, 2)])],
                [add([constant(I32, 3)], [constant(I32, 4)])]
            )];

            // Memory operations with nested pointer derivation
            field_ptr = [derive_field(8)([constant(Ptr, 0x2000)])];
            _element_ptr = [derive_element(4)(
                [constant(Ptr, 0x3000)],
                [constant(I32, 5)]
            )];

            // Load from derived pointer
            _loaded_value = [mem_load(I32, 0)(field_ptr)];

            // Store with nested value calculation
            _stored_value = [mem_store(0)(
                [constant(Ptr, 0x4000)],
                [add([constant(I32, 100)], [constant(I32, 200)])]
            )];

            // Stack operations with nested values
            _stack_stored = [stack_store(0)(
                [add([constant(I32, 50)], [constant(I32, 25)])]
            )];

            // Function call with nested arguments
            _func_call = [call_user_function(FunctionId)(
                [constant(I32, 1)],
                [add([constant(I32, 10)], [constant(I32, 20)])],
                [project(0)([constant(I32, 42)])]
            )];

            // Break with nested values
            _break_instr = [break_(0)(
                [constant(I32, 0)],
                [add([constant(I32, 1)], [constant(I32, 2)])]
            )];

            // Conditional break with nested condition
            _cond_break = [break_if(1)(
                [add([constant(I32, 5)], [constant(I32, 3)])],
                [constant(I32, 42)]
            )];

            // Loop argument with nested initial value
            _loop_instr = [loop_arg(
                [add([constant(I32, 1)], [constant(I32, 1)])]
            )];
        };

        use BinaryOpcode::*;
        use InsnKind::*;
        use ValType::*;
        #[rustfmt::skip]
        assert_eq!(
            insns,
            ti_vec![
                /* 0*/ Const { r#type: I32, value: 10 },
                /* 1*/ Const { r#type: I32, value: 20 },
                /* 2*/ Const { r#type: Ptr, value: 4096 },
                /* 3*/ Const { r#type: I32, value: 5 },
                /* 4*/ Const { r#type: I32, value: 15 },
                /* 5*/ BinaryOp { op: Add, lhs: InsnPc(3), rhs: InsnPc(4) },
                /* 6*/ Const { r#type: I32, value: 1 },
                /* 7*/ Const { r#type: I32, value: 2 },
                /* 8*/ BinaryOp { op: Add, lhs: InsnPc(6), rhs: InsnPc(7) },
                /* 9*/ Const { r#type: I32, value: 3 },
                /*10*/ Const { r#type: I32, value: 4 },
                /*11*/ BinaryOp { op: Add, lhs: InsnPc(9), rhs: InsnPc(10) },
                /*12*/ BinaryOp { op: Add, lhs: InsnPc(8), rhs: InsnPc(11) },
                /*13*/ Const { r#type: Ptr, value: 8192 },
                /*14*/ DeriveField { offset: 8, ptr: InsnPc(13) },
                /*15*/ Const { r#type: Ptr, value: 12288 },
                /*16*/ Const { r#type: I32, value: 5 },
                /*17*/ DeriveElement { element_size: 4, ptr: InsnPc(15), index: InsnPc(16) },
                /*18*/ MemLoad { r#type: I32, offset: 0, ptr: InsnPc(14) },
                /*19*/ Const { r#type: Ptr, value: 16384 },
                /*20*/ Const { r#type: I32, value: 100 },
                /*21*/ Const { r#type: I32, value: 200 },
                /*22*/ BinaryOp { op: Add, lhs: InsnPc(20), rhs: InsnPc(21) },
                /*23*/ MemStore { offset: 0, ptr: InsnPc(19), value: InsnPc(22) },
                /*24*/ Const { r#type: I32, value: 50 },
                /*25*/ Const { r#type: I32, value: 25 },
                /*26*/ BinaryOp { op: Add, lhs: InsnPc(24), rhs: InsnPc(25) },
                /*27*/ StackStore { offset: 0, value: InsnPc(26) },
                /*28*/ Const { r#type: I32, value: 1 },
                /*29*/ Const { r#type: I32, value: 10 },
                /*30*/ Const { r#type: I32, value: 20 },
                /*31*/ BinaryOp { op: Add, lhs: InsnPc(29), rhs: InsnPc(30) },
                /*32*/ Const { r#type: I32, value: 42 },
                /*33*/ Project { multivalue: InsnPc(32), index: 0 },
                /*34*/ CallUserFunction { function: FunctionId, args: Box::new([InsnPc(28), InsnPc(31), InsnPc(33)]) },
                /*35*/ Const { r#type: I32, value: 0 },
                /*36*/ Const { r#type: I32, value: 1 },
                /*37*/ Const { r#type: I32, value: 2 },
                /*38*/ BinaryOp { op: Add, lhs: InsnPc(36), rhs: InsnPc(37) },
                /*39*/ Break { depth: 0, values: Box::new([InsnPc(35), InsnPc(38)]) },
                /*40*/ Const { r#type: I32, value: 5 },
                /*41*/ Const { r#type: I32, value: 3 },
                /*42*/ BinaryOp { op: Add, lhs: InsnPc(40), rhs: InsnPc(41) },
                /*43*/ Const { r#type: I32, value: 42 },
                /*44*/ ConditionalBreak { condition: InsnPc(42), depth: 1, values: Box::new([InsnPc(43)]) },
                /*45*/ Const { r#type: I32, value: 1 },
                /*46*/ Const { r#type: I32, value: 1 },
                /*47*/ BinaryOp { op: Add, lhs: InsnPc(45), rhs: InsnPc(46) },
                /*48*/ LoopArgument { initial_value: InsnPc(47) }
            ]
        );
    }
}
