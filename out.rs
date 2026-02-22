#[other_attributes]
fn my_function(a: MyPair, b: MyPair) -> Thing {
    do_something()
}
mod my_function {
    use super::*;
    use ffi_translate::ComponentIndex as Ci;
    use ffi_translate::FfiCompose as Fc;
    use lir::{AsLir, HostFunctionInfo};
    pub static INFO: HostFunctionInfo = HostFunctionInfo {
        name: "my_function_unmangled",
        parameter_types: &[
            <<<MyPair as Fc>::Components as Ci<0usize>>::Component as AsLir>::Type,
            <<<MyPair as Fc>::Components as Ci<1usize>>::Component as AsLir>::Type,
            <<<MyPair as Fc>::Components as Ci<0usize>>::Component as AsLir>::Type,
            <<<MyPair as Fc>::Components as Ci<1usize>>::Component as AsLir>::Type,
        ],
        return_type: &[<Thing as AsLir>::Type],
    };
    #[unsafe(export_name = "my_function_unmangled")]
    fn my_function_unmangled(
        a_0: <<MyPair as Fc>::Components as Ci<0usize>>::Component,
        a_1: <<MyPair as Fc>::Components as Ci<1usize>>::Component,
        b_0: <<MyPair as Fc>::Components as Ci<0usize>>::Component,
        b_1: <<MyPair as Fc>::Components as Ci<1usize>>::Component,
    ) -> Thing {
        let a = <MyPair as Fc>::recompose((a_0, a_1));
        let b = <MyPair as Fc>::recompose((b_0, b_1));
        super::my_function(a, b)
    }
}
