pub use crate::instructions;

#[macro_export]
macro_rules! push_node {
    ($insns:expr; $label:ident; $node_label:ident) => {
        let $label = $node_label;
    };
    ($insns:expr; $label:ident; [constant($ty:ident, $value:expr)]) => {
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::Const {
            r#type: ValType::$ty,
            value: $value,
        });
    };
    ($insns:expr; $label:ident; [argument($ty:ident, $index:expr)]) => {
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::Argument {
            r#type: ValType::$ty,
            index: $index,
        });
    };
    ($insns:expr; $label:ident; [project($index:expr)($multivalue:tt)]) => {
        $crate::push_node!($insns; multivalue; $multivalue);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::Project {
            multivalue,
            index: $index,
        });
    };
    ($insns:expr; $label:ident; [derive_field($offset:expr)($ptr:tt)]) => {
        $crate::push_node!($insns; ptr; $ptr);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::DeriveField {
            offset: $offset,
            ptr,
        });
    };
    ($insns:expr; $label:ident; [derive_element($element_size:expr)($ptr:tt, $index:tt)]) => {
        $crate::push_node!($insns; ptr; $ptr);
        $crate::push_node!($insns; index; $index);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::DeriveElement {
            element_size: $element_size,
            ptr,
            index,
        });
    };
    ($insns:expr; $label:ident; [mem_load($ty:ident, $offset:expr)($ptr:tt)]) => {
        $crate::push_node!($insns; ptr; $ptr);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::MemLoad {
            r#type: ValType::$ty,
            offset: $offset,
            ptr,
        });
    };
    ($insns:expr; $label:ident; [mem_store($offset:expr)($ptr:tt, $value:tt)]) => {
        $crate::push_node!($insns; ptr; $ptr);
        $crate::push_node!($insns; value; $value);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::MemStore {
            offset: $offset,
            ptr,
            value,
        });
    };
    ($insns:expr; $label:ident; [stack_load($ty:ident, $offset:expr)]) => {
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::StackLoad {
            r#type: ValType::$ty,
            offset: $offset,
        });
    };
    ($insns:expr; $label:ident; [stack_store($offset:expr)($value:tt)]) => {
        $crate::push_node!($insns; value; $value);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::StackStore {
            offset: $offset,
            value,
        });
    };
    ($insns:expr; $label:ident; [call_imported_function($function:expr)($($args:tt),*)]) => {
        let mut args_vec = Vec::new();
        $(
            $crate::push_node!($insns; arg; $args);
            args_vec.push(arg);
        )*
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::CallImportedFunction {
            function: $function,
            args: args_vec.into_boxed_slice(),
        });
    };
    ($insns:expr; $label:ident; [call_user_function($function:expr)($($args:tt),*)]) => {
        let mut args_vec = Vec::new();
        $(
            $crate::push_node!($insns; arg; $args);
            args_vec.push(arg);
        )*
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::CallUserFunction {
            function: $function,
            args: args_vec.into_boxed_slice(),
        });
    };
    ($insns:expr; $label:ident; [break_($depth:expr)($($values:tt),*)]) => {
        let mut values_vec = Vec::new();
        $(
            $crate::push_node!($insns; value; $values);
            values_vec.push(value);
        )*
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::Break {
            depth: $depth,
            values: values_vec.into_boxed_slice(),
        });
    };
    ($insns:expr; $label:ident; [break_if($depth:expr)($condition:tt, $($values:tt),*)]) => {
        $crate::push_node!($insns; condition; $condition);
        let mut values_vec = Vec::new();
        $(
            $crate::push_node!($insns; value; $values);
            values_vec.push(value);
        )*
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::ConditionalBreak {
            condition,
            depth: $depth,
            values: values_vec.into_boxed_slice(),
        });
    };
    ($insns:expr; $label:ident; [loop_argument($initial_value:tt)]) => {
        $crate::push_node!($insns; initial_value; $initial_value);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::LoopArgument {
            initial_value,
        });
    };
    ($insns:expr; $label:ident; [add($lhs:tt, $rhs:tt)]) => {
        $crate::push_node!($insns; lhs; $lhs);
        $crate::push_node!($insns; rhs; $rhs);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::BinaryOp {
            op: BinaryOpcode::Add,
            lhs,
            rhs,
        });
    };
    ($insns:expr; $label:ident; [sub($lhs:tt, $rhs:tt)]) => {
        $crate::push_node!($insns; lhs; $lhs);
        $crate::push_node!($insns; rhs; $rhs);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::BinaryOp {
            op: BinaryOpcode::Sub,
            lhs,
            rhs,
        });
    };
    ($insns:expr; $label:ident; [block { $($body:tt)* }]) => {
        let old_len = $insns.len();
        let $label = InsnPc::from(old_len);
        $insns.push(InsnKind::Block {
            body_len: 0,
        });
        $crate::instruction_seq!($insns; $($body)*);
        let new_len = $insns.len();
        let InsnKind::Block { body_len } = &mut $insns[$label] else {
            panic!("Expected block at {:?}", $label);
        };
        *body_len = new_len - old_len - 1;
    };
    ($insns:expr; $label:ident; [if_else($condition:tt) {$($then:tt)*} {$($else:tt)*}]) => {
        $crate::push_node!($insns; condition; $condition);
        let old_len = $insns.len();
        let $label = InsnPc::from(old_len);
        $insns.push(InsnKind::IfElse {
            condition,
            then_len: 0,
            else_len: 0,
        });
        $crate::instruction_seq!($insns; $($then)*);
        let middle_len = $insns.len();
        $crate::instruction_seq!($insns; $($else)*);
        let new_len = $insns.len();
        let InsnKind::IfElse { then_len, else_len, .. } = &mut $insns[$label] else {
            panic!("Expected if-else at {:?}", $label);
        };
        *then_len = middle_len - old_len - 1;
        *else_len = new_len - middle_len;
    };
    ($insns:expr; $label:ident; [loop_arg($initial:tt)]) => {
        $crate::push_node!($insns; initial_value; $initial);
        let $label = InsnPc::from($insns.len());
        $insns.push(InsnKind::LoopArgument {
            initial_value,
        });
    }
}

