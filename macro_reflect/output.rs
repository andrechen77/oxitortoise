#[automatically_derived]
unsafe impl<T: Bound> crate::util::reflection::ReflectComponents for Test<T>
where
    Test<T>: crate::util::reflection::Reflect,
{
    fn mir_type() -> crate::mir::MirType {
        std::sync::Arc::new(crate::mir::MirTypeInfo {
            static_ty: Some(<Self as crate::util::reflection::Reflect>::TYPE),
            contents: std::default::Default::default(),
        })
    }
}

mod __internal_macro_reflect_mod_Test_T_ {
    static TYPE_INFO: crate::util::reflection::TypeInfo = crate::util::reflection::TypeInfo {
        debug_name: stringify!(Test<T>),
        layout: Some(::std::alloc::Layout::new::<Test<T>>()),
        is_zeroable: false,
        clone: crate::util::reflection::CloneKind::Copy,
        drop_fn: Some(|ptr| ::std::ptr::drop_in_place(ptr as *mut Test<T>)),
        make_mir_type: <Test<T> as crate::util::reflection::ReflectComponents>::mir_type,
    };
    unsafe impl crate::util::reflection::Reflect for Test<T> {
        const TYPE: crate::util::reflection::Type = &TYPE_INFO;
    }
}

mod __internal_macro_reflect_mod_Test_T_ {
    static TYPE_INFO: crate::util::reflection::TypeInfo = crate::util::reflection::TypeInfo {
        debug_name: stringify!(Test<T>),
        layout: Some(::std::alloc::Layout::new::<Test<T>>()),
        is_zeroable: false,
        clone: crate::util::reflection::CloneKind::Dynamic { clone_fn_info: &CLONE_HOST_FN_INFO },
        drop_fn: Some(|ptr| ::std::ptr::drop_in_place(ptr as *mut Test<T>)),
        make_mir_type: <Test<T> as crate::util::reflection::ReflectComponents>::mir_type,
    };
    static CLONE_HOST_FN_INFO: crate::mir::HostFunctionInfo = crate::mir::HostFunctionInfo {
        debug_name: concat!(stringify!(Test<T>), "::clone"),
        parameter_types: &[<Test<T> as crate::util::reflection::Reflect>::TYPE],
        return_type: <Test<T> as crate::util::reflection::Reflect>::TYPE,
    };
    unsafe impl crate::util::reflection::Reflect for Test<T> {
        const TYPE: crate::util::reflection::Type = &TYPE_INFO;
    }
}
