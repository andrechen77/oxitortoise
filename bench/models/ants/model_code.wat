(module $model_code.wasm
  (@dylink.0
    (mem-info)
  )
  (type (;0;) (func (param f64 f64 f64 f64) (result f64)))
  (type (;1;) (func (param f64) (result f64)))
  (type (;2;) (func (param i32 i32 i32 f64 f64)))
  (type (;3;) (func (param f64) (result i32)))
  (type (;4;) (func (param i32 i32) (result i32)))
  (type (;5;) (func (param i32 i32) (result f64)))
  (type (;6;) (func (param i32)))
  (type (;7;) (func (param i32) (result i64)))
  (type (;8;) (func (param i32 i64 i64 i32 i32)))
  (type (;9;) (func (param i32 i32)))
  (type (;10;) (func (param i32) (result f64)))
  (type (;11;) (func (param i32 i32 f64)))
  (type (;12;) (func))
  (type (;13;) (func (param i32 i32 f64 f64) (result f64)))
  (type (;14;) (func (param i32 i32 i32)))
  (type (;15;) (func (param i32 i32 i64)))
  (import "env" "memory" (memory (;0;) 0))
  (import "env" "__indirect_function_table" (table (;0;) 0 funcref))
  (import "env" "__stack_pointer" (global $__stack_pointer (;0;) (mut i32)))
  (import "env" "oxitortoise_scale_color" (func $oxitortoise_scale_color (;0;) (type 0)))
  (import "env" "oxitortoise_normalize_heading" (func $oxitortoise_normalize_heading (;1;) (type 1)))
  (import "env" "oxitortoise_offset_distance_by_heading" (func $oxitortoise_offset_distance_by_heading (;2;) (type 2)))
  (import "env" "oxitortoise_is_nan" (func $oxitortoise_is_nan (;3;) (type 3)))
  (import "env" "oxitortoise_round" (func $oxitortoise_round (;4;) (type 1)))
  (import "env" "oxitortoise_patch_at" (func $oxitortoise_patch_at (;5;) (type 4)))
  (import "env" "oxitortoise_distance_euclidean_no_wrap" (func $oxitortoise_distance_euclidean_no_wrap (;6;) (type 5)))
  (import "env" "oxitortoise_next_int" (func $oxitortoise_next_int (;7;) (type 4)))
  (import "env" "oxitortoise_clear_all" (func $oxitortoise_clear_all (;8;) (type 6)))
  (import "env" "oxitortoise_get_default_turtle_breed" (func $oxitortoise_get_default_turtle_breed (;9;) (type 7)))
  (import "env" "oxitortoise_create_turtles" (func $oxitortoise_create_turtles (;10;) (type 8)))
  (import "env" "oxitortoise_for_all_patches" (func $oxitortoise_for_all_patches (;11;) (type 9)))
  (import "env" "oxitortoise_reset_ticks" (func $oxitortoise_reset_ticks (;12;) (type 6)))
  (import "env" "oxitortoise_get_ticks" (func $oxitortoise_get_ticks (;13;) (type 10)))
  (import "env" "oxitortoise_for_all_turtles" (func $oxitortoise_for_all_turtles (;14;) (type 9)))
  (import "env" "oxitortoise_diffuse_8" (func $oxitortoise_diffuse_8 (;15;) (type 11)))
  (import "env" "oxitortoise_advance_tick" (func $oxitortoise_advance_tick (;16;) (type 6)))
  (global $setup_body0 (;3;) (mut i32) (i32.const 0))
  (global $setup_body1 (;4;) (mut i32) (i32.const 0))
  (global $go_body0 (;5;) (mut i32) (i32.const 0))
  (global $go_body1 (;6;) (mut i32) (i32.const 0))
  (export "__wasm_call_ctors" (func $__wasm_call_ctors))
  (export "recolor_patch" (func $recolor_patch))
  (export "chemical_at_angle" (func $chemical_at_angle))
  (export "uphill_chemical" (func $uphill_chemical))
  (export "nest_scent_at_angle" (func $nest_scent_at_angle))
  (export "uphill_nest_scent" (func $uphill_nest_scent))
  (export "setup_body0" (func $setup_body0))
  (export "setup_body1" (func $setup_body1))
  (export "setup" (func $setup))
  (export "shim_setup" (func $shim_setup))
  (export "go_body0" (func $go_body0))
  (export "go_body1" (func $go_body1))
  (export "go" (func $go))
  (export "shim_go" (func $shim_go))
  (func $__wasm_call_ctors (;17;) (type 12))
  (func $recolor_patch (;18;) (type 9) (param i32 i32)
    (local i32 i32 i32 i32 f64)
    (local.set 2
      (i32.shl
        (local.get 1)
        (i32.const 3)))
    (local.set 4
      (i32.load offset=380
        (local.tee 3
          (i32.load
            (local.get 0)))))
    (block ;; label = @1
      (block ;; label = @2
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (i32.load8_u offset=56
                (local.tee 5
                  (i32.add
                    (i32.load offset=344
                      (local.get 3))
                    (i32.mul
                      (local.get 1)
                      (i32.const 80)))))))
          (local.set 6
            (f64.const 0x1.ccp+6 (;=115;)))
          (br 1 (;@2;)))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (f64.gt
                (f64.load offset=48
                  (local.get 5))
                (f64.const 0x0p+0 (;=0;)))))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (f64.ne
                (local.tee 6
                  (f64.load offset=72
                    (local.get 5)))
                (f64.const 0x1p+0 (;=1;))))
            (local.set 6
              (f64.const 0x1.54p+6 (;=85;)))
            (br 2 (;@2;)))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (f64.ne
                (local.get 6)
                (f64.const 0x1p+1 (;=2;))))
            (local.set 6
              (f64.const 0x1.7cp+6 (;=95;)))
            (br 2 (;@2;)))
          (br_if 2 (;@1;)
            (f64.ne
              (local.get 6)
              (f64.const 0x1.8p+1 (;=3;))))
          (local.set 6
            (f64.const 0x1.a4p+6 (;=105;)))
          (br 1 (;@2;)))
        (local.set 6
          (call $oxitortoise_scale_color
            (f64.const 0x1.04p+6 (;=65;))
            (f64.load
              (i32.add
                (i32.load offset=416
                  (local.get 3))
                (local.get 2)))
            (f64.const 0x1.999999999999ap-4 (;=0.1;))
            (f64.const 0x1.4p+2 (;=5;)))))
      (f64.store
        (i32.add
          (local.get 4)
          (local.get 2))
        (local.get 6)))
    (i32.store8
      (local.tee 1
        (i32.add
          (i32.load offset=56
            (local.get 0))
          (local.get 1)))
      (i32.or
        (i32.load8_u
          (local.get 1))
        (i32.const 1)))
  )
  (func $chemical_at_angle (;19;) (type 13) (param i32 i32 f64 f64) (result f64)
    (local i32)
    (global.set $__stack_pointer
      (local.tee 4
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 48))))
    (local.set 3
      (call $oxitortoise_normalize_heading
        (f64.add
          (local.get 2)
          (local.get 3))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 4)
          (i32.const 8))
        (i32.const 8))
      (i64.load
        (i32.add
          (local.get 1)
          (i32.const 8))))
    (i64.store offset=8
      (local.get 4)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 4)
        (i32.const 32))
      (local.get 0)
      (i32.add
        (local.get 4)
        (i32.const 8))
      (local.get 3)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 3
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=32
            (local.get 4))))
      (i32.store offset=24
        (local.get 4)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=32
              (local.get 4)))))
      (i32.store offset=28
        (local.get 4)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=40
              (local.get 4)))))
      (i64.store
        (local.get 4)
        (i64.load offset=24 align=4
          (local.get 4)))
      (local.set 1
        (call $oxitortoise_patch_at
          (local.get 0)
          (local.get 4)))
      (local.set 3
        (f64.load
          (i32.add
            (i32.load offset=416
              (local.get 0))
            (i32.shl
              (local.get 1)
              (i32.const 3))))))
    (global.set $__stack_pointer
      (i32.add
        (local.get 4)
        (i32.const 48)))
    (local.get 3)
  )
  (func $uphill_chemical (;20;) (type 14) (param i32 i32 i32)
    (local i32 f64 i32 i32 f64 f64)
    (global.set $__stack_pointer
      (local.tee 3
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 96))))
    (local.set 4
      (call $oxitortoise_normalize_heading
        (f64.add
          (f64.load
            (local.get 2))
          (f64.const 0x0p+0 (;=0;)))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 56))
        (i32.const 8))
      (i64.load
        (local.tee 5
          (i32.add
            (local.get 1)
            (i32.const 8)))))
    (i64.store offset=56
      (local.get 3)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 3)
        (i32.const 80))
      (local.get 0)
      (i32.add
        (local.get 3)
        (i32.const 56))
      (local.get 4)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 4
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=80
            (local.get 3))))
      (i32.store offset=72
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=80
              (local.get 3)))))
      (i32.store offset=76
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=88
              (local.get 3)))))
      (i64.store offset=48
        (local.get 3)
        (i64.load offset=72 align=4
          (local.get 3)))
      (local.set 6
        (call $oxitortoise_patch_at
          (local.get 0)
          (i32.add
            (local.get 3)
            (i32.const 48))))
      (local.set 4
        (f64.load
          (i32.add
            (i32.load offset=416
              (local.get 0))
            (i32.shl
              (local.get 6)
              (i32.const 3))))))
    (local.set 4
      (local.get 4))
    (local.set 7
      (call $oxitortoise_normalize_heading
        (f64.add
          (f64.load
            (local.get 2))
          (f64.const 0x1.68p+5 (;=45;)))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 32))
        (i32.const 8))
      (i64.load
        (local.get 5)))
    (i64.store offset=32
      (local.get 3)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 3)
        (i32.const 80))
      (local.get 0)
      (i32.add
        (local.get 3)
        (i32.const 32))
      (local.get 7)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 7
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=80
            (local.get 3))))
      (i32.store offset=72
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=80
              (local.get 3)))))
      (i32.store offset=76
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=88
              (local.get 3)))))
      (i64.store offset=24
        (local.get 3)
        (i64.load offset=72 align=4
          (local.get 3)))
      (local.set 5
        (call $oxitortoise_patch_at
          (local.get 0)
          (i32.add
            (local.get 3)
            (i32.const 24))))
      (local.set 7
        (f64.load
          (i32.add
            (i32.load offset=416
              (local.get 0))
            (i32.shl
              (local.get 5)
              (i32.const 3))))))
    (local.set 7
      (local.get 7))
    (local.set 8
      (call $oxitortoise_normalize_heading
        (f64.add
          (f64.load
            (local.get 2))
          (f64.const -0x1.68p+5 (;=-45;)))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 8))
        (i32.const 8))
      (i64.load
        (i32.add
          (local.get 1)
          (i32.const 8))))
    (i64.store offset=8
      (local.get 3)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 3)
        (i32.const 80))
      (local.get 0)
      (i32.add
        (local.get 3)
        (i32.const 8))
      (local.get 8)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 8
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=80
            (local.get 3))))
      (i32.store offset=72
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=80
              (local.get 3)))))
      (i32.store offset=76
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=88
              (local.get 3)))))
      (i64.store
        (local.get 3)
        (i64.load offset=72 align=4
          (local.get 3)))
      (local.set 1
        (call $oxitortoise_patch_at
          (local.get 0)
          (local.get 3)))
      (local.set 8
        (f64.load
          (i32.add
            (i32.load offset=416
              (local.get 0))
            (i32.shl
              (local.get 1)
              (i32.const 3))))))
    (local.set 8
      (local.get 8))
    (block ;; label = @1
      (block ;; label = @2
        (br_if 0 (;@2;)
          (f64.gt
            (local.get 7)
            (local.get 4)))
        (br_if 1 (;@1;)
          (i32.eqz
            (f64.gt
              (local.get 8)
              (local.get 4)))))
      (f64.store
        (local.get 2)
        (call $oxitortoise_normalize_heading
          (f64.add
            (f64.load
              (local.get 2))
            (select
              (f64.const 0x1.68p+5 (;=45;))
              (f64.const -0x1.68p+5 (;=-45;))
              (f64.gt
                (local.get 7)
                (local.get 8)))))))
    (global.set $__stack_pointer
      (i32.add
        (local.get 3)
        (i32.const 96)))
  )
  (func $nest_scent_at_angle (;21;) (type 13) (param i32 i32 f64 f64) (result f64)
    (local i32)
    (global.set $__stack_pointer
      (local.tee 4
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 48))))
    (local.set 3
      (call $oxitortoise_normalize_heading
        (f64.add
          (local.get 2)
          (local.get 3))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 4)
          (i32.const 8))
        (i32.const 8))
      (i64.load
        (i32.add
          (local.get 1)
          (i32.const 8))))
    (i64.store offset=8
      (local.get 4)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 4)
        (i32.const 32))
      (local.get 0)
      (i32.add
        (local.get 4)
        (i32.const 8))
      (local.get 3)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 3
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=32
            (local.get 4))))
      (i32.store offset=24
        (local.get 4)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=32
              (local.get 4)))))
      (i32.store offset=28
        (local.get 4)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=40
              (local.get 4)))))
      (i64.store
        (local.get 4)
        (i64.load offset=24 align=4
          (local.get 4)))
      (local.set 1
        (call $oxitortoise_patch_at
          (local.get 0)
          (local.get 4)))
      (local.set 3
        (f64.load offset=64
          (i32.add
            (i32.load offset=344
              (local.get 0))
            (i32.mul
              (local.get 1)
              (i32.const 80))))))
    (global.set $__stack_pointer
      (i32.add
        (local.get 4)
        (i32.const 48)))
    (local.get 3)
  )
  (func $uphill_nest_scent (;22;) (type 14) (param i32 i32 i32)
    (local i32 f64 i32 i32 f64 f64)
    (global.set $__stack_pointer
      (local.tee 3
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 96))))
    (local.set 4
      (call $oxitortoise_normalize_heading
        (f64.add
          (f64.load
            (local.get 2))
          (f64.const 0x0p+0 (;=0;)))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 56))
        (i32.const 8))
      (i64.load
        (local.tee 5
          (i32.add
            (local.get 1)
            (i32.const 8)))))
    (i64.store offset=56
      (local.get 3)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 3)
        (i32.const 80))
      (local.get 0)
      (i32.add
        (local.get 3)
        (i32.const 56))
      (local.get 4)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 4
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=80
            (local.get 3))))
      (i32.store offset=72
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=80
              (local.get 3)))))
      (i32.store offset=76
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=88
              (local.get 3)))))
      (i64.store offset=48
        (local.get 3)
        (i64.load offset=72 align=4
          (local.get 3)))
      (local.set 6
        (call $oxitortoise_patch_at
          (local.get 0)
          (i32.add
            (local.get 3)
            (i32.const 48))))
      (local.set 4
        (f64.load offset=64
          (i32.add
            (i32.load offset=344
              (local.get 0))
            (i32.mul
              (local.get 6)
              (i32.const 80))))))
    (local.set 4
      (local.get 4))
    (local.set 7
      (call $oxitortoise_normalize_heading
        (f64.add
          (f64.load
            (local.get 2))
          (f64.const 0x1.68p+5 (;=45;)))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 32))
        (i32.const 8))
      (i64.load
        (local.get 5)))
    (i64.store offset=32
      (local.get 3)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 3)
        (i32.const 80))
      (local.get 0)
      (i32.add
        (local.get 3)
        (i32.const 32))
      (local.get 7)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 7
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=80
            (local.get 3))))
      (i32.store offset=72
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=80
              (local.get 3)))))
      (i32.store offset=76
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=88
              (local.get 3)))))
      (i64.store offset=24
        (local.get 3)
        (i64.load offset=72 align=4
          (local.get 3)))
      (local.set 5
        (call $oxitortoise_patch_at
          (local.get 0)
          (i32.add
            (local.get 3)
            (i32.const 24))))
      (local.set 7
        (f64.load offset=64
          (i32.add
            (i32.load offset=344
              (local.get 0))
            (i32.mul
              (local.get 5)
              (i32.const 80))))))
    (local.set 7
      (local.get 7))
    (local.set 8
      (call $oxitortoise_normalize_heading
        (f64.add
          (f64.load
            (local.get 2))
          (f64.const -0x1.68p+5 (;=-45;)))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 8))
        (i32.const 8))
      (i64.load
        (i32.add
          (local.get 1)
          (i32.const 8))))
    (i64.store offset=8
      (local.get 3)
      (i64.load
        (local.get 1)))
    (call $oxitortoise_offset_distance_by_heading
      (i32.add
        (local.get 3)
        (i32.const 80))
      (local.get 0)
      (i32.add
        (local.get 3)
        (i32.const 8))
      (local.get 8)
      (f64.const 0x1p+0 (;=1;)))
    (local.set 8
      (f64.const 0x0p+0 (;=0;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (call $oxitortoise_is_nan
          (f64.load offset=80
            (local.get 3))))
      (i32.store offset=72
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=80
              (local.get 3)))))
      (i32.store offset=76
        (local.get 3)
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=88
              (local.get 3)))))
      (i64.store
        (local.get 3)
        (i64.load offset=72 align=4
          (local.get 3)))
      (local.set 1
        (call $oxitortoise_patch_at
          (local.get 0)
          (local.get 3)))
      (local.set 8
        (f64.load offset=64
          (i32.add
            (i32.load offset=344
              (local.get 0))
            (i32.mul
              (local.get 1)
              (i32.const 80))))))
    (local.set 8
      (local.get 8))
    (block ;; label = @1
      (block ;; label = @2
        (br_if 0 (;@2;)
          (f64.gt
            (local.get 7)
            (local.get 4)))
        (br_if 1 (;@1;)
          (i32.eqz
            (f64.gt
              (local.get 8)
              (local.get 4)))))
      (f64.store
        (local.get 2)
        (call $oxitortoise_normalize_heading
          (f64.add
            (f64.load
              (local.get 2))
            (select
              (f64.const 0x1.68p+5 (;=45;))
              (f64.const -0x1.68p+5 (;=-45;))
              (f64.gt
                (local.get 7)
                (local.get 8)))))))
    (global.set $__stack_pointer
      (i32.add
        (local.get 3)
        (i32.const 96)))
  )
  (func $setup_body0 (;23;) (type 15) (param i32 i32 i64)
    (local i32 i32)
    (i64.store offset=40
      (local.tee 4
        (i32.add
          (i32.load offset=16
            (i32.load
              (local.get 1)))
          (i32.mul
            (local.tee 3
              (i32.wrap_i64
                (local.get 2)))
            (i32.const 112))))
      (i64.const 4624633867356078080))
    (i64.store offset=80
      (local.get 4)
      (i64.const 4611686018427387904))
    (i32.store16
      (local.tee 1
        (i32.add
          (i32.load offset=52
            (local.get 1))
          (i32.shl
            (local.get 3)
            (i32.const 1))))
      (i32.or
        (i32.load16_u
          (local.get 1))
        (i32.const 514)))
  )
  (func $setup_body1 (;24;) (type 14) (param i32 i32 i32)
    (local i32 i32 i32 i32 i64 f64 f64 i32)
    (global.set $__stack_pointer
      (local.tee 3
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 208))))
    (i64.store
      (local.tee 4
        (i32.add
          (i32.add
            (local.get 3)
            (i32.const 192))
          (i32.const 8)))
      (i64.load
        (local.tee 6
          (i32.add
            (local.tee 5
              (i32.add
                (i32.load offset=344
                  (i32.load
                    (local.get 1)))
                (i32.mul
                  (local.get 2)
                  (i32.const 80))))
            (i32.const 16)))))
    (local.set 7
      (i64.load offset=8
        (local.get 5)))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 176))
        (i32.const 8))
      (i64.const 0))
    (i64.store offset=192
      (local.get 3)
      (local.get 7))
    (i64.store offset=176
      (local.get 3)
      (i64.const 0))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 112))
        (i32.const 8))
      (i64.load
        (local.get 6)))
    (local.set 7
      (i64.load offset=8
        (local.get 5)))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 96))
        (i32.const 8))
      (i64.const 0))
    (i64.store offset=112
      (local.get 3)
      (local.get 7))
    (i64.store offset=96
      (local.get 3)
      (i64.const 0))
    (f64.store offset=64
      (local.get 5)
      (f64.sub
        (f64.const 0x1.9p+7 (;=200;))
        (local.tee 8
          (call $oxitortoise_distance_euclidean_no_wrap
            (i32.add
              (local.get 3)
              (i32.const 112))
            (i32.add
              (local.get 3)
              (i32.const 96))))))
    (i32.store8 offset=56
      (local.get 5)
      (f64.lt
        (local.get 8)
        (f64.const 0x1.4p+2 (;=5;))))
    (local.set 9
      (f64.load offset=552
        (local.tee 6
          (i32.load
            (local.get 1)))))
    (local.set 8
      (f64.load offset=528
        (local.get 6)))
    (i64.store
      (local.tee 6
        (i32.add
          (i32.add
            (local.get 3)
            (i32.const 160))
          (i32.const 8)))
      (i64.const 0))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 80))
        (i32.const 8))
      (i64.load
        (local.get 4)))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 64))
        (i32.const 8))
      (i64.load
        (local.get 6)))
    (f64.store offset=160
      (local.get 3)
      (f64.mul
        (local.get 8)
        (f64.const 0x1.3333333333333p-1 (;=0.6;))))
    (i64.store offset=80
      (local.get 3)
      (i64.load offset=192
        (local.get 3)))
    (i64.store offset=64
      (local.get 3)
      (i64.load offset=160
        (local.get 3)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eqz
          (f64.lt
            (call $oxitortoise_distance_euclidean_no_wrap
              (i32.add
                (local.get 3)
                (i32.const 80))
              (i32.add
                (local.get 3)
                (i32.const 64)))
            (f64.const 0x1.4p+2 (;=5;)))))
      (i64.store offset=72
        (local.get 5)
        (i64.const 4607182418800017408)))
    (f64.store
      (local.tee 6
        (i32.add
          (i32.add
            (local.get 3)
            (i32.const 144))
          (i32.const 8)))
      (f64.mul
        (local.get 9)
        (f64.const -0x1.3333333333333p-1 (;=-0.6;))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 48))
        (i32.const 8))
      (i64.load
        (local.get 4)))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 32))
        (i32.const 8))
      (i64.load
        (local.get 6)))
    (f64.store offset=144
      (local.get 3)
      (f64.mul
        (local.get 8)
        (f64.const -0x1.3333333333333p-1 (;=-0.6;))))
    (i64.store offset=48
      (local.get 3)
      (i64.load offset=192
        (local.get 3)))
    (i64.store offset=32
      (local.get 3)
      (i64.load offset=144
        (local.get 3)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eqz
          (f64.lt
            (call $oxitortoise_distance_euclidean_no_wrap
              (i32.add
                (local.get 3)
                (i32.const 48))
              (i32.add
                (local.get 3)
                (i32.const 32)))
            (f64.const 0x1.4p+2 (;=5;)))))
      (i64.store offset=72
        (local.get 5)
        (i64.const 4611686018427387904)))
    (f64.store
      (local.tee 4
        (i32.add
          (i32.add
            (local.get 3)
            (i32.const 128))
          (i32.const 8)))
      (f64.mul
        (local.get 9)
        (f64.const 0x1.999999999999ap-1 (;=0.8;))))
    (i64.store
      (i32.add
        (i32.add
          (local.get 3)
          (i32.const 16))
        (i32.const 8))
      (i64.load
        (i32.add
          (i32.add
            (local.get 3)
            (i32.const 192))
          (i32.const 8))))
    (i64.store
      (i32.add
        (local.get 3)
        (i32.const 8))
      (i64.load
        (local.get 4)))
    (f64.store offset=128
      (local.get 3)
      (f64.mul
        (local.get 8)
        (f64.const -0x1.999999999999ap-1 (;=-0.8;))))
    (i64.store offset=16
      (local.get 3)
      (i64.load offset=192
        (local.get 3)))
    (i64.store
      (local.get 3)
      (i64.load offset=128
        (local.get 3)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eqz
          (f64.lt
            (call $oxitortoise_distance_euclidean_no_wrap
              (i32.add
                (local.get 3)
                (i32.const 16))
              (local.get 3))
            (f64.const 0x1.4p+2 (;=5;)))))
      (i64.store offset=72
        (local.get 5)
        (i64.const 4613937818241073152)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eqz
          (f64.gt
            (f64.load offset=72
              (local.get 5))
            (f64.const 0x0p+0 (;=0;)))))
      (f64.store offset=48
        (local.get 5)
        (select
          (f64.const 0x1p+1 (;=2;))
          (f64.const 0x1p+0 (;=1;))
          (call $oxitortoise_next_int
            (local.get 1)
            (i32.const 2)))))
    (local.set 4
      (i32.shl
        (local.get 2)
        (i32.const 3)))
    (local.set 6
      (i32.load offset=380
        (local.tee 5
          (i32.load
            (local.get 1)))))
    (block ;; label = @1
      (block ;; label = @2
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (i32.load8_u offset=56
                (local.tee 10
                  (i32.add
                    (i32.load offset=344
                      (local.get 5))
                    (i32.mul
                      (local.get 2)
                      (i32.const 80)))))))
          (local.set 8
            (f64.const 0x1.ccp+6 (;=115;)))
          (br 1 (;@2;)))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (f64.gt
                (f64.load offset=48
                  (local.get 10))
                (f64.const 0x0p+0 (;=0;)))))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (f64.ne
                (local.tee 8
                  (f64.load offset=72
                    (local.get 10)))
                (f64.const 0x1p+0 (;=1;))))
            (local.set 8
              (f64.const 0x1.54p+6 (;=85;)))
            (br 2 (;@2;)))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (f64.ne
                (local.get 8)
                (f64.const 0x1p+1 (;=2;))))
            (local.set 8
              (f64.const 0x1.7cp+6 (;=95;)))
            (br 2 (;@2;)))
          (br_if 2 (;@1;)
            (f64.ne
              (local.get 8)
              (f64.const 0x1.8p+1 (;=3;))))
          (local.set 8
            (f64.const 0x1.a4p+6 (;=105;)))
          (br 1 (;@2;)))
        (local.set 8
          (call $oxitortoise_scale_color
            (f64.const 0x1.04p+6 (;=65;))
            (f64.load
              (i32.add
                (i32.load offset=416
                  (local.get 5))
                (local.get 4)))
            (f64.const 0x1.999999999999ap-4 (;=0.1;))
            (f64.const 0x1.4p+2 (;=5;)))))
      (f64.store
        (i32.add
          (local.get 6)
          (local.get 4))
        (local.get 8)))
    (i32.store8
      (local.tee 5
        (i32.add
          (i32.load offset=56
            (local.get 1))
          (local.get 2)))
      (i32.or
        (i32.load8_u
          (local.get 5))
        (i32.const 1)))
    (global.set $__stack_pointer
      (i32.add
        (local.get 3)
        (i32.const 208)))
  )
  (func $setup (;25;) (type 6) (param i32)
    (local i32 i32 i32 i64)
    (global.set $__stack_pointer
      (local.tee 1
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 64))))
    (local.set 2
      (i32.load
        (local.get 0)))
    (call $oxitortoise_clear_all
      (local.get 0))
    (i32.store offset=60
      (local.get 1)
      (local.tee 3
        (i32.add
          (local.get 0)
          (i32.const 64))))
    (i32.store offset=56
      (local.get 1)
      (global.get $setup_body0))
    (local.set 4
      (call $oxitortoise_get_default_turtle_breed
        (local.get 0)))
    (i64.store
      (i32.add
        (i32.add
          (local.get 1)
          (i32.const 40))
        (i32.const 8))
      (i64.const 0))
    (i64.store
      (i32.add
        (i32.add
          (local.get 1)
          (i32.const 24))
        (i32.const 8))
      (i64.const 0))
    (i64.store offset=40
      (local.get 1)
      (i64.const 0))
    (i64.store offset=24
      (local.get 1)
      (i64.const 0))
    (i64.store offset=16
      (local.get 1)
      (i64.load offset=56 align=4
        (local.get 1)))
    (call $oxitortoise_create_turtles
      (local.get 0)
      (local.get 4)
      (i64.const 125)
      (i32.add
        (local.get 1)
        (i32.const 24))
      (i32.add
        (local.get 1)
        (i32.const 16)))
    (i32.store offset=60
      (local.get 1)
      (local.get 3))
    (i32.store offset=56
      (local.get 1)
      (global.get $setup_body1))
    (i64.store offset=8
      (local.get 1)
      (i64.load offset=56 align=4
        (local.get 1)))
    (call $oxitortoise_for_all_patches
      (local.get 0)
      (i32.add
        (local.get 1)
        (i32.const 8)))
    (call $oxitortoise_reset_ticks
      (local.get 2))
    (f64.store offset=8
      (local.get 0)
      (call $oxitortoise_get_ticks
        (local.get 2)))
    (global.set $__stack_pointer
      (i32.add
        (local.get 1)
        (i32.const 64)))
  )
  (func $shim_setup (;26;) (type 9) (param i32 i32)
    (call $setup
      (local.get 0))
  )
  (func $go_body0 (;27;) (type 15) (param i32 i32 i64)
    (local i32 i32 i32 i32 i32 i32 f64 i32 i32 f64)
    (global.set $__stack_pointer
      (local.tee 3
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 112))))
    (local.set 2
      (i64.load offset=8
        (local.tee 6
          (i32.add
            (i32.load offset=16
              (local.tee 4
                (i32.load
                  (local.get 1))))
            (i32.mul
              (local.tee 5
                (i32.wrap_i64
                  (local.get 2)))
              (i32.const 112))))))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (f64.le
          (call $oxitortoise_get_ticks
            (local.get 4))
          (f64.convert_i64_u
            (local.get 2))))
      (local.set 7
        (i32.add
          (local.get 6)
          (i32.const 88)))
      (local.set 8
        (i32.add
          (local.get 6)
          (i32.const 96)))
      (local.set 9
        (f64.load offset=40
          (local.get 6)))
      (local.set 10
        (i32.trunc_sat_f64_u
          (call $oxitortoise_round
            (f64.load offset=96
              (local.get 6)))))
      (block ;; label = @2
        (block ;; label = @3
          (br_if 0 (;@3;)
            (f64.ne
              (local.get 9)
              (f64.const 0x1.ep+3 (;=15;))))
          (i32.store offset=104
            (local.get 3)
            (local.get 10))
          (i32.store offset=108
            (local.get 3)
            (i32.trunc_sat_f64_u
              (call $oxitortoise_round
                (f64.load offset=104
                  (local.get 6)))))
          (i64.store offset=48
            (local.get 3)
            (i64.load offset=104 align=4
              (local.get 3)))
          (local.set 10
            (call $oxitortoise_patch_at
              (local.get 4)
              (i32.add
                (local.get 3)
                (i32.const 48))))
          (block ;; label = @4
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (f64.gt
                    (local.tee 9
                      (f64.load offset=48
                        (local.tee 11
                          (i32.add
                            (i32.load offset=344
                              (local.get 4))
                            (i32.mul
                              (local.get 10)
                              (i32.const 80))))))
                    (f64.const 0x0p+0 (;=0;)))))
              (i64.store offset=40
                (local.get 6)
                (i64.const 4628011567076605952))
              (f64.store
                (i32.add
                  (local.get 11)
                  (i32.const 48))
                (f64.add
                  (local.get 9)
                  (f64.const -0x1p+0 (;=-1;))))
              (f64.store offset=88
                (local.get 6)
                (call $oxitortoise_normalize_heading
                  (f64.add
                    (f64.load offset=88
                      (local.get 6))
                    (f64.const 0x1.68p+7 (;=180;)))))
              (i32.store16
                (local.tee 6
                  (i32.add
                    (i32.load offset=52
                      (local.get 1))
                    (i32.shl
                      (local.get 5)
                      (i32.const 1))))
                (i32.or
                  (i32.load16_u
                    (local.get 6))
                  (i32.const 1030)))
              (br 1 (;@4;)))
            (br_if 0 (;@4;)
              (i32.eqz
                (f64.ge
                  (local.tee 12
                    (f64.load
                      (i32.add
                        (i32.load offset=416
                          (local.get 4))
                        (i32.shl
                          (local.get 10)
                          (i32.const 3)))))
                  (f64.const 0x1.999999999999ap-5 (;=0.05;)))))
            (br_if 0 (;@4;)
              (i32.eqz
                (f64.lt
                  (local.get 12)
                  (f64.const 0x1p+1 (;=2;)))))
            (i64.store
              (i32.add
                (i32.add
                  (local.get 3)
                  (i32.const 32))
                (i32.const 8))
              (i64.load
                (i32.add
                  (local.get 8)
                  (i32.const 8))))
            (i64.store offset=32
              (local.get 3)
              (i64.load
                (local.get 8)))
            (call $uphill_chemical
              (local.get 4)
              (i32.add
                (local.get 3)
                (i32.const 32))
              (local.get 7)))
          (br_if 1 (;@2;)
            (i32.eqz
              (f64.gt
                (local.get 9)
                (f64.const 0x0p+0 (;=0;)))))
          (br 2 (;@1;)))
        (i32.store offset=96
          (local.get 3)
          (local.get 10))
        (i32.store offset=100
          (local.get 3)
          (i32.trunc_sat_f64_u
            (call $oxitortoise_round
              (f64.load offset=104
                (local.get 6)))))
        (i64.store offset=72
          (local.get 3)
          (i64.load offset=96 align=4
            (local.get 3)))
        (local.set 10
          (call $oxitortoise_patch_at
            (local.get 4)
            (i32.add
              (local.get 3)
              (i32.const 72))))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.ne
              (i32.load8_u offset=56
                (i32.add
                  (i32.load offset=344
                    (local.get 4))
                  (i32.mul
                    (local.get 10)
                    (i32.const 80))))
              (i32.const 1)))
          (i64.store offset=40
            (local.get 6)
            (i64.const 4624633867356078080))
          (f64.store offset=88
            (local.get 6)
            (call $oxitortoise_normalize_heading
              (f64.add
                (f64.load offset=88
                  (local.get 6))
                (f64.const 0x1.68p+7 (;=180;)))))
          (br 1 (;@2;)))
        (f64.store
          (local.tee 6
            (i32.add
              (i32.load offset=416
                (local.get 4))
              (i32.shl
                (local.get 10)
                (i32.const 3))))
          (f64.add
            (f64.load
              (local.get 6))
            (f64.const 0x1.ep+5 (;=60;))))
        (i64.store
          (i32.add
            (i32.add
              (local.get 3)
              (i32.const 56))
            (i32.const 8))
          (i64.load
            (i32.add
              (local.get 8)
              (i32.const 8))))
        (i64.store offset=56
          (local.get 3)
          (i64.load
            (local.get 8)))
        (call $uphill_nest_scent
          (local.get 4)
          (i32.add
            (local.get 3)
            (i32.const 56))
          (local.get 7)))
      (local.set 6
        (call $oxitortoise_next_int
          (local.get 1)
          (i32.const 40)))
      (f64.store
        (local.get 7)
        (call $oxitortoise_normalize_heading
          (f64.add
            (f64.load
              (local.get 7))
            (f64.convert_i32_u
              (local.get 6)))))
      (local.set 6
        (call $oxitortoise_next_int
          (local.get 1)
          (i32.const 40)))
      (f64.store
        (local.get 7)
        (call $oxitortoise_normalize_heading
          (f64.sub
            (f64.load
              (local.get 7))
            (f64.convert_i32_u
              (local.get 6)))))
      (local.set 9
        (f64.load
          (local.get 7)))
      (i64.store
        (i32.add
          (i32.add
            (local.get 3)
            (i32.const 16))
          (i32.const 8))
        (i64.load
          (local.tee 6
            (i32.add
              (local.get 8)
              (i32.const 8)))))
      (i64.store offset=16
        (local.get 3)
        (i64.load
          (local.get 8)))
      (call $oxitortoise_offset_distance_by_heading
        (i32.add
          (local.get 3)
          (i32.const 80))
        (local.get 4)
        (i32.add
          (local.get 3)
          (i32.const 16))
        (local.get 9)
        (f64.const 0x1p+0 (;=1;)))
      (block ;; label = @2
        (br_if 0 (;@2;)
          (i32.eqz
            (call $oxitortoise_is_nan
              (f64.load offset=80
                (local.get 3)))))
        (f64.store
          (local.get 7)
          (call $oxitortoise_normalize_heading
            (f64.add
              (f64.load
                (local.get 7))
              (f64.const 0x1.68p+7 (;=180;))))))
      (local.set 9
        (f64.load
          (local.get 7)))
      (i64.store
        (i32.add
          (local.get 3)
          (i32.const 8))
        (i64.load
          (local.get 6)))
      (i64.store
        (local.get 3)
        (i64.load
          (local.get 8)))
      (call $oxitortoise_offset_distance_by_heading
        (i32.add
          (local.get 3)
          (i32.const 80))
        (local.get 4)
        (local.get 3)
        (local.get 9)
        (f64.const 0x1p+0 (;=1;)))
      (block ;; label = @2
        (br_if 0 (;@2;)
          (call $oxitortoise_is_nan
            (f64.load offset=80
              (local.get 3))))
        (i64.store
          (local.get 8)
          (i64.load offset=80
            (local.get 3)))
        (i64.store
          (i32.add
            (local.get 8)
            (i32.const 8))
          (i64.load
            (i32.add
              (i32.add
                (local.get 3)
                (i32.const 80))
              (i32.const 8)))))
      (i32.store16
        (local.tee 6
          (i32.add
            (i32.load offset=52
              (local.get 1))
            (i32.shl
              (local.get 5)
              (i32.const 1))))
        (i32.or
          (i32.load16_u
            (local.get 6))
          (i32.const 1030))))
    (global.set $__stack_pointer
      (i32.add
        (local.get 3)
        (i32.const 112)))
  )
  (func $go_body1 (;28;) (type 14) (param i32 i32 i32)
    (local i32 i32 i32 i32 f64)
    (f64.store
      (local.tee 4
        (i32.add
          (i32.load offset=416
            (i32.load
              (local.get 1)))
          (local.tee 3
            (i32.shl
              (local.get 2)
              (i32.const 3)))))
      (f64.mul
        (f64.load
          (local.get 4))
        (f64.const 0x1.ccccccccccccdp-1 (;=0.9;))))
    (local.set 5
      (i32.load offset=380
        (local.tee 4
          (i32.load
            (local.get 1)))))
    (block ;; label = @1
      (block ;; label = @2
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (i32.load8_u offset=56
                (local.tee 6
                  (i32.add
                    (i32.load offset=344
                      (local.get 4))
                    (i32.mul
                      (local.get 2)
                      (i32.const 80)))))))
          (local.set 7
            (f64.const 0x1.ccp+6 (;=115;)))
          (br 1 (;@2;)))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (f64.gt
                (f64.load offset=48
                  (local.get 6))
                (f64.const 0x0p+0 (;=0;)))))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (f64.ne
                (local.tee 7
                  (f64.load offset=72
                    (local.get 6)))
                (f64.const 0x1p+0 (;=1;))))
            (local.set 7
              (f64.const 0x1.54p+6 (;=85;)))
            (br 2 (;@2;)))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (f64.ne
                (local.get 7)
                (f64.const 0x1p+1 (;=2;))))
            (local.set 7
              (f64.const 0x1.7cp+6 (;=95;)))
            (br 2 (;@2;)))
          (br_if 2 (;@1;)
            (f64.ne
              (local.get 7)
              (f64.const 0x1.8p+1 (;=3;))))
          (local.set 7
            (f64.const 0x1.a4p+6 (;=105;)))
          (br 1 (;@2;)))
        (local.set 7
          (call $oxitortoise_scale_color
            (f64.const 0x1.04p+6 (;=65;))
            (f64.load
              (i32.add
                (i32.load offset=416
                  (local.get 4))
                (local.get 3)))
            (f64.const 0x1.999999999999ap-4 (;=0.1;))
            (f64.const 0x1.4p+2 (;=5;)))))
      (f64.store
        (i32.add
          (local.get 5)
          (local.get 3))
        (local.get 7)))
    (i32.store8
      (local.tee 2
        (i32.add
          (i32.load offset=56
            (local.get 1))
          (local.get 2)))
      (i32.or
        (i32.load8_u
          (local.get 2))
        (i32.const 1)))
  )
  (func $go (;29;) (type 6) (param i32)
    (local i32 i32 i32)
    (global.set $__stack_pointer
      (local.tee 1
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 48))))
    (local.set 2
      (i32.load
        (local.get 0)))
    (i32.store offset=44
      (local.get 1)
      (local.tee 3
        (i32.add
          (local.get 0)
          (i32.const 64))))
    (i32.store offset=40
      (local.get 1)
      (global.get $go_body0))
    (i64.store offset=24
      (local.get 1)
      (i64.load offset=40 align=4
        (local.get 1)))
    (call $oxitortoise_for_all_turtles
      (local.get 0)
      (i32.add
        (local.get 1)
        (i32.const 24)))
    (i32.store16 offset=38 align=1
      (local.get 1)
      (i32.const 2))
    (i32.store16 offset=22
      (local.get 1)
      (i32.const 2))
    (call $oxitortoise_diffuse_8
      (local.get 2)
      (i32.add
        (local.get 1)
        (i32.const 22))
      (f64.const 0x1p-1 (;=0.5;)))
    (i32.store offset=44
      (local.get 1)
      (local.get 3))
    (i32.store offset=40
      (local.get 1)
      (global.get $go_body1))
    (i64.store offset=8
      (local.get 1)
      (i64.load offset=40 align=4
        (local.get 1)))
    (call $oxitortoise_for_all_patches
      (local.get 0)
      (i32.add
        (local.get 1)
        (i32.const 8)))
    (call $oxitortoise_advance_tick
      (local.get 2))
    (f64.store offset=8
      (local.get 0)
      (call $oxitortoise_get_ticks
        (local.get 2)))
    (global.set $__stack_pointer
      (i32.add
        (local.get 1)
        (i32.const 48)))
  )
  (func $shim_go (;30;) (type 9) (param i32 i32)
    (call $go
      (local.get 0))
  )
  (@custom ".debug_loc" (after code) "\ff\ff\ff\ff\06\00\00\00\00\00\00\00\04\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\11\00\00\00\13\00\00\00\04\00\ed\02\00\9f\13\00\00\00\04\01\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\c5\00\00\00\c8\00\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\e8\00\00\00\ed\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00,\00\00\00.\00\00\00\04\00\ed\02\00\9f.\00\00\00\04\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\00\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00!\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\00\00\00\00\cf\00\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\af\00\00\00\c0\00\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\bb\00\00\00\be\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00 \00\00\00*\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\002\00\00\00\89\00\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\00\00\00\00\9f\02\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00Q\00\00\00E\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\c6\00\00\00\d7\00\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\d2\00\00\00\d5\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\dc\00\00\00\9f\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\e1\00\00\00\eb\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\f3\00\00\00E\01\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\82\01\00\00\93\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\8e\01\00\00\91\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\98\01\00\00\9f\02\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\9d\01\00\00\a7\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\af\01\00\00\04\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00>\02\00\00O\02\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00J\02\00\00M\02\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00T\02\00\00\9f\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00\00\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00!\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00\00\00\00\00\d0\00\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00\af\00\00\00\c1\00\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00 \00\00\00*\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\002\00\00\00\89\00\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\00\00\00\00\a2\02\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00Q\00\00\00F\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\c6\00\00\00\d8\00\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\dd\00\00\00\a2\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\e2\00\00\00\ec\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\f4\00\00\00F\01\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\83\01\00\00\95\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\9a\01\00\00\a2\02\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\9f\01\00\00\a9\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\b1\01\00\00\06\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00@\02\00\00R\02\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00W\02\00\00\a2\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\006\00\00\008\00\00\00\04\00\ed\02\01\9f8\00\00\00\fb\02\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\b9\00\00\00\bb\00\00\00\04\00\ed\02\02\9f\bb\00\00\00`\01\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\dd\00\00\00\d7\03\00\00\04\00\ed\00\09\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\e5\00\00\002\03\00\00\04\00\ed\00\08\9fy\03\00\00\a9\03\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00S\01\00\00]\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\e4\01\00\00\ee\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00w\02\00\00\81\02\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\c7\02\00\00\c8\02\00\00\04\00\ed\02\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\d8\02\00\00\da\02\00\00\04\00\ed\02\00\9f\da\02\00\00\d7\03\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\8c\03\00\00\8f\03\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\af\03\00\00\b4\03\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffD\08\00\00\f3\02\00\00\f5\02\00\00\04\00\ed\02\00\9f\f5\02\00\00\d7\03\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\1d\0c\00\00\1e\00\00\00\dc\00\00\00\04\00\ed\00\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00\00\00\00\00F\00\00\00\04\00\ed\00\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00 \00\00\00\22\00\00\00\04\00\ed\02\00\9f\22\00\00\00\9c\03\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00/\00\00\001\00\00\00\04\00\ed\02\00\9f1\00\00\00\88\01\00\00\04\00\ed\00\06\9f\9a\01\00\00^\02\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00N\00\00\00\8e\03\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00V\00\00\00\8e\03\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00\b0\00\00\00\9a\01\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00;\01\00\00>\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00>\01\00\00@\01\00\00\04\00\ed\02\00\9f@\01\00\00\88\01\00\00\04\00\ed\00\0c\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00\ce\01\00\00^\02\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00\1e\02\00\00 \02\00\00\04\00\ed\02\00\9f \02\00\00^\02\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\0d\00\00u\02\00\00v\02\00\00\04\00\ed\02\02\9f\95\02\00\00\96\02\00\00\04\00\ed\02\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\a4\10\00\00\0a\00\00\00\0e\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\a4\10\00\00\16\00\00\00\18\00\00\00\04\00\ed\02\00\9f\18\00\00\00R\00\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\a4\10\00\00/\00\00\001\00\00\00\04\00\ed\02\00\9f1\00\00\00\22\01\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\a4\10\00\00\e3\00\00\00\e6\00\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\a4\10\00\00\06\01\00\00\0b\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\a4\10\00\00J\00\00\00L\00\00\00\04\00\ed\02\00\9fL\00\00\00\22\01\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\c8\11\00\00\1b\00\00\00\b8\00\00\00\04\00\ed\00\02\9f\00\00\00\00\00\00\00\00")
  (@custom ".debug_abbrev" (after code) "\01\11\01%\0e\13\05\03\0e\10\17\1b\0e\11\01U\17\00\00\02\0f\00I\13\00\00\03\16\00I\13\03\0e:\0b;\0b\00\00\04\13\01\0b\0b:\0b;\0b\00\00\05\0d\00\03\0eI\13:\0b;\0b8\0b\00\00\06\01\01I\13\00\00\07!\00I\137\0b\00\00\08$\00\03\0e>\0b\0b\0b\00\00\09$\00\03\0e\0b\0b>\0b\00\00\0a.\01\11\01\12\06@\18\97B\191\13\00\00\0b\05\00\02\171\13\00\00\0c\05\00\02\181\13\00\00\0d4\00\02\171\13\00\00\0e\89\82\01\001\13\11\01\00\00\0f.\01\03\0e:\0b;\0b'\19I\13<\19?\19\00\00\10\05\00I\13\00\00\11\05\001\13\00\00\124\00\02\181\13\00\00\13\17\01\0b\05:\0b;\0b\00\00\14\13\01\0b\05:\0b;\0b\00\00\15\0d\00\03\0eI\13:\0b;\0b8\05\00\00\16\0f\00\00\00\17!\00I\137\05\00\00\18\17\01\0b\0b:\0b;\0b\00\00\19.\01\03\0e:\0b;\0b'\19I\13?\19 \0b\00\00\1a\05\00\03\0e:\0b;\0bI\13\00\00\1b4\00\03\0e:\0b;\0bI\13\00\00\1c.\01\11\01\12\06@\18\97B\19\03\0e:\0b;\0b'\19?\19\00\00\1d\05\00\02\17\03\0e:\0b;\0bI\13\00\00\1e\05\00\02\18\03\0e:\0b;\0bI\13\00\00\1f4\00\02\17\03\0e:\0b;\0bI\13\00\00 \1d\011\13\11\01\12\06X\0bY\0bW\0b\00\00!\05\00\1c\0f1\13\00\00\224\001\13\00\00#4\00\02\17\03\0e:\0b;\05I\13\00\00$\1d\011\13\11\01\12\06X\0bY\05W\0b\00\00%.\01\11\01\12\06@\18\97B\19\03\0e:\0b;\05'\19?\19\00\00&\05\00\03\0e:\0b;\05I\13\00\00'\05\00\02\18\03\0e:\0b;\05I\13\00\00(4\00\03\0e:\0b;\05I\13\00\00).\01\03\0e:\0b;\0b'\19?\19 \0b\00\00*4\00\02\18\03\0e:\0b;\05I\13\00\00+\0b\01\11\01\12\06\00\00,.\01\03\0e:\0b;\0b'\19<\19?\19\00\00-\15\01'\19\00\00.\05\00\02\17\03\0e:\0b;\05I\13\00\00\00")
  (@custom ".debug_info" (after code) "\88\14\00\00\04\00\00\00\00\00\04\01\91\06\00\00\1d\00\af\05\00\00\00\00\00\000\01\00\00\00\00\00\00\00\00\00\00\02+\00\00\00\036\00\00\00i\06\00\00\02\8b\04P\02\84\05\0b\05\00\00\83\00\00\00\02\85\00\05\bc\05\00\00\a8\00\00\00\02\86\08\05\e8\04\00\00\04\01\00\00\02\870\05K\00\00\004\01\00\00\02\888\05\9e\00\00\00\04\01\00\00\02\89@\05\ee\02\00\00\04\01\00\00\02\8aH\00\06\8f\00\00\00\07\a1\00\00\00\01\00\03\9a\00\00\00\0d\01\00\00\01\ca\08\01\03\00\00\08\01\09\ef\05\00\00\08\07\03\b3\00\00\00\d2\05\00\00\027\04(\023\05[\03\00\00\dc\00\00\00\024\00\05\9a\03\00\00\16\01\00\00\025\10\05\8f\02\00\00\04\01\00\00\026 \00\03\e7\00\00\00_\00\00\00\02\18\04\10\02\15\055\00\00\00\04\01\00\00\02\16\00\05\15\00\00\00\04\01\00\00\02\17\08\00\03\0f\01\00\00\f2\00\00\00\02\12\08\bf\04\00\00\04\08\03!\01\00\00\09\04\00\00\02\13\06-\01\00\00\07\a1\00\00\00\0c\00\08\0a\03\00\00\06\01\08~\03\00\00\02\01\02@\01\00\00\03K\01\00\00A\06\00\00\02\8e\04\08\02\8c\05\88\02\00\00\04\01\00\00\02\8d\00\00\02a\01\00\00\03l\01\00\00\19\06\00\00\02\91\04\08\02\8f\05\a8\03\00\00\04\01\00\00\02\90\00\00\03\88\01\00\00'\01\00\00\01\d4\08\8a\00\00\00\07\04\02\94\01\00\00\03\9f\01\00\00u\06\00\00\02\83\04p\02~\05\0b\05\00\00\83\00\00\00\02\7f\00\05\bc\05\00\00\d4\01\00\00\02\80\08\05^\04\00\00\04\01\00\00\02\81X\05[\03\00\00\dc\00\00\00\02\82`\00\03\df\01\00\00\e0\05\00\00\021\04P\02(\05A\03\00\00D\02\00\00\02)\00\05T\05\00\00a\02\00\00\02*\08\05\82\04\00\00\16\01\00\00\02+\10\05\ae\02\00\00\04\01\00\00\02, \05\9b\03\00\00\16\01\00\00\02-(\05\90\02\00\00\04\01\00\00\02.8\05d\03\00\004\01\00\00\02/@\05f\04\00\00\04\01\00\00\020H\00\03O\02\00\00E\03\00\00\02\10\03Z\02\00\00\1e\01\00\00\01\d9\08\f6\03\00\00\07\08\03O\02\00\00\a7\05\00\00\02\11\0a\06\00\00\00\04\01\00\00\07\ed\03\00\00\00\00\9fx\0b\00\00\0b\00\00\00\00\80\0b\00\00\0c\04\ed\00\01\9f\8b\0b\00\00\0d\1e\00\00\00\96\0b\00\00\0dJ\00\00\00\a1\0b\00\00\0dh\00\00\00\ac\0b\00\00\0d\86\00\00\00\b7\0b\00\00\0e\c2\02\00\00\e6\00\00\00\00\0f\9c\02\00\00\02x\04\01\00\00\10\04\01\00\00\10\04\01\00\00\10\04\01\00\00\10\04\01\00\00\00\0a\0c\01\00\00\cf\00\00\00\04\ed\00\04\9f\c7\05\00\00\0b\ee\00\00\00\d3\05\00\00\11\de\05\00\00\0c\04\ed\00\02\9f\e9\05\00\00\0b\b2\00\00\00\f4\05\00\00\12\02\91 \ff\05\00\00\12\02\91\18+\06\00\00\0d\d0\00\00\00\0a\06\00\00\0d\0c\01\00\00\15\06\00\00\0d*\01\00\00 \06\00\00\0ew\03\00\00+\01\00\00\0e\88\03\00\00g\01\00\00\0eJ\05\00\00\7f\01\00\00\0e[\05\00\00\8e\01\00\00\0e[\05\00\00\a0\01\00\00\0el\05\00\00\b9\01\00\00\00\0fH\04\00\00\02t\04\01\00\00\10\04\01\00\00\00\0f\14\04\00\00\02r\dc\00\00\00\10\a8\03\00\00\10\dc\00\00\00\10\04\01\00\00\10\04\01\00\00\00\02\ad\03\00\00\03\b8\03\00\00\05\05\00\00\02J\13X\02\02E\05\8e\01\00\00\c9\03\00\00\02F\00\04\a0\02F\05\89\01\00\00h\04\00\00\02F\00\059\00\00\00t\04\00\00\02F\10\00\05{\01\00\00\f2\03\00\00\02G\00\14\e8\01\02G\05v\01\00\00\b5\04\00\00\02G\00\159\00\00\00t\04\00\00\02GX\01\00\05\05\00\00\00\1d\04\00\00\02H\00\140\02\02H\05\00\00\00\00\c2\04\00\00\02H\00\159\00\00\00\cf\04\00\00\02H\08\02\00\05\d7\02\00\00H\04\00\00\02I\00\14X\02\02I\05\d2\02\00\00=\05\00\00\02I\00\159\00\00\00\04\01\00\00\02IP\02\00\00\06-\01\00\00\07\a1\00\00\00\10\00\06\80\04\00\00\07\a1\00\00\00\04\00\03\8b\04\00\00\e4\02\00\00\02>\04$\02;\05\cd\05\00\00\a8\04\00\00\02<\00\05Z\05\00\00\a9\04\00\00\02=\04\00\16\06-\01\00\00\07\a1\00\00\00 \00\06-\01\00\00\17\a1\00\00\00X\01\00\06-\01\00\00\17\a1\00\00\00\08\02\00\03\da\04\00\00\0e\00\00\00\02C\18(\02@\05\c8\02\00\00\ea\04\00\00\02A\00\04\10\02A\05\c3\02\00\001\05\00\00\02A\00\059\00\00\00\04\01\00\00\02A\08\00\05\b9\02\00\00\13\05\00\00\02B\00\04(\02B\05\b4\02\00\00\a9\04\00\00\02B\00\059\00\00\00\04\01\00\00\02B \00\00\06-\01\00\00\07\a1\00\00\00\08\00\06-\01\00\00\17\a1\00\00\00P\02\00\0fk\03\00\00\02e4\01\00\00\10\0f\01\00\00\00\0f\ed\04\00\00\02f\04\01\00\00\10\04\01\00\00\00\0f\f8\00\00\00\02s\82\05\00\00\10\a8\03\00\00\10\9f\05\00\00\00\03\8d\05\00\00\8e\05\00\00\02&\03\98\05\00\00(\01\00\00\01\bb\08\93\00\00\00\05\04\03\aa\05\00\00\a9\00\00\00\02\1d\04\08\02\1a\055\00\00\00}\01\00\00\02\1b\00\05\15\00\00\00}\01\00\00\02\1c\04\00\19\ad\04\00\00\02\d0\04\01\00\00\01\1a\ff\04\00\00\02\d0\a8\03\00\00\1a[\03\00\00\02\d0\dc\00\00\00\1a^\04\00\00\02\d0\04\01\00\00\1a\b9\04\00\00\02\d0\04\01\00\00\1b_\05\00\00\02\d2\dc\00\00\00\1b;\04\00\00\02\d1\04\01\00\00\1b\1e\05\00\00\02\d9\82\05\00\00\1b%\06\00\00\02\db\5c\01\00\00\1bz\00\00\00\02\d8\9f\05\00\00\00\1c\dd\01\00\00\9f\02\00\00\04\ed\00\03\9f\a1\03\00\00\02\df\1d\84\01\00\00\ff\04\00\00\02\df\a8\03\00\00\1a[\03\00\00\02\df\dc\00\00\00\1e\04\ed\00\02\9f^\04\00\00\02\dfw\14\00\00\1f\fc\01\00\00w\05\00\00\02\e0\04\01\00\00\1f\92\02\00\00\ca\00\00\00\02\e1\04\01\00\00\1f(\03\00\00\e4\00\00\00\02\e2\04\01\00\00 \c7\05\00\00\06\02\00\00\b3\00\00\00\02\e0\19\0b\a2\01\00\00\d3\05\00\00\0bH\01\00\00\e9\05\00\00!\00\f4\05\00\00\12\03\91\d0\00\ff\05\00\00\0df\01\00\00\0a\06\00\00\0d\c0\01\00\00\15\06\00\00\0d\de\01\00\00 \06\00\00\00 \c7\05\00\00\c7\02\00\00\ae\00\00\00\02\e1\19\0b\1a\02\00\00\e9\05\00\00!\80\80\80\80\80\80\a0\a3@\f4\05\00\00\12\03\91\d0\00\ff\05\00\00\0d8\02\00\00\0a\06\00\00\0dV\02\00\00\15\06\00\00\0dt\02\00\00 \06\00\00\00 \c7\05\00\00\83\03\00\00\ae\00\00\00\02\e2\18\0b\b0\02\00\00\e9\05\00\00!\80\80\80\80\80\80\a0\a3\c0\01\f4\05\00\00\12\03\91\d0\00\ff\05\00\00\0d\ce\02\00\00\0a\06\00\00\0d\ec\02\00\00\15\06\00\00\0d\0a\03\00\00 \06\00\00\00\0ew\03\00\00\0d\02\00\00\0e\88\03\00\00L\02\00\00\0eJ\05\00\00d\02\00\00\0e[\05\00\00s\02\00\00\0e[\05\00\00\85\02\00\00\0el\05\00\00\a1\02\00\00\0ew\03\00\00\ce\02\00\00\0e\88\03\00\00\08\03\00\00\0eJ\05\00\00 \03\00\00\0e[\05\00\00/\03\00\00\0e[\05\00\00A\03\00\00\0el\05\00\00]\03\00\00\0ew\03\00\00\8a\03\00\00\0e\88\03\00\00\c7\03\00\00\0eJ\05\00\00\df\03\00\00\0e[\05\00\00\ee\03\00\00\0e[\05\00\00\00\04\00\00\0el\05\00\00\19\04\00\00\0ew\03\00\00k\04\00\00\00\0a~\04\00\00\d0\00\00\00\04\ed\00\04\9f\c5\08\00\00\0b\82\03\00\00\d1\08\00\00\11\dc\08\00\00\0c\04\ed\00\02\9f\e7\08\00\00\0bF\03\00\00\f2\08\00\00\12\02\91 \fd\08\00\00\12\02\91\18\1e\09\00\00\0dd\03\00\00\08\09\00\00\0d\a0\03\00\00\13\09\00\00\22)\09\00\00\0ew\03\00\00\9d\04\00\00\0e\88\03\00\00\d9\04\00\00\0eJ\05\00\00\f1\04\00\00\0e[\05\00\00\00\05\00\00\0e[\05\00\00\12\05\00\00\0el\05\00\00+\05\00\00\00\19\99\04\00\00\02\ee\04\01\00\00\01\1a\ff\04\00\00\02\ee\a8\03\00\00\1a[\03\00\00\02\ee\dc\00\00\00\1a^\04\00\00\02\ee\04\01\00\00\1a\b9\04\00\00\02\ee\04\01\00\00\1b_\05\00\00\02\f0\dc\00\00\00\1b;\04\00\00\02\ef\04\01\00\00\1b\1e\05\00\00\02\f7\82\05\00\00\1bz\00\00\00\02\f6\9f\05\00\00\1b\82\06\00\00\02\f9&\00\00\00\00\1cP\05\00\00\a2\02\00\00\04\ed\00\03\9f\97\00\00\00\02\fd\1d\fa\03\00\00\ff\04\00\00\02\fd\a8\03\00\00\1a[\03\00\00\02\fd\dc\00\00\00\1e\04\ed\00\02\9f^\04\00\00\02\fdw\14\00\00\1fT\04\00\00k\05\00\00\02\fe\04\01\00\00\1f\cc\04\00\00\be\00\00\00\02\ff\04\01\00\00#D\05\00\00\d9\00\00\00\02\00\01\04\01\00\00 \c5\08\00\00y\05\00\00\b4\00\00\00\02\fe\16\0b\18\04\00\00\d1\08\00\00\0b\be\03\00\00\e7\08\00\00!\00\f2\08\00\00\12\03\91\d0\00\fd\08\00\00\0d\dc\03\00\00\08\09\00\00\0d6\04\00\00\13\09\00\00\00 \c5\08\00\00;\06\00\00\af\00\00\00\02\ff\16\0br\04\00\00\e7\08\00\00!\80\80\80\80\80\80\a0\a3@\f2\08\00\00\12\03\91\d0\00\fd\08\00\00\0d\90\04\00\00\08\09\00\00\0d\ae\04\00\00\13\09\00\00\00$\c5\08\00\00\f8\06\00\00\af\00\00\00\02\00\01\15\0b\ea\04\00\00\e7\08\00\00!\80\80\80\80\80\80\a0\a3\c0\01\f2\08\00\00\12\03\91\d0\00\fd\08\00\00\0d\08\05\00\00\08\09\00\00\0d&\05\00\00\13\09\00\00\00\0ew\03\00\00\80\05\00\00\0e\88\03\00\00\bf\05\00\00\0eJ\05\00\00\d7\05\00\00\0e[\05\00\00\e6\05\00\00\0e[\05\00\00\f8\05\00\00\0el\05\00\00\14\06\00\00\0ew\03\00\00B\06\00\00\0e\88\03\00\00|\06\00\00\0eJ\05\00\00\94\06\00\00\0e[\05\00\00\a3\06\00\00\0e[\05\00\00\b5\06\00\00\0el\05\00\00\d1\06\00\00\0ew\03\00\00\ff\06\00\00\0e\88\03\00\00<\07\00\00\0eJ\05\00\00T\07\00\00\0e[\05\00\00c\07\00\00\0e[\05\00\00u\07\00\00\0el\05\00\00\8e\07\00\00\0ew\03\00\00\e1\07\00\00\00%\f3\07\00\00O\00\00\00\07\ed\03\00\00\00\00\9fT\06\00\00\02\0c\01&7\00\00\00\02\0c\01\a8\04\00\00'\04\ed\00\01\9f;\00\00\00\02\0c\01\c3\0b\00\00'\04\ed\00\02\9f\8d\04\00\00\02\0c\01\d1\0f\00\00(\c6\05\00\00\02\0d\01\8f\01\00\00(\bc\05\00\00\02\0e\01|\14\00\00\00)\e8\03\00\00\02\b9\01\1a;\00\00\00\02\b9\c3\0b\00\00\1a\1e\05\00\00\02\b9\82\05\00\00\1b\ff\04\00\00\02\ba\a8\03\00\00\1b%\06\00\00\02\be\5c\01\00\00\1bM\06\00\00\02\bd;\01\00\00\1b\82\06\00\00\02\bc&\00\00\00\00\02\c8\0b\00\00\03\d3\0b\00\00C\00\00\00\02Y\18@\02V\05\d4\04\00\00\e3\0b\00\00\02W\00\04\04\02W\05\cf\04\00\00*\0c\00\00\02W\00\059\00\00\006\0c\00\00\02W\00\00\05g\02\00\00\0c\0c\00\00\02X\00\04@\02X\05b\02\00\001\05\00\00\02X\00\059\00\00\00X\0c\00\00\02X\08\00\00\06-\01\00\00\07\a1\00\00\00\00\00\02;\0c\00\00\03F\0c\00\00\de\04\00\00\02N\14X\02\02L\05\ff\04\00\00\ad\03\00\00\02M\00\00\03c\0c\00\00x\02\00\00\02T\188\02P\05\cf\03\00\00s\0c\00\00\02Q\00\04\08\02Q\05\ca\03\00\00*\0c\00\00\02Q\00\059\00\00\00\04\01\00\00\02Q\00\00\05\e6\01\00\00\9c\0c\00\00\02R\00\040\02R\05\e1\01\00\00\e3\0c\00\00\02R\00\059\00\00\00\ef\0c\00\00\02R,\00\05\d5\01\00\00\c5\0c\00\00\02S\00\044\02S\05\d0\01\00\00\06\0d\00\00\02S\00\059\00\00\00\12\0d\00\00\02S0\00\00\06-\01\00\00\07\a1\00\00\00,\00\02\f4\0c\00\00\03\ff\0c\00\00\15\01\00\00\01\cf\08P\00\00\00\07\02\06-\01\00\00\07\a1\00\00\000\00\02\8f\00\00\00%D\08\00\00\d7\03\00\00\04\ed\00\03\9f,\06\00\00\02\15\01&7\00\00\00\02\15\01\a8\04\00\00'\04\ed\00\01\9f;\00\00\00\02\15\01\c3\0b\00\00'\04\ed\00\02\9f\dd\03\00\00\02\15\01\82\05\00\00*\03\91\c0\01[\03\00\00\02\18\01\dc\00\00\00#b\05\00\00\f0\03\00\00\02\17\01&\00\00\00#\8e\05\00\00\c6\04\00\00\02\19\01\04\01\00\00+\14\09\00\00\fa\02\00\00#\ba\05\00\00\b9\02\00\00\02$\01\04\01\00\00#\d8\05\00\00\c8\02\00\00\02#\01\04\01\00\00+)\09\00\00\8a\00\00\00#\04\06\00\00\c6\04\00\00\02(\01\04\01\00\00\00+\b4\09\00\00\91\00\00\00#\22\06\00\00\c6\04\00\00\020\01\04\01\00\00\00+F\0a\00\00\92\00\00\00#@\06\00\00\c6\04\00\00\028\01\04\01\00\00\00+\05\0b\00\00\0a\00\00\00#^\06\00\00\17\00\00\00\02A\01}\01\00\00\00$x\0b\00\00\14\0b\00\00\fa\00\00\00\02G\01\03\0d|\06\00\00\96\0b\00\00\0d\a8\06\00\00\a1\0b\00\00\0d\c6\06\00\00\ac\0b\00\00\0d\e4\06\00\00\b7\0b\00\00\00\00\0e\89\0e\00\00\fd\08\00\00\0e\89\0e\00\00\97\09\00\00\0e\89\0e\00\00(\0a\00\00\0e\89\0e\00\00\bb\0a\00\00\0e\9f\0e\00\00\0b\0b\00\00\0e\c2\02\00\00\eb\0b\00\00\00\0f\1a\03\00\00\02q\04\01\00\00\10\dc\00\00\00\10\dc\00\00\00\00\0fe\00\00\00\02z}\01\00\00\10\c3\0b\00\00\10}\01\00\00\00%\1d\0c\00\00\dc\00\00\00\04\ed\00\01\9f\14\03\00\00\02K\01'\04\ed\00\00\9f;\00\00\00\02K\01\c3\0b\00\00#\10\07\00\00\ff\04\00\00\02L\01\a8\03\00\00+J\0c\00\00d\00\00\00*\02\918\d4\03\00\00\02S\01\93\0f\00\00\00+\ae\0c\00\00)\00\00\00*\02\918\d4\03\00\00\02f\01\ee\0f\00\00\00\0eT\0f\00\00C\0c\00\00\0ea\0f\00\00c\0c\00\00\0er\0f\00\00\ae\0c\00\00\0e\dc\0f\00\00\d7\0c\00\00\0e,\10\00\00\df\0c\00\00\0e9\10\00\00\e9\0c\00\00\00,\84\03\00\00\02h\10\c3\0b\00\00\00\0f5\05\00\00\02|a\02\00\00\10\c3\0b\00\00\00,\0f\02\00\00\02m\10\c3\0b\00\00\10a\02\00\00\10O\02\00\00\10\dc\00\00\00\10\93\0f\00\00\00\03\9e\0f\00\00\96\05\00\00\02^\04\08\02[\05F\02\00\00\bb\0f\00\00\02\5c\00\057\00\00\00\a8\04\00\00\02]\04\00\02\c0\0f\00\00-\10\a8\04\00\00\10\c3\0b\00\00\10\d1\0f\00\00\00\03O\02\00\00\9e\05\00\00\02$,*\02\00\00\02o\10\c3\0b\00\00\10\ee\0f\00\00\00\03\f9\0f\00\00\86\05\00\00\02c\04\08\02`\05F\02\00\00\16\10\00\00\02a\00\057\00\00\00\a8\04\00\00\02b\04\00\02\1b\10\00\00-\10\a8\04\00\00\10\c3\0b\00\00\10\82\05\00\00\00,\9d\01\00\00\02i\10\a8\03\00\00\00\0f\b5\01\00\00\02j\04\01\00\00\10\a8\03\00\00\00%\fa\0c\00\00\0a\00\00\00\07\ed\03\00\00\00\00\9f\0f\03\00\00\02r\01'\04\ed\00\00\9f;\00\00\00\02r\01\c3\0b\00\00&\cb\01\00\00\02r\01\a8\04\00\00\0e\b5\0e\00\00\03\0d\00\00\00%\06\0d\00\00\9c\03\00\00\04\ed\00\03\9f`\06\00\00\02v\01&7\00\00\00\02v\01\a8\04\00\00'\04\ed\00\01\9f;\00\00\00\02v\01\c3\0b\00\00..\07\00\00\8d\04\00\00\02v\01\d1\0f\00\00*\03\91\d0\00W\03\00\00\02\cd\01\dc\00\00\00*\06\ed\00\01#\08\9fg\02\00\00\02x\01\81\14\00\00#L\07\00\00\ff\04\00\00\02w\01\a8\03\00\00#x\07\00\00\89\06\00\00\02{\01\8f\01\00\00#\b2\07\00\00^\04\00\00\02\82\01w\14\00\00#\d0\07\00\00[\03\00\00\02\81\01\86\14\00\00+\84\0d\00\00\1c\01\00\00#\ee\07\00\00'\05\00\00\02\88\01\82\05\00\00#\0c\08\00\00v\04\00\00\02\9c\01\5c\01\00\00#*\08\00\00\a8\03\00\00\02\9d\01\04\01\00\00(k\04\00\00\02\89\01&\00\00\00\00+\a1\0e\00\00\c3\00\00\00#V\08\00\00'\05\00\00\02\a6\01\82\05\00\00(k\04\00\00\02\a7\01&\00\00\00+\18\0f\00\00L\00\00\00#t\08\00\00v\04\00\00\02\b2\01\5c\01\00\00\00\00+i\0f\00\00\aa\00\00\00*\03\91\d0\00_\05\00\00\02\c6\01\dc\00\00\00#\a0\08\00\00\b2\00\00\00\02\be\01\04\01\00\00\00\0e9\10\00\00F\0d\00\00\0e[\05\00\00n\0d\00\00\0e[\05\00\00\98\0d\00\00\0el\05\00\00\b4\0d\00\00\0ew\03\00\00\16\0e\00\00\0e7\06\00\00\8e\0e\00\00\0e[\05\00\00\b5\0e\00\00\0el\05\00\00\d2\0e\00\00\0ew\03\00\00\12\0f\00\00\0e5\09\00\00d\0f\00\00\0e\9f\0e\00\00o\0f\00\00\0ew\03\00\00\82\0f\00\00\0e\9f\0e\00\00\8f\0f\00\00\0ew\03\00\00\a2\0f\00\00\0e\88\03\00\00\e9\0f\00\00\0eJ\05\00\00\f6\0f\00\00\0ew\03\00\00\10\10\00\00\0e\88\03\00\00M\10\00\00\0eJ\05\00\00Z\10\00\00\00%\a4\10\00\00\22\01\00\00\07\ed\03\00\00\00\00\9f8\06\00\00\02\d5\01&7\00\00\00\02\d5\01\a8\04\00\00'\04\ed\00\01\9f;\00\00\00\02\d5\01\c3\0b\00\00'\04\ed\00\02\9f\dd\03\00\00\02\d5\01\82\05\00\00#\cc\08\00\00\ff\04\00\00\02\d6\01\a8\03\00\00#\ea\08\00\00%\06\00\00\02\d9\01\5c\01\00\00$x\0b\00\00\ce\10\00\00\f7\00\00\00\02\dd\01\02\0c\04\ed\00\01\9f\80\0b\00\00\0c\04\ed\00\02\9f\8b\0b\00\00\0d\16\09\00\00\96\0b\00\00\0dB\09\00\00\a1\0b\00\00\0d`\09\00\00\ac\0b\00\00\0d~\09\00\00\b7\0b\00\00\00\0e\c2\02\00\00\a2\11\00\00\00%\c8\11\00\00\b8\00\00\00\04\ed\00\01\9fT\03\00\00\02\e0\01'\04\ed\00\00\9f;\00\00\00\02\e0\01\c3\0b\00\00#\aa\09\00\00\ff\04\00\00\02\e1\01\a8\03\00\00+\ea\11\00\00,\00\00\00*\02\91(\d4\03\00\00\02\e5\01\93\0f\00\00\00+6\12\00\00)\00\00\00*\02\91(\d4\03\00\00\02\f1\01\ee\0f\00\00\00\0e\da\13\00\00\12\12\00\00\0e\ec\13\00\006\12\00\00\0e\dc\0f\00\00_\12\00\00\0e+\14\00\00g\12\00\00\0e9\10\00\00q\12\00\00\00,\f3\01\00\00\02n\10\c3\0b\00\00\10\93\0f\00\00\00,\03\06\00\00\02v\10\a8\03\00\00\10\03\14\00\00\10\04\01\00\00\00\03\0e\14\00\00M\02\00\00\02\22\04\02\02\1f\05\22\00\00\00\8f\00\00\00\02 \00\05-\00\00\00\8f\00\00\00\02!\01\00,\b1\03\00\00\02k\10\a8\03\00\00\00%\81\12\00\00\0a\00\00\00\07\ed\03\00\00\00\00\9fO\03\00\00\02\fd\01'\04\ed\00\00\9f;\00\00\00\02\fd\01\c3\0b\00\00&\cb\01\00\00\02\fd\01\a8\04\00\00\0eD\13\00\00\8a\12\00\00\00\02\04\01\00\00\02\d4\01\00\00\02X\0c\00\00\02\dc\00\00\00\00")
  (@custom ".debug_ranges" (after code) "\06\00\00\00\0a\01\00\00\0c\01\00\00\db\01\00\00\dd\01\00\00|\04\00\00~\04\00\00N\05\00\00P\05\00\00\f2\07\00\00\f3\07\00\00B\08\00\00D\08\00\00\1b\0c\00\00\1d\0c\00\00\f9\0c\00\00\fa\0c\00\00\04\0d\00\00\06\0d\00\00\a2\10\00\00\a4\10\00\00\c6\11\00\00\c8\11\00\00\80\12\00\00\81\12\00\00\8b\12\00\00\00\00\00\00\00\00\00\00")
  (@custom ".debug_str" (after code) "_pad_topology\00Topology\00rand_index\00buffer_idx\00field_idx\00env\00context\00Context\00nest\00unsigned short\00Point\00oxitortoise_next_int\00point_ahead_int\00unsigned int\00uphill_nest_scent\00PointInt\00rand_result\00scent_right\00chemical_right\00scent_left\00chemical_left\00Float\00oxitortoise_patch_at\00uint8_t\00uint16_t\00uint64_t\00uint32_t\00/home/anderiux/data/NetLogo/oxitortoise/oxitortoise/bench/models/ants\00_pad_patch_buffers\00_pad_turtle_buffers\00oxitortoise_reset_ticks\00oxitortoise_get_ticks\00args\00_pad_patch_flags\00_pad_turtle_flags\00oxitortoise_for_all_turtles\00oxitortoise_create_turtles\00oxitortoise_for_all_patches\00fn_ptr\00AgentFieldDescriptor\00_pad_dirty_aggregator\00DirtyAggregator\00pcolor\00plabel_color\00oxitortoise_scale_color\00_pad_max_pycor\00_pad_max_pxcor\00_pad_tick_counter\00RowBuffer\00food_source_number\00unsigned char\00shim_setup\00oxitortoise_distance_euclidean_no_wrap\00who\00TurtleWho\00shim_go\00new_position\00hidden\00oxitortoise_is_nan\00_Bool\00oxitortoise_clear_all\00plabel\00uphill_chemical\00oxitortoise_advance_tick\00_pad_tick\00callback\00next_patch\00recolor_patch\00unsigned long long\00RustString\00oxitortoise_offset_distance_by_heading\00real_heading\00oxitortoise_normalize_heading\00size\00patch_here\00patch2_here\00shape_name\00next_turtle\00nest_scent_at_angle\00chemical_at_angle\00double\00distance\00_pad_workspace\00Workspace\00food\00oxitortoise_round\00world\00World\00occupancy_bitfield\00patch_id\00patch_here_id\00oxitortoise_get_default_turtle_breed\00_pad\00point_ahead\00scent_ahead\00chemical_ahead\00CallbackPatchId\00CallbackTurtleId\00BreedId\00model_code.c\00base_data\00turtle_data\00PatchBaseData\00TurtleBaseData\00__ARRAY_SIZE_TYPE__\00oxitortoise_diffuse_8\00PatchGroup2\00patch2\00setup_body1\00go_body1\00PatchGroup1\00patch1\00setup_body0\00go_body0\00PatchGroup0\00TurtleGroup0\00patch0\00turtle0\00clang version 21.0.0git (https:/github.com/llvm/llvm-project 0f0079c29da4b4d5bbd43dced1db9ad6c6d11008)\00")
  (@custom ".debug_line" (after code) "\ee\09\00\00\04\00\80\00\00\00\01\01\01\fb\0e\0d\00\01\01\01\01\00\00\00\01\00\00\01/home/anderiux\00\00.installs/emsdk/upstream/emscripten/cache/sysroot/include/bits/alltypes.h\00\01\00\00model_code.c\00\00\00\00\00\04\02\00\05\02\06\00\00\00\03\b8\01\01\05F\0a\94\05%9\05A[\81\05F\06\08\12\05\0e\061\05\06\06X\03\c1~<\03\bf\01\ac\03\c1~.\05\15\06\03\c1\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\be~<\03\c2\01\ac\03\be~.\05)\06\03\c4\01\d6\06\03\bc~<\03\c4\01\ac\03\bc~.\06\03\c6\01\ba\06\03\ba~<\03\c6\01\ac\03\ba~.\05A\06\03\cb\01\08.\06\03\b5~<\05\14\03\cb\01\08 \03\b5~f\05*\06\03\cd\01\d6\05\02\06X\056<\05\01\06\c9\02\01\00\01\01\04\02\00\05\02\0c\01\00\00\03\cf\01\01\05=\0a\08=\05\17\06X\05\16\06\83\06\03\ae~\02:\01\05%\06\03\d4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08Z\05F\06\9e\05\11\06/\06\03\a4~<\05\01\06\03\dd\01<\05\00\06\03\a3~\ac\05\01\03\dd\01.\02\01\00\01\01\04\02\00\05\02\dd\01\00\00\03\de\01\01\05<\0a\08\9f\06\03\a0~X\05=\06\03\d1\01\90\05\17\06 \05\16\06\83\06\03\ae~\02=\01\05%\06\03\d4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08\84\05F\06\9e\05\11\06/\06\03\a4~<\05<\06\03\e1\01t\06\03\9f~X\05=\06\03\d1\01\90\05\17\06 \05\16\06\83\06\03\ae~\028\01\05%\06\03\d4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08\84\05F\06\9e\05\11\06/\06\03\a4~<\05;\06\03\e2\01t\06\03\9e~X\05=\06\03\d1\01\90\05\17\06 \05\16\06\83\06\03\ae~\02;\01\05%\06\03\d4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08Z\05F\06\9e\05\11\06/\06\03\a4~<\05\15\06\03\e3\01t\05&\06\90\03\9d~\9e\05\16\06\03\e4\01\08\90\05\00\06\03\9c~f\05\01\06\03\ec\01\ac\02\0d\00\01\01\04\02\00\05\02~\04\00\00\03\ed\01\01\05=\0a\08=\05\17\06X\05\16\06\83\06\03\90~\02:\01\05%\06\03\f2\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08Z\05\11\ad\06\03\86~t\05\01\06\03\fb\01 \05\00\06\03\85~\ac\05\01\03\fb\01.\02\01\00\01\01\04\02\00\05\02P\05\00\00\03\fc\01\01\05;\0a\08\9f\06\03\82~X\05=\06\03\ef\01\90\05\17\06 \05\16\06\83\06\03\90~\02=\01\05%\06\03\f2\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08\84\05\11\ad\06\03\86~t\05;\06\03\ff\01X\06\03\81~X\05=\06\03\ef\01\90\05\17\06 \05\16\06\83\06\03\90~\028\01\05%\06\03\f2\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08\84\05\11\ad\06\03\86~t\05:\06\03\80\02X\06\03\80~X\05=\06\03\ef\01\90\05\17\06 \05\16\06\83\06\03\90~\02;\01\05%\06\03\f2\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05A\08Z\05\11\ad\06\03\86~t\05\12\06\03\81\02X\05 \06\90\03\ff}\9e\05\13\06\03\82\02\08\90\05\00\06\03\fe}f\05\01\06\03\8a\02\ac\02\0d\00\01\01\04\02\05A\0a\00\05\02\f6\07\00\00\03\8c\02\01\05^\06X\05c<\05,\06\83\05\13\e7\06\03\ef}<\05\12\06\03\90\02\c8\05+?\05\02\06\90\05G.\05\01\06\d7\02\01\00\01\01\04\02\00\05\02D\08\00\00\03\94\02\01\05$\0a\08\a1\059\8f\05U\06t\05Z\90\03\e9}.\05$\06\03\98\02J\05D\f3\05$\d5\05D\bb\05\13\06J\05\1c\06\02Q\18\05\14\06<\05\19\06\ef\05\0e\06 \05(\06C\05FY\81\05M\87\05\15\06\f2\05W\02/\12\05M \05\15J\03\d8}\02*\01\05\11\06\03\a9\02\90\06\03\d7}J\05\1f\06\03\aa\02\ba\06\03\d6}<\05M\06\03\b0\02 \05o\06\08X\05M \05\15<\05X\02-\12\05M \05\15J\03\d0}\02(\01\05\11\06\03\b1\02\90\06\03\cf}J\05\1f\06\03\b2\02\c8\06\03\ce}<\05M\06\03\b8\02 \05n\06\08X\05M \05\15<\05X\021\12\05M \05\15J\03\c8}\02%\01\05\11\06\03\b9\02\90\06\03\c7}J\05\1f\06\03\ba\02\c8\06\03\c6}<\05\0f\06\03\c0\02 \05\22\06\f2\03\c0}J\05\1b\06\03\c1\02\08t\05\13g\05\11\06 \03\be}<\05F\06\03\bd\01X\05%9\05A[\81\05F\06\08\12\05\0e\061\05\06\06X\03\c1~<\03\bf\01\ac\03\c1~.\05\15\06\03\c1\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\be~<\03\c2\01\ac\03\be~.\05)\06\03\c4\01\d6\06\03\bc~<\03\c4\01\ac\03\bc~.\06\03\c6\01\ba\06\03\ba~<\03\c6\01\ac\03\ba~.\05A\06\03\cb\01\08.\06\03\b5~<\05\14\03\cb\01\08 \03\b5~f\05*\06\03\cd\01\d6\05\02\06X\056<\05\01\06\03\fc\00\c8\02\0d\00\01\01\04\02\00\05\02\1d\0c\00\00\03\ca\02\01\05%\0a\08g\05\02w\06\03\b1}\82\05\13\06\03\d9\02t\05\1f\03z \05\04\03\0a\f2\a0\05\03\c4\05\04\08\16\05\03p\05\1e\03\0b\02$\01\05\03\08$\05\02\08j\05'\83\05%\06\9e\05\01\06=\02\0d\00\01\01\04\02\05\02\0a\00\05\02\fb\0c\00\00\03\f2\02\01\05\01\83\02\01\00\01\01\04\02\00\05\02\06\0d\00\00\03\f5\02\01\05%\0a\08\9f\05E\5c\05J\06X\05\19\06\9f\05 \06t\05\06\9e\05\1d<\03\84}<\06\03\82\03X\06\03\fe|<\05\1e\06\03\81\03X\05\19@\05\00\06\03\fb|t\05\1f\03\85\03\08\c8\05B\06?\05\9d\01\06t\05\81\01t\05wf\05B.\05\1c<\05G\06\08\83\05\14\e7\05\19\06\08 \03\f4|J\05\1e\06\03\8e\03\c8\05\16\081\05.M\057\06\f2\05\10 \05\0ef\05$\06?\05\05\06\90\05@.\05\05\06\d7\06\03\e8|.\05H\06\03\9c\03 \05M\06\9e\05\22\06/\06\03\e3|<\05\11\06\03\9e\03\ac\05\19\06 \03\e2|<\03\9e\03\ac\05\05\06L\06\03\e0|\02,\01\05\19\06\03\8c\03\ba\06\03\f4|f\05B\06\03\a6\03 \05\9d\01\06t\05\81\01t\05wf\05B.\05\1c<\05G\06\08\91\05\14\cb\05\08\06t\03\d6|<\05\1e\06\03\ac\03\c8\05.?\057\06\f2\05\10 \05\0ef\05\04\06=\06\03\d0|.\05I\06\03\b2\03 \05N\06\9e\05\1b\06/\05\05\08?\06\03\ca|\02,\01\05\1e\06\03\be\03X\05,\83\05\17s\055=\05\0e\06 \05\0cf\05\18\06w\05,\83\05\11s\055=\05\0e\06 \05\0cf\05P\06?\05\17\06t\05&\06\02=\13\05\07\06t\05-\06\91\056\06\f2\05\0f \05\0df\03\b8|<\05P\06\03\cd\03 \05\17\06\ac\05'\06\02.\13\05\07\06t\05\06f\05\0f\06/\06\03\b1|\08\c8\05!\06\03\d2\03 \05\02\06\90\05=.\03\ae|\d6\05\01\06\03\d3\03 \02\0d\00\01\01\04\02\05%\0a\00\05\02\a9\10\00\00\03\d5\03\01\05A[\06\03\a7|J\05F\03\d9\03J\05\13\06K\05%\03\e0}\08<\05A[\81\05F\06\08\12\05\0e\061\05\06\06X\03\c1~<\03\bf\01\ac\03\c1~.\05\15\06\03\c1\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\be~<\03\c2\01\ac\03\be~.\05)\06\03\c4\01\d6\06\03\bc~<\03\c4\01\ac\03\bc~.\06\03\c6\01\ba\06\03\ba~<\03\c6\01\ac\03\ba~.\05A\06\03\cb\01\08.\06\03\b5~<\05\14\03\cb\01\08 \03\b5~f\05*\06\03\cd\01\d6\05\02\06X\056<\05\01\06\03\91\02\c8\02\01\00\01\01\04\02\00\05\02\c8\11\00\00\03\df\03\01\05%\0a\08=\06\03\9f|t\05\13\06\03\e7\03t\05\1f\1e\05\03\f6\06\03\97|\08f\05\1f\06\03\ed\03J\05\02\06t\05\1e\06\08\86\05\03\08$\05\02\08j\05'\83\05%\06\9e\05\01\06=\02\0c\00\01\01\04\02\05\02\0a\00\05\02\82\12\00\00\03\fd\03\01\05\01\83\02\01\00\01\01")
  (@custom "target_features" (after code) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)