#[macro_export]
macro_rules! instruction_seq {
    ($insns:expr; $($node_label:ident = $node:tt; )*) => {
        $(
            $crate::push_node!($insns; $node_label; $node);
        )*
    }
}

#[macro_export]
macro_rules! instructions {
    (let $insns:ident; $($inner:tt)*) => {
        let mut $insns: TiVec<InsnPc, InsnKind> = TiVec::new();
        $crate::instruction_seq!($insns; $($inner)*);
    }
}

#[cfg(test)]
mod tests {
    use BinaryOpcode::*;
    use InsnKind::*;
    use ValType::*;
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

    #[test]
    fn test_block() {
        instructions! {
            let insns;
            block_return_val = [block {
                a = [constant(I32, 10)];
                b = [constant(I32, 20)];
                c = [add(a, b)];
            }];
            d = [add(block_return_val, [constant(I32, 30)])];
        };
        #[rustfmt::skip]
        assert_eq!(
            insns,
            ti_vec![
                /* 0*/ Block { body_len: 3 },
                /* 1*/ Const { r#type: I32, value: 10 },
                /* 2*/ Const { r#type: I32, value: 20 },
                /* 3*/ BinaryOp { op: Add, lhs: InsnPc(1), rhs: InsnPc(2) },
                /* 4*/ Const { r#type: I32, value: 30 },
                /* 5*/ BinaryOp { op: Add, lhs: InsnPc(0), rhs: InsnPc(4) },
            ]
        );
    }

    #[test]
    fn test_nested_blocks() {
        instructions! {
            let insns;
            outer = [block {
                a = [constant(I32, 10)];
                b = [constant(I32, 20)];
                c = [add(a, b)];
                inner = [block {
                    d = [constant(I32, 40)];
                }];
                _0 = [break_(0)(inner)];
            }];
            e = [add(outer, [constant(I32, 30)])];
        };

        #[rustfmt::skip]
        assert_eq!(
            insns,
            ti_vec![
                /*0*/ Block { body_len: 6 },
                /*1*/ Const { r#type: I32, value: 10 },
                /*2*/ Const { r#type: I32, value: 20 },
                /*3*/ BinaryOp { op: Add, lhs: InsnPc(1), rhs: InsnPc(2) },
                /*4*/ Block { body_len: 1 },
                /*5*/ Const { r#type: I32, value: 40 },
                /*6*/ Break { depth: 0, values: Box::new([InsnPc(4)]) },
                /*7*/ Const { r#type: I32, value: 30 },
                /*8*/ BinaryOp { op: Add, lhs: InsnPc(0), rhs: InsnPc(7) }
            ]
        );
    }

    #[test]
    fn test_if_else() {
        instructions! {
            let insns;
            if_else = [if_else([constant(I32, 10)]) {
                a = [constant(I32, 20)];
                b = [add(a, a)];
                c = [add(b, b)];
                _0 = [break_(0)(c)];
            } {
                d = [constant(I32, 30)];
                e = [add(d, d)];
                _0 = [break_(0)(e)];
            }];
            f = [add(if_else, [constant(I32, 40)])];
        };

        #[rustfmt::skip]
        assert_eq!(
            insns,
            ti_vec![
                /*0*/ Const { r#type: I32, value: 10 },
                /*1*/ IfElse { condition: InsnPc(0), then_len: 4, else_len: 3 },
                /*2*/ Const { r#type: I32, value: 20 },
                /*3*/ BinaryOp { op: Add, lhs: InsnPc(2), rhs: InsnPc(2) },
                /*4*/ BinaryOp { op: Add, lhs: InsnPc(3), rhs: InsnPc(3) },
                /*5*/ Break { depth: 0, values: Box::new([InsnPc(4)]) },
                /*6*/ Const { r#type: I32, value: 30 },
                /*7*/ BinaryOp { op: Add, lhs: InsnPc(6), rhs: InsnPc(6) },
                /*8*/ Break { depth: 0, values: Box::new([InsnPc(7)]) },
                /*9*/ Const { r#type: I32, value: 40 },
                /*10*/ BinaryOp { op: Add, lhs: InsnPc(1), rhs: InsnPc(9) }
            ]
        );
    }
}
