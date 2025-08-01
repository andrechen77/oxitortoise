(module $model_code.wasm
  (@dylink.0
    (mem-info)
  )
  (type (;0;) (func (param f64 f64 f64 f64) (result f64)))
  (type (;1;) (func (param f64) (result f64)))
  (type (;2;) (func (param i32 i32 i32 f64 f64)))
  (type (;3;) (func (param f64) (result i32)))
  (type (;4;) (func (param i32 i32) (result i32)))
  (type (;5;) (func (param i32)))
  (type (;6;) (func (param i32) (result i64)))
  (type (;7;) (func (param i32 i64 i64 i32) (result i32)))
  (type (;8;) (func (param i32 i32)))
  (type (;9;) (func (param i32) (result i32)))
  (type (;10;) (func (param i32 i32) (result f64)))
  (type (;11;) (func (param i32) (result f64)))
  (type (;12;) (func (param i32 i32 f64)))
  (type (;13;) (func))
  (type (;14;) (func (param i32 i32 f64 f64) (result f64)))
  (type (;15;) (func (param i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 0))
  (import "env" "__indirect_function_table" (table (;0;) 0 funcref))
  (import "env" "__stack_pointer" (global $__stack_pointer (;0;) (mut i32)))
  (import "env" "oxitortoise_scale_color" (func $oxitortoise_scale_color (;0;) (type 0)))
  (import "env" "oxitortoise_normalize_heading" (func $oxitortoise_normalize_heading (;1;) (type 1)))
  (import "env" "oxitortoise_offset_distance_by_heading" (func $oxitortoise_offset_distance_by_heading (;2;) (type 2)))
  (import "env" "oxitortoise_is_nan" (func $oxitortoise_is_nan (;3;) (type 3)))
  (import "env" "oxitortoise_round" (func $oxitortoise_round (;4;) (type 1)))
  (import "env" "oxitortoise_patch_at" (func $oxitortoise_patch_at (;5;) (type 4)))
  (import "env" "oxitortoise_clear_all" (func $oxitortoise_clear_all (;6;) (type 5)))
  (import "env" "oxitortoise_get_default_turtle_breed" (func $oxitortoise_get_default_turtle_breed (;7;) (type 6)))
  (import "env" "oxitortoise_create_turtles" (func $oxitortoise_create_turtles (;8;) (type 7)))
  (import "env" "oxitortoise_next_turtle_from_iter" (func $oxitortoise_next_turtle_from_iter (;9;) (type 8)))
  (import "env" "oxitortoise_drop_turtle_iter" (func $oxitortoise_drop_turtle_iter (;10;) (type 5)))
  (import "env" "oxitortoise_make_all_patches_iter" (func $oxitortoise_make_all_patches_iter (;11;) (type 9)))
  (import "env" "oxitortoise_next_patch_from_iter" (func $oxitortoise_next_patch_from_iter (;12;) (type 9)))
  (import "env" "oxitortoise_distance_euclidean_no_wrap" (func $oxitortoise_distance_euclidean_no_wrap (;13;) (type 10)))
  (import "env" "oxitortoise_next_int" (func $oxitortoise_next_int (;14;) (type 4)))
  (import "env" "oxitortoise_drop_patch_iter" (func $oxitortoise_drop_patch_iter (;15;) (type 5)))
  (import "env" "oxitortoise_reset_ticks" (func $oxitortoise_reset_ticks (;16;) (type 5)))
  (import "env" "oxitortoise_get_ticks" (func $oxitortoise_get_ticks (;17;) (type 11)))
  (import "env" "oxitortoise_make_all_turtles_iter" (func $oxitortoise_make_all_turtles_iter (;18;) (type 9)))
  (import "env" "oxitortoise_diffuse_8" (func $oxitortoise_diffuse_8 (;19;) (type 12)))
  (import "env" "oxitortoise_advance_tick" (func $oxitortoise_advance_tick (;20;) (type 5)))
  (export "__wasm_call_ctors" (func $__wasm_call_ctors))
  (export "recolor_patch" (func $recolor_patch))
  (export "chemical_at_angle" (func $chemical_at_angle))
  (export "uphill_chemical" (func $uphill_chemical))
  (export "nest_scent_at_angle" (func $nest_scent_at_angle))
  (export "uphill_nest_scent" (func $uphill_nest_scent))
  (export "setup" (func $setup))
  (export "shim_setup" (func $shim_setup))
  (export "go" (func $go))
  (export "shim_go" (func $shim_go))
  (func $__wasm_call_ctors (;21;) (type 13))
  (func $recolor_patch (;22;) (type 8) (param i32 i32)
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
  (func $chemical_at_angle (;23;) (type 14) (param i32 i32 f64 f64) (result f64)
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
  (func $uphill_chemical (;24;) (type 15) (param i32 i32 i32)
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
  (func $nest_scent_at_angle (;25;) (type 14) (param i32 i32 f64 f64) (result f64)
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
  (func $uphill_nest_scent (;26;) (type 15) (param i32 i32 i32)
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
  (func $setup (;27;) (type 5) (param i32)
    (local i32 i32 i64 i32 i32 i32 i32 i32 i32 f64 f64)
    (global.set $__stack_pointer
      (local.tee 1
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 240))))
    (local.set 2
      (i32.load
        (local.get 0)))
    (call $oxitortoise_clear_all
      (local.get 0))
    (local.set 3
      (call $oxitortoise_get_default_turtle_breed
        (local.get 0)))
    (i64.store
      (i32.add
        (i32.add
          (local.get 1)
          (i32.const 224))
        (i32.const 8))
      (i64.const 0))
    (i64.store
      (i32.add
        (i32.add
          (local.get 1)
          (i32.const 128))
        (i32.const 8))
      (i64.const 0))
    (i64.store offset=224
      (local.get 1)
      (i64.const 0))
    (i64.store offset=128
      (local.get 1)
      (i64.const 0))
    (call $oxitortoise_next_turtle_from_iter
      (i32.add
        (local.get 1)
        (i32.const 208))
      (local.tee 4
        (call $oxitortoise_create_turtles
          (local.get 0)
          (local.get 3)
          (i64.const 125)
          (i32.add
            (local.get 1)
            (i32.const 128)))))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eqz
          (i32.or
            (local.tee 5
              (i32.load offset=208
                (local.get 1)))
            (i32.load offset=212
              (local.get 1)))))
      (local.set 5
        (local.get 5))
      (loop ;; label = @2
        (i64.store offset=40
          (local.tee 6
            (i32.add
              (i32.load offset=16
                (local.get 2))
              (i32.mul
                (local.tee 5
                  (local.get 5))
                (i32.const 112))))
          (i64.const 4624633867356078080))
        (i64.store offset=80
          (local.get 6)
          (i64.const 4611686018427387904))
        (i32.store16
          (local.tee 5
            (i32.add
              (i32.load offset=52
                (local.get 0))
              (i32.shl
                (local.get 5)
                (i32.const 1))))
          (i32.or
            (i32.load16_u
              (local.get 5))
            (i32.const 514)))
        (call $oxitortoise_next_turtle_from_iter
          (i32.add
            (local.get 1)
            (i32.const 208))
          (local.get 4))
        (local.set 5
          (local.tee 6
            (i32.load offset=208
              (local.get 1))))
        (br_if 0 (;@2;)
          (i32.or
            (local.get 6)
            (i32.load offset=212
              (local.get 1))))))
    (call $oxitortoise_drop_turtle_iter
      (local.get 4))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eq
          (local.tee 5
            (call $oxitortoise_next_patch_from_iter
              (local.tee 7
                (call $oxitortoise_make_all_patches_iter
                  (local.get 0)))))
          (i32.const -1)))
      (local.set 5
        (local.get 5))
      (loop ;; label = @2
        (i64.store
          (local.tee 6
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 208))
              (i32.const 8)))
          (i64.load
            (local.tee 9
              (i32.add
                (local.tee 5
                  (i32.add
                    (i32.load offset=344
                      (local.get 2))
                    (local.tee 8
                      (i32.mul
                        (local.tee 4
                          (local.get 5))
                        (i32.const 80)))))
                (i32.const 16)))))
        (local.set 3
          (i64.load offset=8
            (local.get 5)))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 192))
            (i32.const 8))
          (i64.const 0))
        (i64.store offset=208
          (local.get 1)
          (local.get 3))
        (i64.store offset=192
          (local.get 1)
          (i64.const 0))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 112))
            (i32.const 8))
          (i64.load
            (local.get 9)))
        (local.set 3
          (i64.load offset=8
            (local.get 5)))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 96))
            (i32.const 8))
          (i64.const 0))
        (i64.store offset=112
          (local.get 1)
          (local.get 3))
        (i64.store offset=96
          (local.get 1)
          (i64.const 0))
        (f64.store offset=64
          (local.get 5)
          (f64.sub
            (f64.const 0x1.9p+7 (;=200;))
            (local.tee 10
              (call $oxitortoise_distance_euclidean_no_wrap
                (i32.add
                  (local.get 1)
                  (i32.const 112))
                (i32.add
                  (local.get 1)
                  (i32.const 96))))))
        (i32.store8 offset=56
          (local.get 5)
          (f64.lt
            (local.get 10)
            (f64.const 0x1.4p+2 (;=5;))))
        (local.set 11
          (f64.load offset=552
            (local.get 2)))
        (local.set 10
          (f64.load offset=528
            (local.get 2)))
        (i64.store
          (local.tee 9
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 176))
              (i32.const 8)))
          (i64.const 0))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 80))
            (i32.const 8))
          (i64.load
            (local.get 6)))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 64))
            (i32.const 8))
          (i64.load
            (local.get 9)))
        (f64.store offset=176
          (local.get 1)
          (f64.mul
            (local.get 10)
            (f64.const 0x1.3333333333333p-1 (;=0.6;))))
        (i64.store offset=80
          (local.get 1)
          (i64.load offset=208
            (local.get 1)))
        (i64.store offset=64
          (local.get 1)
          (i64.load offset=176
            (local.get 1)))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (f64.lt
                (call $oxitortoise_distance_euclidean_no_wrap
                  (i32.add
                    (local.get 1)
                    (i32.const 80))
                  (i32.add
                    (local.get 1)
                    (i32.const 64)))
                (f64.const 0x1.4p+2 (;=5;)))))
          (i64.store offset=72
            (local.get 5)
            (i64.const 4607182418800017408)))
        (f64.store
          (local.tee 9
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 160))
              (i32.const 8)))
          (f64.mul
            (local.get 11)
            (f64.const -0x1.3333333333333p-1 (;=-0.6;))))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 48))
            (i32.const 8))
          (i64.load
            (local.get 6)))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 32))
            (i32.const 8))
          (i64.load
            (local.get 9)))
        (f64.store offset=160
          (local.get 1)
          (f64.mul
            (local.get 10)
            (f64.const -0x1.3333333333333p-1 (;=-0.6;))))
        (i64.store offset=48
          (local.get 1)
          (i64.load offset=208
            (local.get 1)))
        (i64.store offset=32
          (local.get 1)
          (i64.load offset=160
            (local.get 1)))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (f64.lt
                (call $oxitortoise_distance_euclidean_no_wrap
                  (i32.add
                    (local.get 1)
                    (i32.const 48))
                  (i32.add
                    (local.get 1)
                    (i32.const 32)))
                (f64.const 0x1.4p+2 (;=5;)))))
          (i64.store offset=72
            (local.get 5)
            (i64.const 4611686018427387904)))
        (f64.store
          (local.tee 9
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 144))
              (i32.const 8)))
          (f64.mul
            (local.get 11)
            (f64.const 0x1.999999999999ap-1 (;=0.8;))))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 16))
            (i32.const 8))
          (i64.load
            (local.get 6)))
        (i64.store
          (i32.add
            (local.get 1)
            (i32.const 8))
          (i64.load
            (local.get 9)))
        (f64.store offset=144
          (local.get 1)
          (f64.mul
            (local.get 10)
            (f64.const -0x1.999999999999ap-1 (;=-0.8;))))
        (i64.store offset=16
          (local.get 1)
          (i64.load offset=208
            (local.get 1)))
        (i64.store
          (local.get 1)
          (i64.load offset=144
            (local.get 1)))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (f64.lt
                (call $oxitortoise_distance_euclidean_no_wrap
                  (i32.add
                    (local.get 1)
                    (i32.const 16))
                  (local.get 1))
                (f64.const 0x1.4p+2 (;=5;)))))
          (i64.store offset=72
            (local.get 5)
            (i64.const 4613937818241073152)))
        (block ;; label = @3
          (br_if 0 (;@3;)
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
                (local.get 0)
                (i32.const 2)))))
        (local.set 6
          (i32.shl
            (local.get 4)
            (i32.const 3)))
        (local.set 9
          (i32.load offset=380
            (local.tee 5
              (i32.load
                (local.get 0)))))
        (block ;; label = @3
          (block ;; label = @4
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (i32.load8_u offset=56
                    (local.tee 8
                      (i32.add
                        (i32.load offset=344
                          (local.get 5))
                        (local.get 8))))))
              (local.set 10
                (f64.const 0x1.ccp+6 (;=115;)))
              (br 1 (;@4;)))
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (f64.gt
                    (f64.load offset=48
                      (local.get 8))
                    (f64.const 0x0p+0 (;=0;)))))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (f64.ne
                    (local.tee 10
                      (f64.load offset=72
                        (local.get 8)))
                    (f64.const 0x1p+0 (;=1;))))
                (local.set 10
                  (f64.const 0x1.54p+6 (;=85;)))
                (br 2 (;@4;)))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (f64.ne
                    (local.get 10)
                    (f64.const 0x1p+1 (;=2;))))
                (local.set 10
                  (f64.const 0x1.7cp+6 (;=95;)))
                (br 2 (;@4;)))
              (br_if 2 (;@3;)
                (f64.ne
                  (local.get 10)
                  (f64.const 0x1.8p+1 (;=3;))))
              (local.set 10
                (f64.const 0x1.a4p+6 (;=105;)))
              (br 1 (;@4;)))
            (local.set 10
              (call $oxitortoise_scale_color
                (f64.const 0x1.04p+6 (;=65;))
                (f64.load
                  (i32.add
                    (i32.load offset=416
                      (local.get 5))
                    (local.get 6)))
                (f64.const 0x1.999999999999ap-4 (;=0.1;))
                (f64.const 0x1.4p+2 (;=5;)))))
          (f64.store
            (i32.add
              (local.get 9)
              (local.get 6))
            (local.get 10)))
        (i32.store8
          (local.tee 5
            (i32.add
              (i32.load offset=56
                (local.get 0))
              (local.get 4)))
          (i32.or
            (i32.load8_u
              (local.get 5))
            (i32.const 1)))
        (local.set 5
          (local.tee 6
            (call $oxitortoise_next_patch_from_iter
              (local.get 7))))
        (br_if 0 (;@2;)
          (i32.ne
            (local.get 6)
            (i32.const -1)))))
    (call $oxitortoise_drop_patch_iter
      (local.get 7))
    (call $oxitortoise_reset_ticks
      (local.get 2))
    (f64.store offset=8
      (local.get 0)
      (call $oxitortoise_get_ticks
        (local.get 2)))
    (global.set $__stack_pointer
      (i32.add
        (local.get 1)
        (i32.const 240)))
  )
  (func $shim_setup (;28;) (type 8) (param i32 i32)
    (call $setup
      (local.get 0))
  )
  (func $go (;29;) (type 5) (param i32)
    (local i32 i32 i32 i32 i32 i64 i32 i32 f64 i32 i32 i32)
    (global.set $__stack_pointer
      (local.tee 1
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 128))))
    (local.set 2
      (i32.load
        (local.get 0)))
    (call $oxitortoise_next_turtle_from_iter
      (i32.add
        (local.get 1)
        (i32.const 96))
      (local.tee 3
        (call $oxitortoise_make_all_turtles_iter
          (local.get 0))))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eqz
          (i32.or
            (local.tee 4
              (i32.load offset=96
                (local.get 1)))
            (i32.load offset=100
              (local.get 1)))))
      (local.set 4
        (local.get 4))
      (loop ;; label = @2
        (local.set 6
          (i64.load offset=8
            (local.tee 4
              (i32.add
                (i32.load offset=16
                  (local.get 2))
                (i32.mul
                  (local.tee 5
                    (local.get 4))
                  (i32.const 112))))))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (f64.le
              (call $oxitortoise_get_ticks
                (local.get 2))
              (f64.convert_i64_u
                (local.get 6))))
          (local.set 7
            (i32.add
              (local.get 4)
              (i32.const 88)))
          (local.set 8
            (i32.add
              (local.get 4)
              (i32.const 96)))
          (local.set 9
            (f64.load offset=40
              (local.get 4)))
          (local.set 10
            (i32.trunc_sat_f64_u
              (call $oxitortoise_round
                (f64.load offset=96
                  (local.get 4)))))
          (block ;; label = @4
            (block ;; label = @5
              (br_if 0 (;@5;)
                (f64.ne
                  (local.get 9)
                  (f64.const 0x1.ep+3 (;=15;))))
              (i32.store offset=120
                (local.get 1)
                (local.get 10))
              (i32.store offset=124
                (local.get 1)
                (i32.trunc_sat_f64_u
                  (call $oxitortoise_round
                    (f64.load offset=104
                      (local.get 4)))))
              (i64.store offset=56
                (local.get 1)
                (i64.load offset=120 align=4
                  (local.get 1)))
              (local.set 10
                (call $oxitortoise_patch_at
                  (local.get 2)
                  (i32.add
                    (local.get 1)
                    (i32.const 56))))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (i32.eqz
                    (local.tee 12
                      (f64.gt
                        (local.tee 9
                          (f64.load offset=48
                            (local.tee 11
                              (i32.add
                                (i32.load offset=344
                                  (local.get 2))
                                (i32.mul
                                  (local.get 10)
                                  (i32.const 80))))))
                        (f64.const 0x0p+0 (;=0;))))))
                (i64.store offset=40
                  (local.get 4)
                  (i64.const 4628011567076605952))
                (f64.store
                  (i32.add
                    (local.get 11)
                    (i32.const 48))
                  (f64.add
                    (local.get 9)
                    (f64.const -0x1p+0 (;=-1;))))
                (f64.store offset=88
                  (local.get 4)
                  (call $oxitortoise_normalize_heading
                    (f64.add
                      (f64.load offset=88
                        (local.get 4))
                      (f64.const 0x1.68p+7 (;=180;)))))
                (i32.store16
                  (local.tee 4
                    (i32.add
                      (i32.load offset=52
                        (local.get 0))
                      (i32.shl
                        (local.get 5)
                        (i32.const 1))))
                  (i32.or
                    (i32.load16_u
                      (local.get 4))
                    (i32.const 1030)))
                (br_if 2 (;@4;)
                  (i32.eqz
                    (local.get 12)))
                (br 3 (;@3;)))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (i32.eqz
                    (f64.ge
                      (local.tee 9
                        (f64.load
                          (i32.add
                            (i32.load offset=416
                              (local.get 2))
                            (i32.shl
                              (local.get 10)
                              (i32.const 3)))))
                      (f64.const 0x1.999999999999ap-5 (;=0.05;)))))
                (br_if 0 (;@6;)
                  (i32.eqz
                    (f64.lt
                      (local.get 9)
                      (f64.const 0x1p+1 (;=2;)))))
                (i64.store
                  (i32.add
                    (i32.add
                      (local.get 1)
                      (i32.const 40))
                    (i32.const 8))
                  (i64.load
                    (i32.add
                      (local.get 8)
                      (i32.const 8))))
                (i64.store offset=40
                  (local.get 1)
                  (i64.load
                    (local.get 8)))
                (call $uphill_chemical
                  (local.get 2)
                  (i32.add
                    (local.get 1)
                    (i32.const 40))
                  (local.get 7)))
              (br_if 1 (;@4;)
                (i32.eqz
                  (local.get 12)))
              (br 2 (;@3;)))
            (i32.store offset=112
              (local.get 1)
              (local.get 10))
            (i32.store offset=116
              (local.get 1)
              (i32.trunc_sat_f64_u
                (call $oxitortoise_round
                  (f64.load offset=104
                    (local.get 4)))))
            (i64.store offset=80
              (local.get 1)
              (i64.load offset=112 align=4
                (local.get 1)))
            (local.set 10
              (call $oxitortoise_patch_at
                (local.get 2)
                (i32.add
                  (local.get 1)
                  (i32.const 80))))
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.ne
                  (i32.load8_u offset=56
                    (i32.add
                      (i32.load offset=344
                        (local.get 2))
                      (i32.mul
                        (local.get 10)
                        (i32.const 80))))
                  (i32.const 1)))
              (i64.store offset=40
                (local.get 4)
                (i64.const 4624633867356078080))
              (f64.store offset=88
                (local.get 4)
                (call $oxitortoise_normalize_heading
                  (f64.add
                    (f64.load offset=88
                      (local.get 4))
                    (f64.const 0x1.68p+7 (;=180;)))))
              (br 1 (;@4;)))
            (f64.store
              (local.tee 4
                (i32.add
                  (i32.load offset=416
                    (local.get 2))
                  (i32.shl
                    (local.get 10)
                    (i32.const 3))))
              (f64.add
                (f64.load
                  (local.get 4))
                (f64.const 0x1.ep+5 (;=60;))))
            (i64.store
              (i32.add
                (i32.add
                  (local.get 1)
                  (i32.const 64))
                (i32.const 8))
              (i64.load
                (i32.add
                  (local.get 8)
                  (i32.const 8))))
            (i64.store offset=64
              (local.get 1)
              (i64.load
                (local.get 8)))
            (call $uphill_nest_scent
              (local.get 2)
              (i32.add
                (local.get 1)
                (i32.const 64))
              (local.get 7)))
          (local.set 4
            (call $oxitortoise_next_int
              (local.get 0)
              (i32.const 40)))
          (f64.store
            (local.get 7)
            (call $oxitortoise_normalize_heading
              (f64.add
                (f64.load
                  (local.get 7))
                (f64.convert_i32_u
                  (local.get 4)))))
          (local.set 4
            (call $oxitortoise_next_int
              (local.get 0)
              (i32.const 40)))
          (f64.store
            (local.get 7)
            (call $oxitortoise_normalize_heading
              (f64.sub
                (f64.load
                  (local.get 7))
                (f64.convert_i32_u
                  (local.get 4)))))
          (local.set 9
            (f64.load
              (local.get 7)))
          (i64.store
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 24))
              (i32.const 8))
            (i64.load
              (local.tee 4
                (i32.add
                  (local.get 8)
                  (i32.const 8)))))
          (i64.store offset=24
            (local.get 1)
            (i64.load
              (local.get 8)))
          (call $oxitortoise_offset_distance_by_heading
            (i32.add
              (local.get 1)
              (i32.const 96))
            (local.get 2)
            (i32.add
              (local.get 1)
              (i32.const 24))
            (local.get 9)
            (f64.const 0x1p+0 (;=1;)))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (i32.eqz
                (call $oxitortoise_is_nan
                  (f64.load offset=96
                    (local.get 1)))))
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
              (i32.add
                (local.get 1)
                (i32.const 8))
              (i32.const 8))
            (i64.load
              (local.get 4)))
          (i64.store offset=8
            (local.get 1)
            (i64.load
              (local.get 8)))
          (call $oxitortoise_offset_distance_by_heading
            (i32.add
              (local.get 1)
              (i32.const 96))
            (local.get 2)
            (i32.add
              (local.get 1)
              (i32.const 8))
            (local.get 9)
            (f64.const 0x1p+0 (;=1;)))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (call $oxitortoise_is_nan
                (f64.load offset=96
                  (local.get 1))))
            (i64.store
              (local.get 8)
              (i64.load offset=96
                (local.get 1)))
            (i64.store
              (local.get 4)
              (i64.load
                (i32.add
                  (i32.add
                    (local.get 1)
                    (i32.const 96))
                  (i32.const 8)))))
          (i32.store16
            (local.tee 4
              (i32.add
                (i32.load offset=52
                  (local.get 0))
                (i32.shl
                  (local.get 5)
                  (i32.const 1))))
            (i32.or
              (i32.load16_u
                (local.get 4))
              (i32.const 1030))))
        (call $oxitortoise_next_turtle_from_iter
          (i32.add
            (local.get 1)
            (i32.const 96))
          (local.get 3))
        (local.set 4
          (local.tee 7
            (i32.load offset=96
              (local.get 1))))
        (br_if 0 (;@2;)
          (i32.or
            (local.get 7)
            (i32.load offset=100
              (local.get 1))))))
    (call $oxitortoise_drop_turtle_iter
      (local.get 3))
    (i32.store16 offset=6
      (local.get 1)
      (i32.const 2))
    (i32.store16 offset=94 align=1
      (local.get 1)
      (i32.const 2))
    (call $oxitortoise_diffuse_8
      (local.get 2)
      (i32.add
        (local.get 1)
        (i32.const 6))
      (f64.const 0x1p-1 (;=0.5;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eq
          (local.tee 4
            (call $oxitortoise_next_patch_from_iter
              (local.tee 10
                (call $oxitortoise_make_all_patches_iter
                  (local.get 0)))))
          (i32.const -1)))
      (local.set 4
        (local.get 4))
      (loop ;; label = @2
        (f64.store
          (local.tee 7
            (i32.add
              (i32.load offset=416
                (local.get 2))
              (local.tee 8
                (i32.shl
                  (local.tee 4
                    (local.get 4))
                  (i32.const 3)))))
          (f64.mul
            (f64.load
              (local.get 7))
            (f64.const 0x1.ccccccccccccdp-1 (;=0.9;))))
        (local.set 5
          (i32.load offset=380
            (local.tee 7
              (i32.load
                (local.get 0)))))
        (block ;; label = @3
          (block ;; label = @4
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (i32.load8_u offset=56
                    (local.tee 3
                      (i32.add
                        (i32.load offset=344
                          (local.get 7))
                        (i32.mul
                          (local.get 4)
                          (i32.const 80)))))))
              (local.set 9
                (f64.const 0x1.ccp+6 (;=115;)))
              (br 1 (;@4;)))
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (f64.gt
                    (f64.load offset=48
                      (local.get 3))
                    (f64.const 0x0p+0 (;=0;)))))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (f64.ne
                    (local.tee 9
                      (f64.load offset=72
                        (local.get 3)))
                    (f64.const 0x1p+0 (;=1;))))
                (local.set 9
                  (f64.const 0x1.54p+6 (;=85;)))
                (br 2 (;@4;)))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (f64.ne
                    (local.get 9)
                    (f64.const 0x1p+1 (;=2;))))
                (local.set 9
                  (f64.const 0x1.7cp+6 (;=95;)))
                (br 2 (;@4;)))
              (br_if 2 (;@3;)
                (f64.ne
                  (local.get 9)
                  (f64.const 0x1.8p+1 (;=3;))))
              (local.set 9
                (f64.const 0x1.a4p+6 (;=105;)))
              (br 1 (;@4;)))
            (local.set 9
              (call $oxitortoise_scale_color
                (f64.const 0x1.04p+6 (;=65;))
                (f64.load
                  (i32.add
                    (i32.load offset=416
                      (local.get 7))
                    (local.get 8)))
                (f64.const 0x1.999999999999ap-4 (;=0.1;))
                (f64.const 0x1.4p+2 (;=5;)))))
          (f64.store
            (i32.add
              (local.get 5)
              (local.get 8))
            (local.get 9)))
        (i32.store8
          (local.tee 4
            (i32.add
              (i32.load offset=56
                (local.get 0))
              (local.get 4)))
          (i32.or
            (i32.load8_u
              (local.get 4))
            (i32.const 1)))
        (local.set 4
          (local.tee 7
            (call $oxitortoise_next_patch_from_iter
              (local.get 10))))
        (br_if 0 (;@2;)
          (i32.ne
            (local.get 7)
            (i32.const -1)))))
    (call $oxitortoise_drop_patch_iter
      (local.get 10))
    (call $oxitortoise_advance_tick
      (local.get 2))
    (f64.store offset=8
      (local.get 0)
      (call $oxitortoise_get_ticks
        (local.get 2)))
    (global.set $__stack_pointer
      (i32.add
        (local.get 1)
        (i32.const 128)))
  )
  (func $shim_go (;30;) (type 8) (param i32 i32)
    (call $go
      (local.get 0))
  )
  (@custom ".debug_loc" (after code) "\ff\ff\ff\ff\06\00\00\00\00\00\00\00\04\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\0c\00\00\00\ee\00\00\00\06\00\ed\00\00#\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\11\00\00\00\13\00\00\00\04\00\ed\02\00\9f\13\00\00\004\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\11\00\00\00\13\00\00\00\04\00\ed\02\00\9f\13\00\00\00\04\01\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\c5\00\00\00\c8\00\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\11\00\00\00\13\00\00\00\04\00\ed\02\00\9f\13\00\00\004\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\e8\00\00\00\ed\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00,\00\00\00.\00\00\00\04\00\ed\02\00\9f.\00\00\00\04\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\00\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00!\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\00\00\00\00\cf\00\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\af\00\00\00\c0\00\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0c\01\00\00\bb\00\00\00\be\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00 \00\00\00*\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\002\00\00\00\89\00\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\00\00\00\00\9f\02\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00Q\00\00\00E\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\c6\00\00\00\d7\00\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\d2\00\00\00\d5\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\dc\00\00\00\9f\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\e1\00\00\00\eb\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\f3\00\00\00E\01\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\82\01\00\00\93\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\8e\01\00\00\91\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\98\01\00\00\9f\02\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\9d\01\00\00\a7\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00\af\01\00\00\04\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00>\02\00\00O\02\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00J\02\00\00M\02\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\dd\01\00\00T\02\00\00\9f\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00\00\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00!\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00\00\00\00\00\d0\00\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff~\04\00\00\af\00\00\00\c1\00\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00 \00\00\00*\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\002\00\00\00\89\00\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\00\00\00\00\a2\02\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00Q\00\00\00F\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\c6\00\00\00\d8\00\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\dd\00\00\00\a2\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\e2\00\00\00\ec\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\f4\00\00\00F\01\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\83\01\00\00\95\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\9a\01\00\00\a2\02\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\9f\01\00\00\a9\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00\b1\01\00\00\06\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00@\02\00\00R\02\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffP\05\00\00W\02\00\00\a2\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\1b\00\00\00\f3\04\00\00\06\00\ed\00\00#\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\22\00\00\00\0d\05\00\00\04\00\ed\00\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00y\00\00\00{\00\00\00\04\00\ed\02\01\9f{\00\00\00(\01\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\9b\00\00\00\05\01\00\00\03\00\11\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\f6\00\00\00\f8\00\00\00\06\00\ed\02\00\9f\93\04\f8\00\00\00\02\01\00\00\06\00\ed\00\06\9f\93\04\02\01\00\00\03\01\00\00\0c\00\ed\00\06\9f\93\04\ed\02\01\9f\93\04\03\01\00\00\05\01\00\00\06\00\ed\00\06\9f\93\04\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\19\01\00\00\1b\01\00\00\04\00\ed\02\00\9f\1b\01\00\00\0d\05\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00J\01\00\00L\01\00\00\04\00\ed\02\01\9fL\01\00\00\ff\03\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\cd\01\00\00\cf\01\00\00\04\00\ed\02\02\9f\cf\01\00\00o\02\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\ec\01\00\00\e1\04\00\00\04\00\ed\00\0b\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\f4\01\00\006\04\00\00\04\00\ed\00\0a\9f}\04\00\00\ad\04\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00b\02\00\00l\02\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\f3\02\00\00\fd\02\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\7f\03\00\00\89\03\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\cf\03\00\00\d0\03\00\00\04\00\ed\02\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\e0\03\00\00\e2\03\00\00\04\00\ed\02\00\9f\e2\03\00\00\ff\03\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\e0\03\00\00\e2\03\00\00\04\00\ed\02\00\9f\e2\03\00\00\e1\04\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\90\04\00\00\93\04\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\e0\03\00\00\e2\03\00\00\04\00\ed\02\00\9f\e2\03\00\00\ff\03\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\b3\04\00\00\b8\04\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\f7\03\00\00\f9\03\00\00\04\00\ed\02\00\9f\f9\03\00\00\e1\04\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f4\07\00\00\d6\04\00\00\d8\04\00\00\04\00\ed\02\00\9f\d8\04\00\00\e1\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\1d\00\00\00p\05\00\00\06\00\ed\00\00#\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00$\00\00\00\8a\05\00\00\04\00\ed\00\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\002\00\00\004\00\00\00\04\00\ed\02\01\9f4\00\00\00.\04\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00R\00\00\00w\00\00\00\03\00\11\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00`\00\00\00b\00\00\00\04\00\ed\02\00\9fb\00\00\00\8e\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\7f\00\00\00\c1\03\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\87\00\00\00\c1\03\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\e1\00\00\00\c0\01\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00s\01\00\00v\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00v\01\00\00x\01\00\00\04\00\ed\02\00\9fx\01\00\00\c0\01\00\00\04\00\ed\00\09\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\fc\01\00\00\8e\02\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00L\02\00\00N\02\00\00\04\00\ed\02\00\9fN\02\00\00\8e\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\a5\02\00\00\a6\02\00\00\04\00\ed\02\02\9f\c5\02\00\00\c6\02\00\00\04\00\ed\02\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\d5\03\00\00\d7\03\00\00\06\00\ed\02\00\9f\93\04\d7\03\00\00\e0\03\00\00\06\00\ed\00\07\9f\93\04\e0\03\00\00\e1\03\00\00\0c\00\ed\00\07\9f\93\04\ed\02\01\9f\93\04\e1\03\00\00\e3\03\00\00\06\00\ed\00\07\9f\93\04\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\1b\04\00\00\1d\04\00\00\04\00\ed\02\00\9f\1d\04\00\00\8a\05\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\000\04\00\00|\04\00\00\03\00\11\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00@\04\00\00B\04\00\00\04\00\ed\02\00\9fB\04\00\00|\04\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00Y\04\00\00[\04\00\00\04\00\ed\02\00\9f[\04\00\00|\04\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00Y\04\00\00[\04\00\00\04\00\ed\02\00\9f[\04\00\00^\05\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00\0d\05\00\00\10\05\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00Y\04\00\00[\04\00\00\04\00\ed\02\00\9f[\04\00\00|\04\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\000\05\00\005\05\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00t\04\00\00v\04\00\00\04\00\ed\02\00\9fv\04\00\00^\05\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\0e\0d\00\00S\05\00\00U\05\00\00\04\00\ed\02\00\9fU\05\00\00^\05\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00")
  (@custom ".debug_abbrev" (after code) "\01\11\01%\0e\13\05\03\0e\10\17\1b\0e\11\01U\17\00\00\02\0f\00I\13\00\00\03\16\00I\13\03\0e:\0b;\0b\00\00\04\13\01\0b\0b:\0b;\0b\00\00\05\0d\00\03\0eI\13:\0b;\0b8\0b\00\00\06\01\01I\13\00\00\07!\00I\137\0b\00\00\08$\00\03\0e>\0b\0b\0b\00\00\09$\00\03\0e\0b\0b>\0b\00\00\0a\13\00\03\0e<\19\00\00\0b\17\01\0b\0b:\0b;\0b\00\00\0c\0f\00\00\00\0d.\01\03\0e:\0b;\0b'\19I\13 \0b\00\00\0e\05\00\03\0e:\0b;\0bI\13\00\00\0f4\00\03\0e:\0b;\0bI\13\00\00\10.\01\11\01\12\06@\18\97B\191\13\00\00\11\05\00\02\171\13\00\00\12\05\00\02\181\13\00\00\134\00\02\171\13\00\00\14\1d\011\13\11\01\12\06X\0bY\0bW\0b\00\00\15\05\00\1c\0d1\13\00\00\16\89\82\01\001\13\11\01\00\00\17.\01\03\0e:\0b;\0b'\19I\13<\19?\19\00\00\18\05\00I\13\00\00\19\05\001\13\00\00\1a4\00\02\181\13\00\00\1b.\01\03\0e:\0b;\0b'\19I\13?\19 \0b\00\00\1c.\01\11\01\12\06@\18\97B\19\03\0e:\0b;\0b'\19?\19\00\00\1d\05\00\02\17\03\0e:\0b;\0bI\13\00\00\1e\05\00\02\18\03\0e:\0b;\0bI\13\00\00\1f4\00\02\17\03\0e:\0b;\05I\13\00\00 \1d\011\13\11\01\12\06X\0bY\05W\0b\00\00!\05\00\1c\0f1\13\00\00\224\001\13\00\00#.\01\03\0e:\0b;\05'\19I\13?\19 \0b\00\00$\05\00\03\0e:\0b;\05I\13\00\00%4\00\03\0e:\0b;\05I\13\00\00&.\01\11\01\12\06@\18\97B\19\03\0e:\0b;\05'\19?\19\00\00'\05\00\02\17\03\0e:\0b;\05I\13\00\00(\05\00\02\18\03\0e:\0b;\05I\13\00\00).\01\03\0e:\0b;\0b'\19?\19 \0b\00\00*\0b\01\11\01\12\06\00\00+4\00\02\18\03\0e:\0b;\05I\13\00\00,\1d\001\13\11\01\12\06X\0bY\05W\0b\00\00-.\01\03\0e:\0b;\0b'\19<\19?\19\00\00\00")
  (@custom ".debug_info" (after code) "\14\16\00\00\04\00\00\00\00\00\04\01\b6\06\00\00\1d\00\e1\05\00\00\00\00\00\00'\01\00\00\00\00\00\00\00\00\00\00\02+\00\00\00\036\00\00\00\8e\06\00\00\02\aa\04P\02\a3\05M\05\00\00\83\00\00\00\02\a4\00\05\02\06\00\00\a8\00\00\00\02\a5\08\05\1f\05\00\00\04\01\00\00\02\a60\05B\00\00\004\01\00\00\02\a78\05\95\00\00\00\04\01\00\00\02\a8@\059\03\00\00\04\01\00\00\02\a9H\00\06\8f\00\00\00\07\a1\00\00\00\01\00\03\9a\00\00\00\04\01\00\00\01\ca\08L\03\00\00\08\01\09>\06\00\00\08\07\03\b3\00\00\00!\06\00\00\027\04(\023\05\a6\03\00\00\dc\00\00\00\024\00\05\e9\03\00\00\16\01\00\00\025\10\05.\02\00\00\04\01\00\00\026 \00\03\e7\00\00\00V\00\00\00\02\12\04\10\02\0f\05*\00\00\00\04\01\00\00\02\10\00\05\00\00\00\00\04\01\00\00\02\11\08\00\03\0f\01\00\00\e9\00\00\00\02\0c\08\fb\04\00\00\04\08\03!\01\00\00E\04\00\00\02\0d\06-\01\00\00\07\a1\00\00\00\0c\00\08U\03\00\00\06\01\08\cd\03\00\00\02\01\02@\01\00\00\03K\01\00\00{\06\00\00\02\ad\04\08\02\ab\05'\02\00\00\04\01\00\00\02\ac\00\00\02a\01\00\00\03l\01\00\00h\06\00\00\02\b0\04\08\02\ae\05\f7\03\00\00\04\01\00\00\02\af\00\00\03\88\01\00\00\1e\01\00\00\01\d4\08\81\00\00\00\07\04\02\94\01\00\00\03\9f\01\00\00\9a\06\00\00\02\a2\04p\02\9d\05M\05\00\00\83\00\00\00\02\9e\00\05\02\06\00\00\d4\01\00\00\02\9f\08\05\9a\04\00\00\04\01\00\00\02\a0X\05\a6\03\00\00\dc\00\00\00\02\a1`\00\03\df\01\00\00/\06\00\00\021\04P\02(\05\8c\03\00\00D\02\00\00\02)\00\05\96\05\00\00a\02\00\00\02*\08\05\be\04\00\00\16\01\00\00\02+\10\05M\02\00\00\04\01\00\00\02, \05\ea\03\00\00\16\01\00\00\02-(\05/\02\00\00\04\01\00\00\02.8\05\b3\03\00\004\01\00\00\02/@\05\a2\04\00\00\04\01\00\00\020H\00\03O\02\00\00\90\03\00\00\02\0a\03Z\02\00\00\15\01\00\00\01\d9\082\04\00\00\07\08\03O\02\00\00\d9\05\00\00\02\0b\03w\02\00\00\c8\05\00\00\02&\03\82\02\00\00\1f\01\00\00\01\bb\08\8a\00\00\00\05\04\02\8e\02\00\00\02\93\02\00\00\03\9e\02\00\00\15\05\00\00\02R\0a\15\05\00\00\02-\01\00\00\02\ad\02\00\00\03\b8\02\00\00G\05\00\00\02T\0aG\05\00\00\02\c2\02\00\00\03\cd\02\00\00\17\02\00\00\02P\0b8\02C\05\14\04\00\00\dd\02\00\00\02G\00\04\08\02D\05\9c\05\00\00M\03\00\00\02E\00\050\00\00\00\04\01\00\00\02F\00\00\05\ac\01\00\00\06\03\00\00\02K\00\040\02H\05\9c\05\00\00Y\03\00\00\02I\00\050\00\00\00e\03\00\00\02J,\00\05\a0\01\00\00/\03\00\00\02O\00\044\02L\05\9c\05\00\00|\03\00\00\02M\00\050\00\00\00\88\03\00\00\02N0\00\00\06-\01\00\00\07\a1\00\00\00\00\00\06-\01\00\00\07\a1\00\00\00,\00\02j\03\00\00\03u\03\00\00\0c\01\00\00\01\cf\08G\00\00\00\07\02\06-\01\00\00\07\a1\00\00\000\00\02\8f\00\00\00\02\92\03\00\00\0c\02\04\01\00\00\0d6\05\00\00\02g\a8\02\00\00\01\0e2\00\00\00\02g\bb\03\00\00\0f\0b\05\00\00\02h\8e\02\00\00\00\02\c0\03\00\00\03\cb\03\00\00:\00\00\00\02A\0a:\00\00\00\0d\ee\05\00\00\02p\92\03\00\00\01\0eA\05\00\00\02p\a8\02\00\00\0e\17\00\00\00\02p\82\02\00\00\00\10\06\00\00\00\04\01\00\00\07\ed\03\00\00\00\00\9fv\0c\00\00\11\00\00\00\00~\0c\00\00\12\04\ed\00\01\9f\89\0c\00\00\13\1e\00\00\00\c0\0c\00\00\13j\00\00\00\94\0c\00\00\13\96\00\00\00\9f\0c\00\00\13\e0\00\00\00\aa\0c\00\00\13\fe\00\00\00\b5\0c\00\00\14\98\03\00\00\12\00\00\00\05\00\00\00\02\d9\11\12\04\ed\00\00\9f\a4\03\00\00\13>\00\00\00\af\03\00\00\00\14\d0\03\00\00\17\00\00\00\08\00\00\00\02\dd'\11\b4\00\00\00\dc\03\00\00\15\01\e7\03\00\00\00\14\d0\03\00\00\1f\00\00\00\11\00\00\00\02\dc'\12\04\ed\00\03\9f\dc\03\00\00\15\00\e7\03\00\00\00\16\b7\04\00\00\e6\00\00\00\00\17;\02\00\00\02\97\04\01\00\00\18\04\01\00\00\18\04\01\00\00\18\04\01\00\00\18\04\01\00\00\00\10\0c\01\00\00\cf\00\00\00\04\ed\00\04\9f\14\06\00\00\11f\01\00\00 \06\00\00\19+\06\00\00\12\04\ed\00\02\9f6\06\00\00\11*\01\00\00A\06\00\00\1a\02\91 L\06\00\00\1a\02\91\18x\06\00\00\13H\01\00\00W\06\00\00\13\84\01\00\00b\06\00\00\13\a2\01\00\00m\06\00\00\14\d0\03\00\00\bb\01\00\00\0a\00\00\00\02\fb'\15\02\e7\03\00\00\00\16\83\05\00\00+\01\00\00\16\94\05\00\00g\01\00\00\16\b4\05\00\00\7f\01\00\00\16\c5\05\00\00\8e\01\00\00\16\c5\05\00\00\a0\01\00\00\16\d6\05\00\00\b9\01\00\00\00\17\84\04\00\00\02\93\04\01\00\00\18\04\01\00\00\00\17P\04\00\00\02\91\dc\00\00\00\18\a8\02\00\00\18\dc\00\00\00\18\04\01\00\00\18\04\01\00\00\00\17\ba\03\00\00\02|4\01\00\00\18\0f\01\00\00\00\17$\05\00\00\02}\04\01\00\00\18\04\01\00\00\00\17\ef\00\00\00\02\92l\02\00\00\18\a8\02\00\00\18\ec\05\00\00\00\03\f7\05\00\00\a0\00\00\00\02\17\04\08\02\14\05*\00\00\00}\01\00\00\02\15\00\05\00\00\00\00}\01\00\00\02\16\04\00\1b\e9\04\00\00\02\f0\04\01\00\00\01\0eA\05\00\00\02\f0\a8\02\00\00\0e\a6\03\00\00\02\f0\dc\00\00\00\0e\9a\04\00\00\02\f0\04\01\00\00\0e\f5\04\00\00\02\f0\04\01\00\00\0f\a1\05\00\00\02\f2\dc\00\00\00\0fw\04\00\00\02\f1\04\01\00\00\0f`\05\00\00\02\f9l\02\00\00\0ft\06\00\00\02\fb\5c\01\00\00\0fq\00\00\00\02\f8\ec\05\00\00\00\1c\dd\01\00\00\9f\02\00\00\04\ed\00\03\9f\f0\03\00\00\02\ff\1d\fc\01\00\00A\05\00\00\02\ff\a8\02\00\00\0e\a6\03\00\00\02\ff\dc\00\00\00\1e\04\ed\00\02\9f\9a\04\00\00\02\ff\93\03\00\00\1ft\02\00\00\b9\05\00\00\02\00\01\04\01\00\00\1f\0a\03\00\00\c1\00\00\00\02\01\01\04\01\00\00\1f\a0\03\00\00\db\00\00\00\02\02\01\04\01\00\00 \14\06\00\00\06\02\00\00\b3\00\00\00\02\00\01\19\11\1a\02\00\00 \06\00\00\11\c0\01\00\006\06\00\00!\00A\06\00\00\1a\03\91\d0\00L\06\00\00\13\de\01\00\00W\06\00\00\138\02\00\00b\06\00\00\13V\02\00\00m\06\00\00\14\d0\03\00\00\a3\02\00\00\0a\00\00\00\02\fb'\15\02\e7\03\00\00\00\00 \14\06\00\00\c7\02\00\00\ae\00\00\00\02\01\01\19\11\92\02\00\006\06\00\00!\80\80\80\80\80\80\a0\a3@A\06\00\00\1a\03\91\d0\00L\06\00\00\13\b0\02\00\00W\06\00\00\13\ce\02\00\00b\06\00\00\13\ec\02\00\00m\06\00\00\14\d0\03\00\00_\03\00\00\0a\00\00\00\02\fb'\15\02\e7\03\00\00\00\00 \14\06\00\00\83\03\00\00\ae\00\00\00\02\02\01\18\11(\03\00\006\06\00\00!\80\80\80\80\80\80\a0\a3\c0\01A\06\00\00\1a\03\91\d0\00L\06\00\00\13F\03\00\00W\06\00\00\13d\03\00\00b\06\00\00\13\82\03\00\00m\06\00\00\14\d0\03\00\00\1b\04\00\00\0a\00\00\00\02\fb'\15\02\e7\03\00\00\00\00\16\83\05\00\00\0d\02\00\00\16\94\05\00\00L\02\00\00\16\b4\05\00\00d\02\00\00\16\c5\05\00\00s\02\00\00\16\c5\05\00\00\85\02\00\00\16\d6\05\00\00\a1\02\00\00\16\83\05\00\00\ce\02\00\00\16\94\05\00\00\08\03\00\00\16\b4\05\00\00 \03\00\00\16\c5\05\00\00/\03\00\00\16\c5\05\00\00A\03\00\00\16\d6\05\00\00]\03\00\00\16\83\05\00\00\8a\03\00\00\16\94\05\00\00\c7\03\00\00\16\b4\05\00\00\df\03\00\00\16\c5\05\00\00\ee\03\00\00\16\c5\05\00\00\00\04\00\00\16\d6\05\00\00\19\04\00\00\16\83\05\00\00k\04\00\00\00\10~\04\00\00\d0\00\00\00\04\ed\00\04\9fu\09\00\00\11\fa\03\00\00\82\09\00\00\19\8e\09\00\00\12\04\ed\00\02\9f\9a\09\00\00\11\be\03\00\00\a6\09\00\00\1a\02\91 \b2\09\00\00\1a\02\91\18\d6\09\00\00\13\dc\03\00\00\be\09\00\00\13\18\04\00\00\ca\09\00\00\22\e2\09\00\00 \d0\03\00\00-\05\00\00\0b\00\00\00\02\19\01'\15\00\e7\03\00\00\00\16\83\05\00\00\9d\04\00\00\16\94\05\00\00\d9\04\00\00\16\b4\05\00\00\f1\04\00\00\16\c5\05\00\00\00\05\00\00\16\c5\05\00\00\12\05\00\00\16\d6\05\00\00+\05\00\00\00#\d5\04\00\00\02\0e\01\04\01\00\00\01$A\05\00\00\02\0e\01\a8\02\00\00$\a6\03\00\00\02\0e\01\dc\00\00\00$\9a\04\00\00\02\0e\01\04\01\00\00$\f5\04\00\00\02\0e\01\04\01\00\00%\a1\05\00\00\02\10\01\dc\00\00\00%w\04\00\00\02\0f\01\04\01\00\00%`\05\00\00\02\17\01l\02\00\00%q\00\00\00\02\16\01\ec\05\00\00%\a7\06\00\00\02\19\01&\00\00\00\00&P\05\00\00\a2\02\00\00\04\ed\00\03\9f\8e\00\00\00\02\1d\01'r\04\00\00A\05\00\00\02\1d\01\a8\02\00\00$\a6\03\00\00\02\1d\01\dc\00\00\00(\04\ed\00\02\9f\9a\04\00\00\02\1d\01\93\03\00\00\1f\cc\04\00\00\ad\05\00\00\02\1e\01\04\01\00\00\1fD\05\00\00\b5\00\00\00\02\1f\01\04\01\00\00\1f\bc\05\00\00\d0\00\00\00\02 \01\04\01\00\00 u\09\00\00y\05\00\00\b4\00\00\00\02\1e\01\16\11\90\04\00\00\82\09\00\00\116\04\00\00\9a\09\00\00!\00\a6\09\00\00\1a\03\91\d0\00\b2\09\00\00\13T\04\00\00\be\09\00\00\13\ae\04\00\00\ca\09\00\00 \d0\03\00\00\16\06\00\00\0b\00\00\00\02\19\01'\15\00\e7\03\00\00\00\00 u\09\00\00;\06\00\00\af\00\00\00\02\1f\01\16\11\ea\04\00\00\9a\09\00\00!\80\80\80\80\80\80\a0\a3@\a6\09\00\00\1a\03\91\d0\00\b2\09\00\00\13\08\05\00\00\be\09\00\00\13&\05\00\00\ca\09\00\00 \d0\03\00\00\d3\06\00\00\0b\00\00\00\02\19\01'\15\00\e7\03\00\00\00\00 u\09\00\00\f8\06\00\00\af\00\00\00\02 \01\15\11b\05\00\00\9a\09\00\00!\80\80\80\80\80\80\a0\a3\c0\01\a6\09\00\00\1a\03\91\d0\00\b2\09\00\00\13\80\05\00\00\be\09\00\00\13\9e\05\00\00\ca\09\00\00 \d0\03\00\00\90\07\00\00\0b\00\00\00\02\19\01'\15\00\e7\03\00\00\00\00\16\83\05\00\00\80\05\00\00\16\94\05\00\00\bf\05\00\00\16\b4\05\00\00\d7\05\00\00\16\c5\05\00\00\e6\05\00\00\16\c5\05\00\00\f8\05\00\00\16\d6\05\00\00\14\06\00\00\16\83\05\00\00B\06\00\00\16\94\05\00\00|\06\00\00\16\b4\05\00\00\94\06\00\00\16\c5\05\00\00\a3\06\00\00\16\c5\05\00\00\b5\06\00\00\16\d6\05\00\00\d1\06\00\00\16\83\05\00\00\ff\06\00\00\16\94\05\00\00<\07\00\00\16\b4\05\00\00T\07\00\00\16\c5\05\00\00c\07\00\00\16\c5\05\00\00u\07\00\00\16\d6\05\00\00\8e\07\00\00\16\83\05\00\00\e1\07\00\00\00\0d\0c\06\00\00\02l\92\03\00\00\01\0eA\05\00\00\02l\a8\02\00\00\0e\17\00\00\00\02l\82\02\00\00\00\0dS\02\00\00\02x\04\01\00\00\01\0eA\05\00\00\02x\a8\02\00\00\00\0df\02\00\00\02t\04\01\00\00\01\0eA\05\00\00\02t\a8\02\00\00\00)$\04\00\00\02\d8\01\0e2\00\00\00\02\d8\bb\03\00\00\0e`\05\00\00\02\d8l\02\00\00\0fA\05\00\00\02\d9\a8\02\00\00\0ft\06\00\00\02\de\5c\01\00\00\0f\87\06\00\00\02\dd;\01\00\00\0f\a7\06\00\00\02\dc&\00\00\00\0f\06\02\00\00\02\da\bd\02\00\00\00&\f4\07\00\00\0d\05\00\00\04\ed\00\01\9f_\03\00\00\02,\01(\04\ed\00\00\9f2\00\00\00\02,\01\bb\03\00\00\1f\da\05\00\00\06\02\00\00\02-\01\bd\02\00\00\1f\fa\05\00\00A\05\00\00\02.\01\a8\02\00\00 \98\03\00\00\0f\08\00\00\07\00\00\00\02.\01\11\12\04\ed\00\00\9f\a4\03\00\00\00*\1e\08\00\00\e5\00\00\00\1f\18\06\00\004\03\00\00\025\017\10\00\00\1fa\06\00\00\c9\04\00\00\02;\01]\10\00\00*\8d\08\00\00I\00\00\00%\15\06\00\00\02?\01\8f\01\00\00%\02\06\00\00\02@\01\0d\16\00\00 #\0c\00\00\8d\08\00\00\0e\00\00\00\02?\010\11D\06\00\00:\0c\00\00\00\00\00*\03\09\00\00\dc\03\00\00\1f\b7\06\00\004\03\00\00\02L\01\c0\10\00\00\1f\e9\08\00\00\19\04\00\00\02M\01l\02\00\00* \09\00\00\a2\03\00\00+\03\91\d0\01\a6\03\00\00\02Q\01\dc\00\00\00\1f\e3\06\00\00,\04\00\00\02P\01&\00\00\00\1f\0f\07\00\00\02\05\00\00\02R\01\04\01\00\00 \d0\03\00\00+\09\00\00\0f\00\00\00\02P\01(\15\00\e7\03\00\00\00*\d8\09\00\00\ea\02\00\00\1f;\07\00\00\5c\02\00\00\02]\01\04\01\00\00\1fY\07\00\00o\02\00\00\02\5c\01\04\01\00\00,F\0c\00\00\d8\09\00\00\08\00\00\00\02]\01\17,^\0c\00\00\e0\09\00\00\08\00\00\00\02\5c\01\17*\e8\09\00\00\8a\00\00\00\1f\85\07\00\00\02\05\00\00\02a\01\04\01\00\00\00*s\0a\00\00\91\00\00\00\1f\a3\07\00\00\02\05\00\00\02i\01\04\01\00\00\00*\05\0b\00\00\8b\00\00\00\1f\c1\07\00\00\02\05\00\00\02q\01\04\01\00\00\00*\bd\0b\00\00\0a\00\00\00\1f\df\07\00\00\0c\00\00\00\02z\01}\01\00\00\00 v\0c\00\00\cc\0b\00\00\f6\00\00\00\02\80\01\05\13)\08\00\00\94\0c\00\00\13U\08\00\00\9f\0c\00\00\13\9f\08\00\00\aa\0c\00\00\13\bd\08\00\00\b5\0c\00\00\14\98\03\00\00\cf\0b\00\00\05\00\00\00\02\d9\11\13\fd\07\00\00\af\03\00\00\00\14\d0\03\00\00\d4\0b\00\00\08\00\00\00\02\dd'\11s\08\00\00\dc\03\00\00\15\01\e7\03\00\00\00\14\d0\03\00\00\dc\0b\00\00\0c\00\00\00\02\dc'\12\04\ed\00\05\9f\dc\03\00\00\15\00\e7\03\00\00\00\00\00\00\00\16\f9\0f\00\00\1e\08\00\00\16\06\10\00\00&\08\00\00\16\17\10\00\00m\08\00\00\16L\10\00\00u\08\00\00\16L\10\00\00\e4\08\00\00\16\a2\10\00\00\03\09\00\00\16\af\10\00\00\0d\09\00\00\16\d5\10\00\00\15\09\00\00\16\e6\10\00\00\c1\09\00\00\16\e6\10\00\00V\0a\00\00\16\e6\10\00\00\e7\0a\00\00\16\e6\10\00\00s\0b\00\00\16\fc\10\00\00\c3\0b\00\00\16\b7\04\00\00\9f\0c\00\00\16\d5\10\00\00\ca\0c\00\00\16\12\11\00\00\df\0c\00\00\16\1f\11\00\00\e7\0c\00\00\16,\11\00\00\f1\0c\00\00\00-\d3\03\00\00\02\7f\18\bb\03\00\00\00\17w\05\00\00\02\9ba\02\00\00\18\bb\03\00\00\00\17\b9\01\00\00\02\847\10\00\00\18\bb\03\00\00\18a\02\00\00\18O\02\00\00\18\dc\00\00\00\00\02<\10\00\00\03G\10\00\00\f7\01\00\00\02;\0a\f7\01\00\00\17\de\02\00\00\02\88]\10\00\00\187\10\00\00\00\03h\10\00\00\d0\05\00\00\02$\0b\08\02\1e\05\02\00\00\00x\10\00\00\02\22\00\04\08\02\1f\05\11\00\00\00}\01\00\00\02 \00\05\af\03\00\00j\03\00\00\02!\04\00\05,\00\00\00O\02\00\00\02#\00\00-\1c\03\00\00\02\89\187\10\00\00\00\17\9b\02\00\00\02\8b\c0\10\00\00\18\bb\03\00\00\00\02\c5\10\00\00\03\d0\10\00\00\e9\01\00\00\02=\0a\e9\01\00\00\17\bd\02\00\00\02\8dl\02\00\00\18\c0\10\00\00\00\17e\03\00\00\02\90\04\01\00\00\18\dc\00\00\00\18\dc\00\00\00\00\17\5c\00\00\00\02\99}\01\00\00\18\bb\03\00\00\18}\01\00\00\00-\00\03\00\00\02\8e\18\c0\10\00\00\00-m\01\00\00\02\80\18\a8\02\00\00\00\17\85\01\00\00\02\81\04\01\00\00\18\a8\02\00\00\00&\02\0d\00\00\0a\00\00\00\07\ed\03\00\00\00\00\9fZ\03\00\00\02\8b\01(\04\ed\00\00\9f2\00\00\00\02\8b\01\bb\03\00\00$\9b\01\00\00\02\8b\01\92\03\00\00\16\cc\0c\00\00\0b\0d\00\00\00&\0e\0d\00\00\8a\05\00\00\04\ed\00\01\9f\9f\03\00\00\02\8f\01(\04\ed\00\00\9f2\00\00\00\02\8f\01\bb\03\00\00\1f\15\09\00\00\06\02\00\00\02\90\01\bd\02\00\00\1f5\09\00\00A\05\00\00\02\91\01\a8\02\00\00\1fS\09\00\004\03\00\00\02\94\017\10\00\00\1f\e2\0a\00\00\c9\04\00\00\02\95\01]\10\00\00\1f8\0b\00\00\11\03\00\00\02\f7\01\c0\10\00\00\1f\99\0c\00\00\19\04\00\00\02\f8\01l\02\00\00 \98\03\00\00+\0d\00\00\07\00\00\00\02\91\01\11\12\04\ed\00\00\9f\a4\03\00\00\00*^\0d\00\00q\03\00\00+\03\91\e0\00\a2\03\00\00\02\ea\01\dc\00\00\00\1f\9c\09\00\00\ae\06\00\00\02\98\01\8f\01\00\00\1f\c8\09\00\00\9a\04\00\00\02\9f\01\93\03\00\00\1f\e6\09\00\00\a6\03\00\00\02\9e\01\12\16\00\00 #\0c\00\00^\0d\00\00\0e\00\00\00\02\98\01+\11\7f\09\00\00:\0c\00\00\00*\bd\0d\00\00\11\01\00\00\1f\04\0a\00\00i\05\00\00\02\a5\01l\02\00\00\1f\22\0a\00\00\b2\04\00\00\02\b9\01\5c\01\00\00\1f@\0a\00\00\f7\03\00\00\02\ba\01\04\01\00\00%\a7\04\00\00\02\a6\01&\00\00\00 \d0\03\00\00\ef\0d\00\00\0d\00\00\00\02\a6\01.\15\00\e7\03\00\00\00 \d0\03\00\00s\0e\00\00\0c\00\00\00\02\b9\01/\15\02\e7\03\00\00\00\00*\d7\0e\00\00\c5\00\00\00\1fl\0a\00\00i\05\00\00\02\c3\01l\02\00\00%\a7\04\00\00\02\c4\01&\00\00\00 \d0\03\00\00\0a\0f\00\00\0d\00\00\00\02\c4\01.\15\00\e7\03\00\00\00*N\0f\00\00N\00\00\00\1f\8a\0a\00\00\b2\04\00\00\02\cf\01\5c\01\00\00 \d0\03\00\00N\0f\00\00\0a\00\00\00\02\cf\010\15\02\e7\03\00\00\00\00\00*\a1\0f\00\00\aa\00\00\00+\03\91\e0\00\a1\05\00\00\02\e3\01\dc\00\00\00\1f\b6\0a\00\00\a9\00\00\00\02\db\01\04\01\00\00\00\00*<\11\00\00\1d\01\00\00\1f\81\0b\00\00t\06\00\00\02\fb\01\5c\01\00\00 \d0\03\00\00<\11\00\00\0e\00\00\00\02\fb\01(\11d\0b\00\00\e7\03\00\00\00 v\0c\00\00b\11\00\00\f7\00\00\00\02\ff\01\03\13\d9\0b\00\00\94\0c\00\00\13\05\0c\00\00\9f\0c\00\00\13O\0c\00\00\aa\0c\00\00\13m\0c\00\00\b5\0c\00\00\14\98\03\00\00b\11\00\00\05\00\00\00\02\d9\11\13\ad\0b\00\00\af\03\00\00\00\14\d0\03\00\00g\11\00\00\08\00\00\00\02\dd'\11#\0c\00\00\dc\03\00\00\15\01\e7\03\00\00\00\14\d0\03\00\00o\11\00\00\11\00\00\00\02\dc'\12\04\ed\00\07\9f\dc\03\00\00\15\00\e7\03\00\00\00\00\00\16q\15\00\00@\0d\00\00\16L\10\00\00H\0d\00\00\16,\11\00\00\7f\0d\00\00\16\c5\05\00\00\a7\0d\00\00\16\c5\05\00\00\d1\0d\00\00\16\d6\05\00\00\ed\0d\00\00\16\83\05\00\00O\0e\00\00\16\84\06\00\00\ce\0e\00\00\16\c5\05\00\00\eb\0e\00\00\16\d6\05\00\00\08\0f\00\00\16\83\05\00\00H\0f\00\00\16\ef\09\00\00\9c\0f\00\00\16\fc\10\00\00\a7\0f\00\00\16\83\05\00\00\ba\0f\00\00\16\fc\10\00\00\c7\0f\00\00\16\83\05\00\00\da\0f\00\00\16\94\05\00\00!\10\00\00\16\b4\05\00\00.\10\00\00\16\83\05\00\00H\10\00\00\16\94\05\00\00\8b\10\00\00\16\b4\05\00\00\98\10\00\00\16L\10\00\00\de\10\00\00\16\a2\10\00\00\fb\10\00\00\16\82\15\00\00\1f\11\00\00\16\af\10\00\00)\11\00\00\16\d5\10\00\001\11\00\00\16\b7\04\00\006\12\00\00\16\d5\10\00\00a\12\00\00\16\12\11\00\00v\12\00\00\16\c1\15\00\00~\12\00\00\16,\11\00\00\88\12\00\00\00\17y\02\00\00\02\867\10\00\00\18\bb\03\00\00\00-R\06\00\00\02\95\18\a8\02\00\00\18\99\15\00\00\18\04\01\00\00\00\03\a4\15\00\00\d4\01\00\00\02\1c\04\02\02\19\05\17\00\00\00\8f\00\00\00\02\1a\00\05\22\00\00\00\8f\00\00\00\02\1b\01\00-\00\04\00\00\02\82\18\a8\02\00\00\00&\99\12\00\00\0a\00\00\00\07\ed\03\00\00\00\00\9f\9a\03\00\00\02\08\02(\04\ed\00\00\9f2\00\00\00\02\08\02\bb\03\00\00$\9b\01\00\00\02\08\02\92\03\00\00\16|\11\00\00\a2\12\00\00\00\02\d4\01\00\00\02\dc\00\00\00\00")
  (@custom ".debug_ranges" (after code) "\06\00\00\00\0a\01\00\00\0c\01\00\00\db\01\00\00\dd\01\00\00|\04\00\00~\04\00\00N\05\00\00P\05\00\00\f2\07\00\00\f4\07\00\00\01\0d\00\00\02\0d\00\00\0c\0d\00\00\0e\0d\00\00\98\12\00\00\99\12\00\00\a3\12\00\00\00\00\00\00\00\00\00\00")
  (@custom ".debug_str" (after code) "y\00gen_index\00rand_index\00buffer_idx\00field_idx\00raw\00v\00context\00Context\00nest\00unsigned short\00Point\00oxitortoise_next_int\00point_ahead_int\00unsigned int\00uphill_nest_scent\00PointInt\00rand_result\00scent_right\00chemical_right\00scent_left\00chemical_left\00Float\00oxitortoise_patch_at\00uint8_t\00uint16_t\00uint64_t\00uint32_t\00/home/anderiux/data/NetLogo/oxitortoise/oxitortoise/bench/models/ants\00oxitortoise_reset_ticks\00oxitortoise_get_ticks\00args\00patch_flags\00turtle_flags\00oxitortoise_create_turtles\00AgentFieldDescriptor\00PatchIterator\00TurtleIterator\00dirty_aggregator\00DirtyAggregator\00pcolor\00plabel_color\00oxitortoise_scale_color\00world_to_max_pycor\00world_to_max_pxcor\00oxitortoise_make_all_turtles_iter\00oxitortoise_make_all_patches_iter\00oxitortoise_next_patch_from_iter\00oxitortoise_next_turtle_from_iter\00oxitortoise_drop_patch_iter\00oxitortoise_drop_turtle_iter\00food_source_number\00unsigned char\00shim_setup\00oxitortoise_distance_euclidean_no_wrap\00who\00TurtleWho\00shim_go\00new_position\00gen\00hidden\00oxitortoise_is_nan\00_Bool\00oxitortoise_clear_all\00plabel\00uphill_chemical\00oxitortoise_advance_tick\00next_patch\00recolor_patch\00unsigned long long\00RustString\00oxitortoise_offset_distance_by_heading\00real_heading\00oxitortoise_normalize_heading\00size\00patch_here\00patch2_here\00shape_name\00next_turtle\00nest_scent_at_angle\00chemical_at_angle\00double\00distance\00workspace\00Workspace\00food\00oxitortoise_round\00context_to_world\00World\00occupancy_bitfield\00patch_id\00patch_here_id\00oxitortoise_get_default_turtle_breed\00_pad\00point_ahead\00scent_ahead\00chemical_ahead\00PatchId\00TurtleId\00BreedId\00model_code.c\00world_to_patch_data\00base_data\00world_to_turtle_data\00PatchBaseData\00TurtleBaseData\00__ARRAY_SIZE_TYPE__\00oxitortoise_diffuse_8\00PatchGroup2\00patch2\00PatchGroup1\00patch1\00PatchGroup0\00TurtleGroup0\00patch0\00turtle0\00clang version 21.0.0git (https:/github.com/llvm/llvm-project 0f0079c29da4b4d5bbd43dced1db9ad6c6d11008)\00")
  (@custom ".debug_line" (after code) "f\0a\00\00\04\00\80\00\00\00\01\01\01\fb\0e\0d\00\01\01\01\01\00\00\00\01\00\00\01/home/anderiux\00\00.installs/emsdk/upstream/emscripten/cache/sysroot/include/bits/alltypes.h\00\01\00\00model_code.c\00\00\00\00\00\04\02\00\05\02\06\00\00\00\03\d7\01\01\05E\0a\95\05\19\03\8b\7f<\05\09\03\09X\06\82\05E\06\03\eb\00\08\12\05\0e1\05\06\06X\03\a1~<\03\df\01\ac\03\a1~.\05\15\06\03\e1\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\9e~<\03\e2\01\ac\03\9e~.\05)\06\03\e4\01\d6\06\03\9c~<\03\e4\01\ac\03\9c~.\06\03\e6\01\ba\06\03\9a~<\03\e6\01\ac\03\9a~.\05A\06\03\eb\01\08.\06\03\95~<\05\14\03\eb\01\08 \03\95~f\05 \06\03\ed\01\d6\05\02\06X\05,<\05\01\06\c9\02\01\00\01\01\04\02\00\05\02\0c\01\00\00\03\ef\01\01\05=\0a\08=\05\17\06X\05\16\06\83\06\03\8e~\02:\01\05%\06\03\f4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f8~\08X\05E\03\8a\01\9e\05\11/\06\03\84~<\05\01\06\03\fd\01<\05\00\06\03\83~\ac\05\01\03\fd\01.\02\01\00\01\01\04\02\00\05\02\dd\01\00\00\03\fe\01\01\05<\0a\08\9f\06\03\80~X\05=\06\03\f1\01\90\05\17\06 \05\16\06\83\06\03\8e~\02=\01\05%\06\03\f4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f8~\08\82\05E\03\8a\01\9e\05\11/\06\03\84~<\05<\06\03\81\02t\06\03\ff}X\05=\06\03\f1\01\90\05\17\06 \05\16\06\83\06\03\8e~\028\01\05%\06\03\f4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f8~\08\82\05E\03\8a\01\9e\05\11/\06\03\84~<\05;\06\03\82\02t\06\03\fe}X\05=\06\03\f1\01\90\05\17\06 \05\16\06\83\06\03\8e~\02;\01\05%\06\03\f4\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f8~\08X\05E\03\8a\01\9e\05\11/\06\03\84~<\05\15\06\03\83\02t\05&\06\90\03\fd}\9e\05\16\06\03\84\02\08\90\05\00\06\03\fc}f\05\01\06\03\8c\02\ac\02\0d\00\01\01\04\02\00\05\02~\04\00\00\03\8d\02\01\05=\0a\08=\05\17\06X\05\16\06\83\06\03\f0}\02:\01\05%\06\03\92\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\da~\08X\05\11\03\a9\01\ac\06\03\e6}t\05\01\06\03\9b\02 \05\00\06\03\e5}\ac\05\01\03\9b\02.\02\01\00\01\01\04\02\00\05\02P\05\00\00\03\9c\02\01\05;\0a\08\9f\06\03\e2}X\05=\06\03\8f\02\90\05\17\06 \05\16\06\83\06\03\f0}\02=\01\05%\06\03\92\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\da~\08\82\05\11\03\a9\01\ac\06\03\e6}t\05;\06\03\9f\02X\06\03\e1}X\05=\06\03\8f\02\90\05\17\06 \05\16\06\83\06\03\f0}\028\01\05%\06\03\92\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\da~\08\82\05\11\03\a9\01\ac\06\03\e6}t\05:\06\03\a0\02X\06\03\e0}X\05=\06\03\8f\02\90\05\17\06 \05\16\06\83\06\03\f0}\02;\01\05%\06\03\92\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\da~\08X\05\11\03\a9\01\ac\06\03\e6}t\05\12\06\03\a1\02X\05 \06\90\03\df}\9e\05\13\06\03\a2\02\08\90\05\00\06\03\de}f\05\01\06\03\aa\02\ac\02\0d\00\01\01\04\02\00\05\02\f4\07\00\00\03\ab\02\01\05\19\0a\03\bc~\08\9e\05\02\03\c9\01t\05\04\88\a0\05\1a\d2\05\04\08$\05\1a~\05\19Q\05\1a\03y\c8\05\19\c1\05F\06\08t\05\03 \03\c4}<\05\09\06\03\ed\00J\05.\03\d3\01\d6\05\15\e7\06\03\bd}<\05\14\06\03\c2\02\c8\05#?\05\04\06\90\05B.\05\19\06\03w\d6\06\03\c4}\08<\03\bc\02J\05F\82\05\03 \06\03\0bJ\05\19\87\05\18\a0\06\03\b2}\82\05@\03\ce\02J\05\03 \03\b2}.\05&\06\03\d1\02J\05\09\03\a0~\ac\05F\03\df\01\e4\06\03\b0}J\05&\06\03\d1\02J\05F\f3\05&\d5\05F\bb\05\15\06J\05\1e\06\02Q\18\05\16\06<\05\1b\06\ef\05\10\06 \05\09\06\03\a4~<~\05O\03\ec\01\82\05\17\06\f2\05Y\02/\12\05O \05\17J\03\9f}\02*\01\05\13\06\03\e2\02\90\06\03\9e}J\05!\06\03\e3\02\ba\06\03\9d}<\05O\06\03\e9\02 \05q\06\08X\05O \05\17<\05Z\02-\12\05O \05\17J\03\97}\02(\01\05\13\06\03\ea\02\90\06\03\96}J\05!\06\03\eb\02\c8\06\03\95}<\05O\06\03\f1\02 \05p\06\08X\05O \05\17<\05Z\02*\12\05O \05\17J\03\8f}\02%\01\05\13\06\03\f2\02\90\06\03\8e}J\05!\06\03\f3\02\c8\06\03\8d}<\05\11\06\03\f9\02 \05$\06\f2\03\87}J\05\1d\06\03\fa\02\08t\05\15g\05\13\06 \03\85}<\05E\06\03\dd\01X\05\19\03\8b\7f<\05\09\03\09X\06\82\05E\06\03\eb\00\ba\05\0e?\05\06\06X\03\a1~<\03\df\01\ac\03\a1~.\05\15\06\03\e1\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\9e~<\03\e2\01\ac\03\9e~.\05)\06\03\e4\01\d6\06\03\9c~<\03\e4\01\ac\03\9c~.\06\03\e6\01\ba\06\03\9a~<\03\e6\01\ac\03\9a~.\05A\06\03\eb\01\08.\06\03\95~<\05\14\03\eb\01\08 \03\95~f\05 \06\03\ed\01\d6\05\02\06X\05,<\05\18\06\03\e1\00\c8\06\03\b2}\82\05@\03\ce\02\82\05\03 \06\035J\05\02\86\05\1d\83\05\1b\06\9e\05\01\06=\02\0d\00\01\01\04\02\05\02\0a\00\05\02\03\0d\00\00\03\8b\03\01\05\01\83\02\01\00\01\01\04\02\00\05\02\0e\0d\00\00\03\8e\03\01\05\19\0a\03\d9}\08\ba\05\18\03\ae\02t\05\19d\05\18\84\05E\06\08X\05\02 \03\ea|<\05\09\06\03\ed\00J\05J\03\ab\02\d6\05\1a/\05!\06t\05\07\9e\05\1e<\03\e7|<\06\03\9f\03X\06\03\e1|<\05\1f\06\03\9e\03X\05\1a@\05\00\06\03\de|t\05 \03\a2\03\08\c8\05C\06?\05\9e\01\06t\05\82\01t\05xf\05C.\05\1d<\05\09\06\03\cc}\08\82\05\15\03\b8\02\c8\05\1a\06\08 \03\d7|f\05\1f\06\03\ab\03\c8\05\00\06\03\d5|t\05\17\06\03\ae\03\ba\05/M\058\06\f2\05\11 \05\0ff\05%\06?\05\06\06\90\05D.\03\cc|\08J\05\09\06\03\f1\00 \05M\03\c8\02\ba\05#/\06\03\c6|<\05\12\06\03\bb\03\ac\05\1a\06 \03\c5|<\03\bb\03\ac\05\06\06L\06\03\c3|\02,\01\05C\06\03\c3\03\90\05\9e\01\06t\05\82\01t\05xf\05C.\05\1d<\05\09\06\03\ae}\08\90\05\15\03\d6\02\c8\05\09\06t\03\b9|<\05\1f\06\03\c9\03\c8\05/?\058\06\f2\05\11 \05\0ff\05\05\06=\06\03\b3|.\05\09\06\03\f1\00 \05N\03\de\02\9e\05\1c/\05\06\08?\06\03\ad|\02.\01\05\1f\06\03\db\03X\05-\83\05\18s\056=\05\0f\06 \05\0df\05\19\06w\05-\83\05\12s\056=\05\0f\06 \05\0df\05Q\06?\05\18\06t\05'\06\02=\13\05\08\06t\05.\06\91\057\06\f2\05\10 \05\0ef\03\9b|<\05Q\06\03\ea\03 \05\18\06t\05(\06\028\13\05\08\06t\05\07f\05\10\06/\06\03\94|\08\9e\05\22\06\03\ef\03 \05\03\06\90\05A.\03\91|\d6\05\18\06\03\96\03 \06\03\ea|\08.\03\96\03J\05Et\05\02 \06\03\db\00J\06\03\8f|\82\06\03\f4\03J\05\1f\06t\05\02<\05\1e\06\08[\05\17\a0\06\03\87|\82\05E\03\f9\03J\05\02 \03\87|.\05\09\06\03\f1\00J\05F\03\8a\03\d6\05\14K\05\19\03\ec|\08<\05\09\03\09X\06\82\05E\06\03\eb\00\08\12\05\0e1\05\06\06X\03\a1~<\03\df\01\ac\03\a1~.\05\15\06\03\e1\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\9e~<\03\e2\01\ac\03\9e~.\05)\06\03\e4\01\d6\06\03\9c~<\03\e4\01\ac\03\9c~.\06\03\e6\01\ba\06\03\9a~<\03\e6\01\ac\03\9a~.\05A\06\03\eb\01\08.\06\03\95~<\05\14\03\eb\01\08 \03\95~f\05 \06\03\ed\01\d6\05\02\06X\05,<\05\17\06\03\8c\02\c8\06\03\87|\82\05E\03\f9\03\82\05\02 \06R\85\05\1d\83\05\1b\06\9e\05\01\06=\02\0d\00\01\01\04\02\05\02\0a\00\05\02\9a\12\00\00\03\88\04\01\05\01\83\02\01\00\01\01")
  (@custom "target_features" (after code) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)
