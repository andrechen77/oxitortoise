(module $model_code.wasm
  (@dylink.0
    (mem-info)
  )
  (type (;0;) (func (param f64 f64 f64 f64) (result f64)))
  (type (;1;) (func (param i32 i32 i32 i32)))
  (type (;2;) (func (param f64) (result f64)))
  (type (;3;) (func (param i32 i32 i32 f64 f64)))
  (type (;4;) (func (param f64) (result i32)))
  (type (;5;) (func (param i32 i32) (result i32)))
  (type (;6;) (func (param i32)))
  (type (;7;) (func (param i32) (result i64)))
  (type (;8;) (func (param i32 i64 i64 i32) (result i32)))
  (type (;9;) (func (param i32 i32)))
  (type (;10;) (func (param i32) (result i32)))
  (type (;11;) (func (param i32 i32) (result f64)))
  (type (;12;) (func (param i32) (result f64)))
  (type (;13;) (func (param i32 f64)))
  (type (;14;) (func (param i32 i32 f64)))
  (type (;15;) (func))
  (type (;16;) (func (param i32 i32 f64 f64) (result f64)))
  (type (;17;) (func (param i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 0))
  (import "env" "__stack_pointer" (global $__stack_pointer (;0;) (mut i32)))
  (import "env" "__indirect_function_table" (table (;0;) 0 funcref))
  (import "env" "oxitortoise_scale_color" (func $oxitortoise_scale_color (;0;) (type 0)))
  (import "env" "oxitortoise_update_patch" (func $oxitortoise_update_patch (;1;) (type 1)))
  (import "env" "oxitortoise_normalize_heading" (func $oxitortoise_normalize_heading (;2;) (type 2)))
  (import "env" "oxitortoise_offset_distance_by_heading" (func $oxitortoise_offset_distance_by_heading (;3;) (type 3)))
  (import "env" "oxitortoise_is_nan" (func $oxitortoise_is_nan (;4;) (type 4)))
  (import "env" "oxitortoise_round" (func $oxitortoise_round (;5;) (type 2)))
  (import "env" "oxitortoise_patch_at" (func $oxitortoise_patch_at (;6;) (type 5)))
  (import "env" "oxitortoise_clear_all" (func $oxitortoise_clear_all (;7;) (type 6)))
  (import "env" "oxitortoise_get_default_turtle_breed" (func $oxitortoise_get_default_turtle_breed (;8;) (type 7)))
  (import "env" "oxitortoise_create_turtles" (func $oxitortoise_create_turtles (;9;) (type 8)))
  (import "env" "oxitortoise_next_turtle_from_iter" (func $oxitortoise_next_turtle_from_iter (;10;) (type 9)))
  (import "env" "oxitortoise_update_turtle" (func $oxitortoise_update_turtle (;11;) (type 1)))
  (import "env" "oxitortoise_drop_turtle_iter" (func $oxitortoise_drop_turtle_iter (;12;) (type 6)))
  (import "env" "oxitortoise_make_all_patches_iter" (func $oxitortoise_make_all_patches_iter (;13;) (type 10)))
  (import "env" "oxitortoise_next_patch_from_iter" (func $oxitortoise_next_patch_from_iter (;14;) (type 10)))
  (import "env" "oxitortoise_distance_euclidean_no_wrap" (func $oxitortoise_distance_euclidean_no_wrap (;15;) (type 11)))
  (import "env" "oxitortoise_next_int" (func $oxitortoise_next_int (;16;) (type 5)))
  (import "env" "oxitortoise_drop_patch_iter" (func $oxitortoise_drop_patch_iter (;17;) (type 6)))
  (import "env" "oxitortoise_reset_ticks" (func $oxitortoise_reset_ticks (;18;) (type 6)))
  (import "env" "oxitortoise_get_ticks" (func $oxitortoise_get_ticks (;19;) (type 12)))
  (import "env" "oxitortoise_update_tick" (func $oxitortoise_update_tick (;20;) (type 13)))
  (import "env" "oxitortoise_make_all_turtles_iter" (func $oxitortoise_make_all_turtles_iter (;21;) (type 10)))
  (import "env" "oxitortoise_diffuse_8" (func $oxitortoise_diffuse_8 (;22;) (type 14)))
  (import "env" "oxitortoise_advance_tick" (func $oxitortoise_advance_tick (;23;) (type 6)))
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
  (func $__wasm_call_ctors (;24;) (type 15))
  (func $recolor_patch (;25;) (type 9) (param i32 i32)
    (local i32 i32 i32 i32 f64)
    (local.set 2
      (i32.shl
        (local.get 1)
        (i32.const 3)))
    (local.set 4
      (i32.load offset=400
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
                    (i32.load offset=360
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
                (i32.load offset=440
                  (local.get 3))
                (local.get 2)))
            (f64.const 0x1.999999999999ap-4 (;=0.1;))
            (f64.const 0x1.4p+2 (;=5;)))))
      (f64.store
        (i32.add
          (local.get 4)
          (local.get 2))
        (local.get 6)))
    (call $oxitortoise_update_patch
      (i32.add
        (local.get 0)
        (i32.const 8))
      (local.get 3)
      (local.get 1)
      (i32.const 1))
  )
  (func $chemical_at_angle (;26;) (type 16) (param i32 i32 f64 f64) (result f64)
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
            (i32.load offset=440
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
  (func $uphill_chemical (;27;) (type 17) (param i32 i32 i32)
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
            (i32.load offset=440
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
            (i32.load offset=440
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
            (i32.load offset=440
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
  (func $nest_scent_at_angle (;28;) (type 16) (param i32 i32 f64 f64) (result f64)
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
            (i32.load offset=360
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
  (func $uphill_nest_scent (;29;) (type 17) (param i32 i32 i32)
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
            (i32.load offset=360
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
            (i32.load offset=360
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
            (i32.load offset=360
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
  (func $setup (;30;) (type 6) (param i32)
    (local i32 i32 i64 i32 i32 i32 i32 i32 i32 i32 f64 f64)
    (global.set $__stack_pointer
      (local.tee 1
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 256))))
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
          (i32.const 240))
        (i32.const 8))
      (i64.const 0))
    (i64.store
      (i32.add
        (i32.add
          (local.get 1)
          (i32.const 136))
        (i32.const 8))
      (i64.const 0))
    (i64.store offset=240
      (local.get 1)
      (i64.const 0))
    (i64.store offset=136
      (local.get 1)
      (i64.const 0))
    (call $oxitortoise_next_turtle_from_iter
      (i32.add
        (local.get 1)
        (i32.const 232))
      (local.tee 4
        (call $oxitortoise_create_turtles
          (local.get 0)
          (local.get 3)
          (i64.const 125)
          (i32.add
            (local.get 1)
            (i32.const 136)))))
    (i64.store offset=216
      (local.get 1)
      (local.tee 3
        (i64.load offset=232
          (local.get 1))))
    (local.set 5
      (i32.add
        (local.get 0)
        (i32.const 8)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i64.eqz
          (local.get 3)))
      (loop ;; label = @2
        (i64.store offset=40
          (local.tee 6
            (i32.add
              (i32.load offset=16
                (local.get 2))
              (i32.mul
                (i32.load offset=216
                  (local.get 1))
                (i32.const 112))))
          (i64.const 4624633867356078080))
        (i64.store offset=80
          (local.get 6)
          (i64.const 4611686018427387904))
        (i64.store offset=128
          (local.get 1)
          (i64.load offset=216
            (local.get 1)))
        (call $oxitortoise_update_turtle
          (local.get 5)
          (local.get 2)
          (i32.add
            (local.get 1)
            (i32.const 128))
          (i32.const 514))
        (call $oxitortoise_next_turtle_from_iter
          (i32.add
            (local.get 1)
            (i32.const 232))
          (local.get 4))
        (i64.store offset=216
          (local.get 1)
          (local.tee 3
            (i64.load offset=232
              (local.get 1))))
        (br_if 0 (;@2;)
          (i64.ne
            (local.get 3)
            (i64.const 0)))))
    (call $oxitortoise_drop_turtle_iter
      (local.get 4))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eq
          (local.tee 6
            (call $oxitortoise_next_patch_from_iter
              (local.tee 7
                (call $oxitortoise_make_all_patches_iter
                  (local.get 0)))))
          (i32.const -1)))
      (local.set 6
        (local.get 6))
      (loop ;; label = @2
        (i64.store
          (local.tee 4
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 216))
              (i32.const 8)))
          (i64.load
            (local.tee 10
              (i32.add
                (local.tee 6
                  (i32.add
                    (i32.load offset=360
                      (local.get 2))
                    (local.tee 9
                      (i32.mul
                        (local.tee 8
                          (local.get 6))
                        (i32.const 80)))))
                (i32.const 16)))))
        (local.set 3
          (i64.load offset=8
            (local.get 6)))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 200))
            (i32.const 8))
          (i64.const 0))
        (i64.store offset=216
          (local.get 1)
          (local.get 3))
        (i64.store offset=200
          (local.get 1)
          (i64.const 0))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 112))
            (i32.const 8))
          (i64.load
            (local.get 10)))
        (local.set 3
          (i64.load offset=8
            (local.get 6)))
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
          (local.get 6)
          (f64.sub
            (f64.const 0x1.9p+7 (;=200;))
            (local.tee 11
              (call $oxitortoise_distance_euclidean_no_wrap
                (i32.add
                  (local.get 1)
                  (i32.const 112))
                (i32.add
                  (local.get 1)
                  (i32.const 96))))))
        (i32.store8 offset=56
          (local.get 6)
          (f64.lt
            (local.get 11)
            (f64.const 0x1.4p+2 (;=5;))))
        (local.set 12
          (f64.load offset=584
            (local.get 2)))
        (local.set 11
          (f64.load offset=560
            (local.get 2)))
        (i64.store
          (local.tee 10
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 184))
              (i32.const 8)))
          (i64.const 0))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 80))
            (i32.const 8))
          (i64.load
            (local.get 4)))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 64))
            (i32.const 8))
          (i64.load
            (local.get 10)))
        (f64.store offset=184
          (local.get 1)
          (f64.mul
            (local.get 11)
            (f64.const 0x1.3333333333333p-1 (;=0.6;))))
        (i64.store offset=80
          (local.get 1)
          (i64.load offset=216
            (local.get 1)))
        (i64.store offset=64
          (local.get 1)
          (i64.load offset=184
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
            (local.get 6)
            (i64.const 4607182418800017408)))
        (f64.store
          (local.tee 10
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 168))
              (i32.const 8)))
          (f64.mul
            (local.get 12)
            (f64.const -0x1.3333333333333p-1 (;=-0.6;))))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 48))
            (i32.const 8))
          (i64.load
            (local.get 4)))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 32))
            (i32.const 8))
          (i64.load
            (local.get 10)))
        (f64.store offset=168
          (local.get 1)
          (f64.mul
            (local.get 11)
            (f64.const -0x1.3333333333333p-1 (;=-0.6;))))
        (i64.store offset=48
          (local.get 1)
          (i64.load offset=216
            (local.get 1)))
        (i64.store offset=32
          (local.get 1)
          (i64.load offset=168
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
            (local.get 6)
            (i64.const 4611686018427387904)))
        (f64.store
          (local.tee 10
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 152))
              (i32.const 8)))
          (f64.mul
            (local.get 12)
            (f64.const 0x1.999999999999ap-1 (;=0.8;))))
        (i64.store
          (i32.add
            (i32.add
              (local.get 1)
              (i32.const 16))
            (i32.const 8))
          (i64.load
            (local.get 4)))
        (i64.store
          (i32.add
            (local.get 1)
            (i32.const 8))
          (i64.load
            (local.get 10)))
        (f64.store offset=152
          (local.get 1)
          (f64.mul
            (local.get 11)
            (f64.const -0x1.999999999999ap-1 (;=-0.8;))))
        (i64.store offset=16
          (local.get 1)
          (i64.load offset=216
            (local.get 1)))
        (i64.store
          (local.get 1)
          (i64.load offset=152
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
            (local.get 6)
            (i64.const 4613937818241073152)))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (i32.eqz
              (f64.gt
                (f64.load offset=72
                  (local.get 6))
                (f64.const 0x0p+0 (;=0;)))))
          (f64.store offset=48
            (local.get 6)
            (select
              (f64.const 0x1p+1 (;=2;))
              (f64.const 0x1p+0 (;=1;))
              (call $oxitortoise_next_int
                (local.get 0)
                (i32.const 2)))))
        (local.set 4
          (i32.shl
            (local.get 8)
            (i32.const 3)))
        (local.set 10
          (i32.load offset=400
            (local.tee 6
              (i32.load
                (local.get 0)))))
        (block ;; label = @3
          (block ;; label = @4
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (i32.load8_u offset=56
                    (local.tee 9
                      (i32.add
                        (i32.load offset=360
                          (local.get 6))
                        (local.get 9))))))
              (local.set 11
                (f64.const 0x1.ccp+6 (;=115;)))
              (br 1 (;@4;)))
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (f64.gt
                    (f64.load offset=48
                      (local.get 9))
                    (f64.const 0x0p+0 (;=0;)))))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (f64.ne
                    (local.tee 11
                      (f64.load offset=72
                        (local.get 9)))
                    (f64.const 0x1p+0 (;=1;))))
                (local.set 11
                  (f64.const 0x1.54p+6 (;=85;)))
                (br 2 (;@4;)))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (f64.ne
                    (local.get 11)
                    (f64.const 0x1p+1 (;=2;))))
                (local.set 11
                  (f64.const 0x1.7cp+6 (;=95;)))
                (br 2 (;@4;)))
              (br_if 2 (;@3;)
                (f64.ne
                  (local.get 11)
                  (f64.const 0x1.8p+1 (;=3;))))
              (local.set 11
                (f64.const 0x1.a4p+6 (;=105;)))
              (br 1 (;@4;)))
            (local.set 11
              (call $oxitortoise_scale_color
                (f64.const 0x1.04p+6 (;=65;))
                (f64.load
                  (i32.add
                    (i32.load offset=440
                      (local.get 6))
                    (local.get 4)))
                (f64.const 0x1.999999999999ap-4 (;=0.1;))
                (f64.const 0x1.4p+2 (;=5;)))))
          (f64.store
            (i32.add
              (local.get 10)
              (local.get 4))
            (local.get 11)))
        (call $oxitortoise_update_patch
          (local.get 5)
          (local.get 6)
          (local.get 8)
          (i32.const 1))
        (local.set 6
          (local.tee 4
            (call $oxitortoise_next_patch_from_iter
              (local.get 7))))
        (br_if 0 (;@2;)
          (i32.ne
            (local.get 4)
            (i32.const -1)))))
    (call $oxitortoise_drop_patch_iter
      (local.get 7))
    (call $oxitortoise_reset_ticks
      (local.get 2))
    (call $oxitortoise_update_tick
      (local.get 5)
      (call $oxitortoise_get_ticks
        (local.get 2)))
    (global.set $__stack_pointer
      (i32.add
        (local.get 1)
        (i32.const 256)))
  )
  (func $shim_setup (;31;) (type 9) (param i32 i32)
    (call $setup
      (local.get 0))
  )
  (func $go (;32;) (type 6) (param i32)
    (local i32 i32 i32 i64 i32 i32 i32 i32 f64 i32 i32 i32)
    (global.set $__stack_pointer
      (local.tee 1
        (i32.sub
          (global.get $__stack_pointer)
          (i32.const 160))))
    (local.set 2
      (i32.load
        (local.get 0)))
    (call $oxitortoise_next_turtle_from_iter
      (i32.add
        (local.get 1)
        (i32.const 120))
      (local.tee 3
        (call $oxitortoise_make_all_turtles_iter
          (local.get 0))))
    (i64.store offset=152
      (local.get 1)
      (local.tee 4
        (i64.load offset=120
          (local.get 1))))
    (local.set 5
      (i32.add
        (local.get 0)
        (i32.const 8)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i64.eqz
          (local.get 4)))
      (loop ;; label = @2
        (local.set 4
          (i64.load offset=8
            (local.tee 6
              (i32.add
                (i32.load offset=16
                  (local.get 2))
                (i32.mul
                  (i32.load offset=152
                    (local.get 1))
                  (i32.const 112))))))
        (block ;; label = @3
          (br_if 0 (;@3;)
            (f64.le
              (call $oxitortoise_get_ticks
                (local.get 2))
              (f64.convert_i64_u
                (local.get 4))))
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
          (block ;; label = @4
            (block ;; label = @5
              (br_if 0 (;@5;)
                (f64.ne
                  (local.get 9)
                  (f64.const 0x1.ep+3 (;=15;))))
              (i32.store offset=144
                (local.get 1)
                (local.get 10))
              (i32.store offset=148
                (local.get 1)
                (i32.trunc_sat_f64_u
                  (call $oxitortoise_round
                    (f64.load offset=104
                      (local.get 6)))))
              (i64.store offset=80
                (local.get 1)
                (i64.load offset=144 align=4
                  (local.get 1)))
              (local.set 10
                (call $oxitortoise_patch_at
                  (local.get 2)
                  (i32.add
                    (local.get 1)
                    (i32.const 80))))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (i32.eqz
                    (local.tee 12
                      (f64.gt
                        (local.tee 9
                          (f64.load offset=48
                            (local.tee 11
                              (i32.add
                                (i32.load offset=360
                                  (local.get 2))
                                (i32.mul
                                  (local.get 10)
                                  (i32.const 80))))))
                        (f64.const 0x0p+0 (;=0;))))))
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
                (i64.store offset=72
                  (local.get 1)
                  (i64.load offset=152
                    (local.get 1)))
                (call $oxitortoise_update_turtle
                  (local.get 5)
                  (local.get 2)
                  (i32.add
                    (local.get 1)
                    (i32.const 72))
                  (i32.const 1030))
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
                            (i32.load offset=440
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
                      (i32.const 56))
                    (i32.const 8))
                  (i64.load
                    (i32.add
                      (local.get 8)
                      (i32.const 8))))
                (i64.store offset=56
                  (local.get 1)
                  (i64.load
                    (local.get 8)))
                (call $uphill_chemical
                  (local.get 2)
                  (i32.add
                    (local.get 1)
                    (i32.const 56))
                  (local.get 7)))
              (br_if 1 (;@4;)
                (i32.eqz
                  (local.get 12)))
              (br 2 (;@3;)))
            (i32.store offset=136
              (local.get 1)
              (local.get 10))
            (i32.store offset=140
              (local.get 1)
              (i32.trunc_sat_f64_u
                (call $oxitortoise_round
                  (f64.load offset=104
                    (local.get 6)))))
            (i64.store offset=104
              (local.get 1)
              (i64.load offset=136 align=4
                (local.get 1)))
            (local.set 10
              (call $oxitortoise_patch_at
                (local.get 2)
                (i32.add
                  (local.get 1)
                  (i32.const 104))))
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.ne
                  (i32.load8_u offset=56
                    (i32.add
                      (i32.load offset=360
                        (local.get 2))
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
              (br 1 (;@4;)))
            (f64.store
              (local.tee 6
                (i32.add
                  (i32.load offset=440
                    (local.get 2))
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
                  (local.get 1)
                  (i32.const 88))
                (i32.const 8))
              (i64.load
                (i32.add
                  (local.get 8)
                  (i32.const 8))))
            (i64.store offset=88
              (local.get 1)
              (i64.load
                (local.get 8)))
            (call $uphill_nest_scent
              (local.get 2)
              (i32.add
                (local.get 1)
                (i32.const 88))
              (local.get 7)))
          (local.set 6
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
                  (local.get 6)))))
          (local.set 6
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
                  (local.get 6)))))
          (local.set 9
            (f64.load
              (local.get 7)))
          (i64.store
            (i32.add
              (i32.add
                (local.get 1)
                (i32.const 40))
              (i32.const 8))
            (i64.load
              (local.tee 6
                (i32.add
                  (local.get 8)
                  (i32.const 8)))))
          (i64.store offset=40
            (local.get 1)
            (i64.load
              (local.get 8)))
          (call $oxitortoise_offset_distance_by_heading
            (i32.add
              (local.get 1)
              (i32.const 120))
            (local.get 2)
            (i32.add
              (local.get 1)
              (i32.const 40))
            (local.get 9)
            (f64.const 0x1p+0 (;=1;)))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (i32.eqz
                (call $oxitortoise_is_nan
                  (f64.load offset=120
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
                (i32.const 24))
              (i32.const 8))
            (i64.load
              (local.get 6)))
          (i64.store offset=24
            (local.get 1)
            (i64.load
              (local.get 8)))
          (call $oxitortoise_offset_distance_by_heading
            (i32.add
              (local.get 1)
              (i32.const 120))
            (local.get 2)
            (i32.add
              (local.get 1)
              (i32.const 24))
            (local.get 9)
            (f64.const 0x1p+0 (;=1;)))
          (block ;; label = @4
            (br_if 0 (;@4;)
              (call $oxitortoise_is_nan
                (f64.load offset=120
                  (local.get 1))))
            (i64.store
              (local.get 8)
              (i64.load offset=120
                (local.get 1)))
            (i64.store
              (local.get 6)
              (i64.load
                (i32.add
                  (i32.add
                    (local.get 1)
                    (i32.const 120))
                  (i32.const 8)))))
          (i64.store offset=16
            (local.get 1)
            (i64.load offset=152
              (local.get 1)))
          (call $oxitortoise_update_turtle
            (local.get 5)
            (local.get 2)
            (i32.add
              (local.get 1)
              (i32.const 16))
            (i32.const 1030)))
        (call $oxitortoise_next_turtle_from_iter
          (i32.add
            (local.get 1)
            (i32.const 120))
          (local.get 3))
        (i64.store offset=152
          (local.get 1)
          (local.tee 4
            (i64.load offset=120
              (local.get 1))))
        (br_if 0 (;@2;)
          (i64.ne
            (local.get 4)
            (i64.const 0)))))
    (call $oxitortoise_drop_turtle_iter
      (local.get 3))
    (i32.store16 offset=14
      (local.get 1)
      (i32.const 2))
    (i32.store16 offset=118 align=1
      (local.get 1)
      (i32.const 2))
    (call $oxitortoise_diffuse_8
      (local.get 2)
      (i32.add
        (local.get 1)
        (i32.const 14))
      (f64.const 0x1p-1 (;=0.5;)))
    (block ;; label = @1
      (br_if 0 (;@1;)
        (i32.eq
          (local.tee 6
            (call $oxitortoise_next_patch_from_iter
              (local.tee 12
                (call $oxitortoise_make_all_patches_iter
                  (local.get 0)))))
          (i32.const -1)))
      (local.set 6
        (local.get 6))
      (loop ;; label = @2
        (f64.store
          (local.tee 6
            (i32.add
              (i32.load offset=440
                (local.get 2))
              (local.tee 8
                (i32.shl
                  (local.tee 7
                    (local.get 6))
                  (i32.const 3)))))
          (f64.mul
            (f64.load
              (local.get 6))
            (f64.const 0x1.ccccccccccccdp-1 (;=0.9;))))
        (local.set 3
          (i32.load offset=400
            (local.tee 6
              (i32.load
                (local.get 0)))))
        (block ;; label = @3
          (block ;; label = @4
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (i32.load8_u offset=56
                    (local.tee 10
                      (i32.add
                        (i32.load offset=360
                          (local.get 6))
                        (i32.mul
                          (local.get 7)
                          (i32.const 80)))))))
              (local.set 9
                (f64.const 0x1.ccp+6 (;=115;)))
              (br 1 (;@4;)))
            (block ;; label = @5
              (br_if 0 (;@5;)
                (i32.eqz
                  (f64.gt
                    (f64.load offset=48
                      (local.get 10))
                    (f64.const 0x0p+0 (;=0;)))))
              (block ;; label = @6
                (br_if 0 (;@6;)
                  (f64.ne
                    (local.tee 9
                      (f64.load offset=72
                        (local.get 10)))
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
                    (i32.load offset=440
                      (local.get 6))
                    (local.get 8)))
                (f64.const 0x1.999999999999ap-4 (;=0.1;))
                (f64.const 0x1.4p+2 (;=5;)))))
          (f64.store
            (i32.add
              (local.get 3)
              (local.get 8))
            (local.get 9)))
        (call $oxitortoise_update_patch
          (local.get 5)
          (local.get 6)
          (local.get 7)
          (i32.const 1))
        (local.set 6
          (local.tee 7
            (call $oxitortoise_next_patch_from_iter
              (local.get 12))))
        (br_if 0 (;@2;)
          (i32.ne
            (local.get 7)
            (i32.const -1)))))
    (call $oxitortoise_drop_patch_iter
      (local.get 12))
    (call $oxitortoise_advance_tick
      (local.get 2))
    (call $oxitortoise_update_tick
      (local.get 5)
      (call $oxitortoise_get_ticks
        (local.get 2)))
    (global.set $__stack_pointer
      (i32.add
        (local.get 1)
        (i32.const 160)))
  )
  (func $shim_go (;33;) (type 9) (param i32 i32)
    (call $go
      (local.get 0))
  )
  (@custom ".debug_loc" (after code) "\ff\ff\ff\ff\06\00\00\00\00\00\00\00\00\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\0c\00\00\00\f3\00\00\00\06\00\ed\00\00#\08\9f\f3\00\00\00\ff\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\11\00\00\00\13\00\00\00\04\00\ed\02\00\9f\13\00\00\004\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\11\00\00\00\13\00\00\00\04\00\ed\02\00\9f\13\00\00\00\00\01\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\c5\00\00\00\c8\00\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\11\00\00\00\13\00\00\00\04\00\ed\02\00\9f\13\00\00\004\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00\e8\00\00\00\ed\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\06\00\00\00,\00\00\00.\00\00\00\04\00\ed\02\00\9f.\00\00\00\00\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\08\01\00\00\00\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\08\01\00\00!\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\08\01\00\00\00\00\00\00\cf\00\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\08\01\00\00\af\00\00\00\c0\00\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\08\01\00\00\bb\00\00\00\be\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00 \00\00\00*\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\002\00\00\00\89\00\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\00\00\00\00\9f\02\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00Q\00\00\00E\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\c6\00\00\00\d7\00\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\d2\00\00\00\d5\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\dc\00\00\00\9f\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\e1\00\00\00\eb\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\f3\00\00\00E\01\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\82\01\00\00\93\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\8e\01\00\00\91\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\98\01\00\00\9f\02\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\9d\01\00\00\a7\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00\af\01\00\00\04\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00>\02\00\00O\02\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00J\02\00\00M\02\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\d9\01\00\00T\02\00\00\9f\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffz\04\00\00\00\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffz\04\00\00!\00\00\00u\00\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffz\04\00\00\00\00\00\00\d0\00\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffz\04\00\00\af\00\00\00\c1\00\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00 \00\00\00*\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\002\00\00\00\89\00\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\00\00\00\00\a2\02\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00Q\00\00\00F\01\00\00\04\00\ed\00\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\c6\00\00\00\d8\00\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\dd\00\00\00\a2\02\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\e2\00\00\00\ec\00\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\f4\00\00\00F\01\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\83\01\00\00\95\01\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\9a\01\00\00\a2\02\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\9f\01\00\00\a9\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00\b1\01\00\00\06\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00@\02\00\00R\02\00\00\04\00\ed\00\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ffL\05\00\00W\02\00\00\a2\02\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\15\00\00\00\8f\00\00\00\06\00\ed\00\01#\d8\01\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\22\00\00\00\15\05\00\00\04\00\ed\00\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00y\00\00\00{\00\00\00\04\00\ed\02\01\9f{\00\00\004\01\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\96\00\00\00\15\05\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\9f\00\00\00\11\01\00\00\03\00\11\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00%\01\00\00'\01\00\00\04\00\ed\02\00\9f'\01\00\00\15\05\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00V\01\00\00X\01\00\00\04\00\ed\02\01\9fX\01\00\00\0b\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\d9\01\00\00\db\01\00\00\04\00\ed\02\02\9f\db\01\00\00{\02\00\00\04\00\ed\00\0b\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\f8\01\00\00\e6\04\00\00\04\00\ed\00\0c\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\00\02\00\00B\04\00\00\04\00\ed\00\0b\9f\89\04\00\00\b9\04\00\00\04\00\ed\00\0b\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00n\02\00\00x\02\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\ff\02\00\00\09\03\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\8b\03\00\00\95\03\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\db\03\00\00\dc\03\00\00\04\00\ed\02\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\ec\03\00\00\ee\03\00\00\04\00\ed\02\00\9f\ee\03\00\00\0b\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\ec\03\00\00\ee\03\00\00\04\00\ed\02\00\9f\ee\03\00\00\e6\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\9c\04\00\00\9f\04\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\ec\03\00\00\ee\03\00\00\04\00\ed\02\00\9f\ee\03\00\00\0b\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\bf\04\00\00\c4\04\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\03\04\00\00\05\04\00\00\04\00\ed\02\00\9f\05\04\00\00\e6\04\00\00\04\00\ed\00\09\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\f0\07\00\00\db\04\00\00\dd\04\00\00\04\00\ed\02\00\9f\dd\04\00\00\e6\04\00\00\04\00\ed\00\04\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\17\00\00\00G\00\00\00\06\00\ed\00\01#\98\01\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00$\00\00\00\9e\05\00\00\04\00\ed\00\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\002\00\00\004\00\00\00\04\00\ed\02\01\9f4\00\00\00F\04\00\00\04\00\ed\00\03\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00N\00\00\00\9e\05\00\00\04\00\ed\00\05\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00W\00\00\00~\00\00\00\03\00\11\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00g\00\00\00i\00\00\00\04\00\ed\02\00\9fi\00\00\00\a1\02\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\86\00\00\00\d8\03\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\8e\00\00\00\d8\03\00\00\04\00\ed\00\08\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\ec\00\00\00\d0\01\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\83\01\00\00\86\01\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\86\01\00\00\88\01\00\00\04\00\ed\02\00\9f\88\01\00\00\d0\01\00\00\04\00\ed\00\09\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\0f\02\00\00\a1\02\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00_\02\00\00a\02\00\00\04\00\ed\02\00\9fa\02\00\00\a1\02\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\b8\02\00\00\b9\02\00\00\04\00\ed\02\02\9f\d8\02\00\00\d9\02\00\00\04\00\ed\02\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\003\04\00\005\04\00\00\04\00\ed\02\00\9f5\04\00\00\9e\05\00\00\04\00\ed\00\0c\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00H\04\00\00\94\04\00\00\03\00\11\02\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00X\04\00\00Z\04\00\00\04\00\ed\02\00\9fZ\04\00\00\94\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00q\04\00\00s\04\00\00\04\00\ed\02\00\9fs\04\00\00\94\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00q\04\00\00s\04\00\00\04\00\ed\02\00\9fs\04\00\00o\05\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00%\05\00\00(\05\00\00\04\00\ed\02\01\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00q\04\00\00s\04\00\00\04\00\ed\02\00\9fs\04\00\00\94\04\00\00\04\00\ed\00\06\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00H\05\00\00M\05\00\00\04\00\ed\02\00\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00\8c\04\00\00\8e\04\00\00\04\00\ed\02\00\9f\8e\04\00\00o\05\00\00\04\00\ed\00\0a\9f\00\00\00\00\00\00\00\00\ff\ff\ff\ff\12\0d\00\00d\05\00\00f\05\00\00\04\00\ed\02\00\9ff\05\00\00o\05\00\00\04\00\ed\00\07\9f\00\00\00\00\00\00\00\00")
  (@custom ".debug_abbrev" (after code) "\01\11\01%\0e\13\05\03\0e\10\17\1b\0e\11\01U\17\00\00\02\0f\00I\13\00\00\03\16\00I\13\03\0e:\0b;\0b\00\00\04\13\01\0b\0b:\0b;\0b\00\00\05\0d\00\03\0eI\13:\0b;\0b8\0b\00\00\06\01\01I\13\00\00\07!\00I\137\0b\00\00\08$\00\03\0e>\0b\0b\0b\00\00\09$\00\03\0e\0b\0b>\0b\00\00\0a\13\00\0b\0b:\0b;\0b\00\00\0b\0f\00\00\00\0c.\01\03\0e:\0b;\0b'\19I\13 \0b\00\00\0d\05\00\03\0e:\0b;\0bI\13\00\00\0e4\00\03\0e:\0b;\0bI\13\00\00\0f.\01\11\01\12\06@\18\97B\191\13\00\00\10\05\00\02\171\13\00\00\11\05\00\02\181\13\00\00\124\00\02\171\13\00\00\13\1d\011\13\11\01\12\06X\0bY\0bW\0b\00\00\14\05\00\1c\0d1\13\00\00\15\1d\001\13\11\01\12\06X\0bY\0bW\0b\00\00\16\89\82\01\001\13\11\01\00\00\17.\01\03\0e:\0b;\0b'\19I\13<\19?\19\00\00\18\05\00I\13\00\00\19.\01\03\0e:\0b;\0b'\19<\19?\19\00\00\1a\05\001\13\00\00\1b4\00\02\181\13\00\00\1c.\01\03\0e:\0b;\0b'\19I\13?\19 \0b\00\00\1d.\01\11\01\12\06@\18\97B\19\03\0e:\0b;\0b'\19?\19\00\00\1e\05\00\02\17\03\0e:\0b;\0bI\13\00\00\1f\05\00\02\18\03\0e:\0b;\0bI\13\00\00 4\00\02\17\03\0e:\0b;\0bI\13\00\00!\05\00\1c\0f1\13\00\00\224\001\13\00\00#\1d\011\13\11\01\12\06X\0bY\05W\0b\00\00$.\01\03\0e:\0b;\05'\19I\13?\19 \0b\00\00%\05\00\03\0e:\0b;\05I\13\00\00&4\00\03\0e:\0b;\05I\13\00\00'.\01\11\01\12\06@\18\97B\19\03\0e:\0b;\05'\19?\19\00\00(\05\00\02\17\03\0e:\0b;\05I\13\00\00)\05\00\02\18\03\0e:\0b;\05I\13\00\00*4\00\02\17\03\0e:\0b;\05I\13\00\00+.\01\03\0e:\0b;\0b'\19?\19 \0b\00\00,\0b\01U\17\00\00-\0b\01\11\01\12\06\00\00.4\00\02\18\03\0e:\0b;\05I\13\00\00/\1d\001\13\11\01\12\06X\0bY\05W\0b\00\000\17\01\0b\0b:\0b;\0b\00\00\00")
  (@custom ".debug_info" (after code) "M\16\00\00\04\00\00\00\00\00\04\01\db\06\00\00\1d\00\06\06\00\00\00\00\00\00%\01\00\00\00\00\00\00\18\00\00\00\02+\00\00\00\036\00\00\00\b3\06\00\00\02\a1\04P\02\9a\05w\05\00\00\83\00\00\00\02\9b\00\05'\06\00\00\a8\00\00\00\02\9c\08\05I\05\00\00\04\01\00\00\02\9d0\05@\00\00\004\01\00\00\02\9e8\05\93\00\00\00\04\01\00\00\02\9f@\05\18\03\00\00\04\01\00\00\02\a0H\00\06\8f\00\00\00\07\a1\00\00\00\01\00\03\9a\00\00\00\02\01\00\00\01\ca\08+\03\00\00\08\01\09c\06\00\00\08\07\03\b3\00\00\00F\06\00\00\027\04(\023\05\85\03\00\00\dc\00\00\00\024\00\05\c8\03\00\00\16\01\00\00\025\10\05\f2\01\00\00\04\01\00\00\026 \00\03\e7\00\00\00T\00\00\00\02\12\04\10\02\0f\05*\00\00\00\04\01\00\00\02\10\00\05\00\00\00\00\04\01\00\00\02\11\08\00\03\0f\01\00\00\e7\00\00\00\02\0c\08%\05\00\00\04\08\03!\01\00\00U\04\00\00\02\0d\06-\01\00\00\07\a1\00\00\00\0c\00\084\03\00\00\06\01\08\ac\03\00\00\02\01\02@\01\00\00\03K\01\00\00\a0\06\00\00\02\a4\04\08\02\a2\05\eb\01\00\00\04\01\00\00\02\a3\00\00\02a\01\00\00\03l\01\00\00\8d\06\00\00\02\a7\04\08\02\a5\05\d6\03\00\00\04\01\00\00\02\a6\00\00\03\88\01\00\00\1c\01\00\00\01\d4\08\7f\00\00\00\07\04\02\94\01\00\00\03\9f\01\00\00\bf\06\00\00\02\99\04p\02\94\05w\05\00\00\83\00\00\00\02\95\00\05'\06\00\00\d4\01\00\00\02\96\08\05\aa\04\00\00\04\01\00\00\02\97X\05\85\03\00\00\dc\00\00\00\02\98`\00\03\df\01\00\00T\06\00\00\021\04P\02(\05k\03\00\00D\02\00\00\02)\00\05\c0\05\00\00a\02\00\00\02*\08\05\ce\04\00\00\16\01\00\00\02+\10\05\11\02\00\00\04\01\00\00\02, \05\c9\03\00\00\16\01\00\00\02-(\05\f3\01\00\00\04\01\00\00\02.8\05\92\03\00\004\01\00\00\02/@\05\b2\04\00\00\04\01\00\00\020H\00\03O\02\00\00o\03\00\00\02\0a\03Z\02\00\00\13\01\00\00\01\d9\08B\04\00\00\07\08\03O\02\00\00\fe\05\00\00\02\0b\03w\02\00\00\ed\05\00\00\02&\03\82\02\00\00\1d\01\00\00\01\bb\08\88\00\00\00\05\04\02\8e\02\00\00\02\93\02\00\00\03\9e\02\00\00?\05\00\00\02E\0a\00\02E\02-\01\00\00\02\ac\02\00\00\03\b7\02\00\00q\05\00\00\02G\0a\00\02G\02\c0\02\00\00\03\cb\02\00\00\10\03\00\00\02C\0a\00\02C\02\d4\02\00\00\0b\02\04\01\00\00\0c`\05\00\00\02Z\a7\02\00\00\01\0d0\00\00\00\02Z\fd\02\00\00\0e5\05\00\00\02[\8e\02\00\00\00\02\02\03\00\00\03\0d\03\00\008\00\00\00\02A\0a\00\02A\0c\13\06\00\00\02c\d4\02\00\00\01\0dk\05\00\00\02c\a7\02\00\00\0d\17\00\00\00\02c\82\02\00\00\00\0c\fd\02\00\00\02V\bb\02\00\00\01\0d0\00\00\00\02V\fd\02\00\00\00\0f\06\00\00\00\00\01\00\00\07\ed\03\00\00\00\00\9f\fe\0b\00\00\10\00\00\00\00\06\0c\00\00\11\04\ed\00\01\9f\11\0c\00\00\12\1e\00\00\00H\0c\00\00\12x\00\00\00\1c\0c\00\00\12\a4\00\00\00'\0c\00\00\12\ee\00\00\002\0c\00\00\12\0c\01\00\00=\0c\00\00\13\da\02\00\00\12\00\00\00\05\00\00\00\02\d0\11\11\04\ed\00\00\9f\e6\02\00\00\12L\00\00\00\f1\02\00\00\00\13\11\03\00\00\17\00\00\00\08\00\00\00\02\d4'\10\c2\00\00\00\1d\03\00\00\14\01(\03\00\00\00\13\11\03\00\00\1f\00\00\00\11\00\00\00\02\d3'\11\04\ed\00\03\9f\1d\03\00\00\14\00(\03\00\00\00\154\03\00\00\f8\00\00\00\07\00\00\00\02\d1\15\16)\04\00\00\e6\00\00\00\16I\04\00\00\05\01\00\00\00\17\ff\01\00\00\02\8e\04\01\00\00\18\04\01\00\00\18\04\01\00\00\18\04\01\00\00\18\04\01\00\00\00\19)\04\00\00\02p\18\bb\02\00\00\18\a7\02\00\00\18l\02\00\00\18\8f\00\00\00\00\0f\08\01\00\00\cf\00\00\00\04\ed\00\04\9f\a2\05\00\00\10t\01\00\00\ae\05\00\00\1a\b9\05\00\00\11\04\ed\00\02\9f\c4\05\00\00\108\01\00\00\cf\05\00\00\1b\02\91 \da\05\00\00\1b\02\91\18\06\06\00\00\12V\01\00\00\e5\05\00\00\12\92\01\00\00\f0\05\00\00\12\b0\01\00\00\fb\05\00\00\13\11\03\00\00\b7\01\00\00\0a\00\00\00\02\f2'\14\02(\03\00\00\00\16\11\05\00\00'\01\00\00\16\22\05\00\00c\01\00\00\16B\05\00\00{\01\00\00\16S\05\00\00\8a\01\00\00\16S\05\00\00\9c\01\00\00\16d\05\00\00\b5\01\00\00\00\17\94\04\00\00\02\8a\04\01\00\00\18\04\01\00\00\00\17`\04\00\00\02\88\dc\00\00\00\18\a7\02\00\00\18\dc\00\00\00\18\04\01\00\00\18\04\01\00\00\00\17\99\03\00\00\02s4\01\00\00\18\0f\01\00\00\00\17N\05\00\00\02t\04\01\00\00\18\04\01\00\00\00\17\ed\00\00\00\02\89l\02\00\00\18\a7\02\00\00\18z\05\00\00\00\03\85\05\00\00\9e\00\00\00\02\17\04\08\02\14\05*\00\00\00}\01\00\00\02\15\00\05\00\00\00\00}\01\00\00\02\16\04\00\1c\13\05\00\00\02\e7\04\01\00\00\01\0dk\05\00\00\02\e7\a7\02\00\00\0d\85\03\00\00\02\e7\dc\00\00\00\0d\aa\04\00\00\02\e7\04\01\00\00\0d\1f\05\00\00\02\e7\04\01\00\00\0e\c6\05\00\00\02\e9\dc\00\00\00\0e\87\04\00\00\02\e8\04\01\00\00\0e\8a\05\00\00\02\f0l\02\00\00\0e\99\06\00\00\02\f2\5c\01\00\00\0eo\00\00\00\02\efz\05\00\00\00\1d\d9\01\00\00\9f\02\00\00\04\ed\00\03\9f\cf\03\00\00\02\f6\1e\0a\02\00\00k\05\00\00\02\f6\a7\02\00\00\0d\85\03\00\00\02\f6\dc\00\00\00\1f\04\ed\00\02\9f\aa\04\00\00\02\f6\d5\02\00\00 \82\02\00\00\de\05\00\00\02\f7\04\01\00\00 \18\03\00\00\bf\00\00\00\02\f8\04\01\00\00 \ae\03\00\00\d9\00\00\00\02\f9\04\01\00\00\13\a2\05\00\00\02\02\00\00\b3\00\00\00\02\f7\19\10(\02\00\00\ae\05\00\00\10\ce\01\00\00\c4\05\00\00!\00\cf\05\00\00\1b\03\91\d0\00\da\05\00\00\12\ec\01\00\00\e5\05\00\00\12F\02\00\00\f0\05\00\00\12d\02\00\00\fb\05\00\00\13\11\03\00\00\9f\02\00\00\0a\00\00\00\02\f2'\14\02(\03\00\00\00\00\13\a2\05\00\00\c3\02\00\00\ae\00\00\00\02\f8\19\10\a0\02\00\00\c4\05\00\00!\80\80\80\80\80\80\a0\a3@\cf\05\00\00\1b\03\91\d0\00\da\05\00\00\12\be\02\00\00\e5\05\00\00\12\dc\02\00\00\f0\05\00\00\12\fa\02\00\00\fb\05\00\00\13\11\03\00\00[\03\00\00\0a\00\00\00\02\f2'\14\02(\03\00\00\00\00\13\a2\05\00\00\7f\03\00\00\ae\00\00\00\02\f9\18\106\03\00\00\c4\05\00\00!\80\80\80\80\80\80\a0\a3\c0\01\cf\05\00\00\1b\03\91\d0\00\da\05\00\00\12T\03\00\00\e5\05\00\00\12r\03\00\00\f0\05\00\00\12\90\03\00\00\fb\05\00\00\13\11\03\00\00\17\04\00\00\0a\00\00\00\02\f2'\14\02(\03\00\00\00\00\16\11\05\00\00\09\02\00\00\16\22\05\00\00H\02\00\00\16B\05\00\00`\02\00\00\16S\05\00\00o\02\00\00\16S\05\00\00\81\02\00\00\16d\05\00\00\9d\02\00\00\16\11\05\00\00\ca\02\00\00\16\22\05\00\00\04\03\00\00\16B\05\00\00\1c\03\00\00\16S\05\00\00+\03\00\00\16S\05\00\00=\03\00\00\16d\05\00\00Y\03\00\00\16\11\05\00\00\86\03\00\00\16\22\05\00\00\c3\03\00\00\16B\05\00\00\db\03\00\00\16S\05\00\00\ea\03\00\00\16S\05\00\00\fc\03\00\00\16d\05\00\00\15\04\00\00\16\11\05\00\00g\04\00\00\00\0fz\04\00\00\d0\00\00\00\04\ed\00\04\9f\fd\08\00\00\10\08\04\00\00\0a\09\00\00\1a\16\09\00\00\11\04\ed\00\02\9f\22\09\00\00\10\cc\03\00\00.\09\00\00\1b\02\91 :\09\00\00\1b\02\91\18^\09\00\00\12\ea\03\00\00F\09\00\00\12&\04\00\00R\09\00\00\22j\09\00\00#\11\03\00\00)\05\00\00\0b\00\00\00\02\10\01'\14\00(\03\00\00\00\16\11\05\00\00\99\04\00\00\16\22\05\00\00\d5\04\00\00\16B\05\00\00\ed\04\00\00\16S\05\00\00\fc\04\00\00\16S\05\00\00\0e\05\00\00\16d\05\00\00'\05\00\00\00$\ff\04\00\00\02\05\01\04\01\00\00\01%k\05\00\00\02\05\01\a7\02\00\00%\85\03\00\00\02\05\01\dc\00\00\00%\aa\04\00\00\02\05\01\04\01\00\00%\1f\05\00\00\02\05\01\04\01\00\00&\c6\05\00\00\02\07\01\dc\00\00\00&\87\04\00\00\02\06\01\04\01\00\00&\8a\05\00\00\02\0e\01l\02\00\00&o\00\00\00\02\0d\01z\05\00\00&\cc\06\00\00\02\10\01&\00\00\00\00'L\05\00\00\a2\02\00\00\04\ed\00\03\9f\8c\00\00\00\02\14\01(\80\04\00\00k\05\00\00\02\14\01\a7\02\00\00%\85\03\00\00\02\14\01\dc\00\00\00)\04\ed\00\02\9f\aa\04\00\00\02\14\01\d5\02\00\00*\da\04\00\00\d2\05\00\00\02\15\01\04\01\00\00*R\05\00\00\b3\00\00\00\02\16\01\04\01\00\00*\ca\05\00\00\ce\00\00\00\02\17\01\04\01\00\00#\fd\08\00\00u\05\00\00\b4\00\00\00\02\15\01\16\10\9e\04\00\00\0a\09\00\00\10D\04\00\00\22\09\00\00!\00.\09\00\00\1b\03\91\d0\00:\09\00\00\12b\04\00\00F\09\00\00\12\bc\04\00\00R\09\00\00#\11\03\00\00\12\06\00\00\0b\00\00\00\02\10\01'\14\00(\03\00\00\00\00#\fd\08\00\007\06\00\00\af\00\00\00\02\16\01\16\10\f8\04\00\00\22\09\00\00!\80\80\80\80\80\80\a0\a3@.\09\00\00\1b\03\91\d0\00:\09\00\00\12\16\05\00\00F\09\00\00\124\05\00\00R\09\00\00#\11\03\00\00\cf\06\00\00\0b\00\00\00\02\10\01'\14\00(\03\00\00\00\00#\fd\08\00\00\f4\06\00\00\af\00\00\00\02\17\01\15\10p\05\00\00\22\09\00\00!\80\80\80\80\80\80\a0\a3\c0\01.\09\00\00\1b\03\91\d0\00:\09\00\00\12\8e\05\00\00F\09\00\00\12\ac\05\00\00R\09\00\00#\11\03\00\00\8c\07\00\00\0b\00\00\00\02\10\01'\14\00(\03\00\00\00\00\16\11\05\00\00|\05\00\00\16\22\05\00\00\bb\05\00\00\16B\05\00\00\d3\05\00\00\16S\05\00\00\e2\05\00\00\16S\05\00\00\f4\05\00\00\16d\05\00\00\10\06\00\00\16\11\05\00\00>\06\00\00\16\22\05\00\00x\06\00\00\16B\05\00\00\90\06\00\00\16S\05\00\00\9f\06\00\00\16S\05\00\00\b1\06\00\00\16d\05\00\00\cd\06\00\00\16\11\05\00\00\fb\06\00\00\16\22\05\00\008\07\00\00\16B\05\00\00P\07\00\00\16S\05\00\00_\07\00\00\16S\05\00\00q\07\00\00\16d\05\00\00\8a\07\00\00\16\11\05\00\00\dd\07\00\00\00\0c1\06\00\00\02_\d4\02\00\00\01\0dk\05\00\00\02_\a7\02\00\00\0d\17\00\00\00\02_\82\02\00\00\00\0c\17\02\00\00\02k\04\01\00\00\01\0dk\05\00\00\02k\a7\02\00\00\00\0c*\02\00\00\02g\04\01\00\00\01\0dk\05\00\00\02g\a7\02\00\00\00+\1b\04\00\00\02\cf\01\0d0\00\00\00\02\cf\fd\02\00\00\0d\8a\05\00\00\02\cfl\02\00\00\0ek\05\00\00\02\d0\a7\02\00\00\0e\99\06\00\00\02\d5\5c\01\00\00\0e\ac\06\00\00\02\d4;\01\00\00\0e\cc\06\00\00\02\d3&\00\00\00\0e\08\03\00\00\02\d1\bb\02\00\00\00'\f0\07\00\00\15\05\00\00\04\ed\00\01\9f>\03\00\00\02#\01)\04\ed\00\00\9f0\00\00\00\02#\01\fd\02\00\00*\08\06\00\00k\05\00\00\02%\01\a7\02\00\00*R\06\00\00\08\03\00\00\02$\01\bb\02\00\00#\da\02\00\00\0b\08\00\00\07\00\00\00\02%\01\11\11\04\ed\00\00\9f\e6\02\00\00\00,\00\00\00\00*\e8\05\00\00\d9\04\00\00\022\01\17\10\00\00*&\06\00\00\f8\02\00\00\02,\01\f2\0f\00\00-\8d\08\00\00Q\00\00\00&:\06\00\00\026\01\8f\01\00\00&'\06\00\00\027\01F\16\00\00#\ab\0b\00\00\8d\08\00\00\07\00\00\00\026\010\10p\06\00\00\c2\0b\00\00\00\00\00#4\03\00\00\83\08\00\00\03\00\00\00\02$\01\15\11\04\ed\00\00\9f@\03\00\00\00-\0b\09\00\00\d5\03\00\00*\8d\06\00\00\f8\02\00\00\02D\01\a8\10\00\00*\bf\08\00\00\10\04\00\00\02E\01l\02\00\00-(\09\00\00\9b\03\00\00.\03\91\d8\01\85\03\00\00\02I\01\dc\00\00\00*\b9\06\00\00<\04\00\00\02H\01&\00\00\00*\e5\06\00\00,\05\00\00\02J\01\04\01\00\00#\11\03\00\003\09\00\00\0f\00\00\00\02H\01(\14\00(\03\00\00\00-\e0\09\00\00\e3\02\00\00*\11\07\00\00 \02\00\00\02U\01\04\01\00\00*/\07\00\003\02\00\00\02T\01\04\01\00\00/\ce\0b\00\00\e0\09\00\00\08\00\00\00\02U\01\17/\e6\0b\00\00\e8\09\00\00\08\00\00\00\02T\01\17-\f0\09\00\00\8a\00\00\00*[\07\00\00,\05\00\00\02Y\01\04\01\00\00\00-{\0a\00\00\91\00\00\00*y\07\00\00,\05\00\00\02a\01\04\01\00\00\00-\0d\0b\00\00\8b\00\00\00*\97\07\00\00,\05\00\00\02i\01\04\01\00\00\00-\c5\0b\00\00\0a\00\00\00*\b5\07\00\00\0c\00\00\00\02r\01}\01\00\00\00#\fe\0b\00\00\d4\0b\00\00\ef\00\00\00\02x\01\05\12\ff\07\00\00\1c\0c\00\00\12+\08\00\00'\0c\00\00\12u\08\00\002\0c\00\00\12\93\08\00\00=\0c\00\00\13\da\02\00\00\d7\0b\00\00\05\00\00\00\02\d0\11\12\d3\07\00\00\f1\02\00\00\00\13\11\03\00\00\dc\0b\00\00\08\00\00\00\02\d4'\10I\08\00\00\1d\03\00\00\14\01(\03\00\00\00\13\11\03\00\00\e4\0b\00\00\0c\00\00\00\02\d3'\11\04\ed\00\06\9f\1d\03\00\00\14\00(\03\00\00\00\00\00\00\00\16\b4\0f\00\00\1a\08\00\00\16\c1\0f\00\00\22\08\00\00\16\d2\0f\00\00i\08\00\00\16\06\10\00\00q\08\00\00\16n\10\00\00\de\08\00\00\16\06\10\00\00\ec\08\00\00\16\8a\10\00\00\0b\09\00\00\16\97\10\00\00\15\09\00\00\16\bc\10\00\00\1d\09\00\00\16\cd\10\00\00\c9\09\00\00\16\cd\10\00\00^\0a\00\00\16\cd\10\00\00\ef\0a\00\00\16\cd\10\00\00{\0b\00\00\16\e3\10\00\00\cb\0b\00\00\16)\04\00\00\a7\0c\00\00\16I\04\00\00\c3\0c\00\00\16\bc\10\00\00\cb\0c\00\00\16\f9\10\00\00\e0\0c\00\00\16\06\11\00\00\e8\0c\00\00\16\13\11\00\00\f2\0c\00\00\16$\11\00\00\f8\0c\00\00\00\19\b2\03\00\00\02v\18\fd\02\00\00\00\17\a1\05\00\00\02\92a\02\00\00\18\fd\02\00\00\00\17\9e\01\00\00\02{\f2\0f\00\00\18\fd\02\00\00\18a\02\00\00\18O\02\00\00\18\dc\00\00\00\00\02\f7\0f\00\00\03\02\10\00\00\dc\01\00\00\02;\0a\00\02;\17\a2\02\00\00\02\7f\17\10\00\00\18\f2\0f\00\00\00\03\22\10\00\00\f5\05\00\00\02$0\08\02\1e\05\02\00\00\002\10\00\00\02\22\00\04\08\02\1f\05\11\00\00\00}\01\00\00\02 \00\05\8e\03\00\00\5c\10\00\00\02!\04\00\05,\00\00\00O\02\00\00\02#\00\00\03g\10\00\00\0a\01\00\00\01\cf\08E\00\00\00\07\02\19\e5\04\00\00\02o\18\bb\02\00\00\18\a7\02\00\00\18\17\10\00\00\18\5c\10\00\00\00\19\e0\02\00\00\02\80\18\f2\0f\00\00\00\17_\02\00\00\02\82\a8\10\00\00\18\fd\02\00\00\00\02\ad\10\00\00\03\b8\10\00\00\ce\01\00\00\02=\0a\00\02=\17\81\02\00\00\02\84l\02\00\00\18\a8\10\00\00\00\17D\03\00\00\02\87\04\01\00\00\18\dc\00\00\00\18\dc\00\00\00\00\17Z\00\00\00\02\90}\01\00\00\18\fd\02\00\00\18}\01\00\00\00\19\c4\02\00\00\02\85\18\a8\10\00\00\00\19k\01\00\00\02w\18\a7\02\00\00\00\17\83\01\00\00\02x\04\01\00\00\18\a7\02\00\00\00\19\df\03\00\00\02q\18\bb\02\00\00\18\04\01\00\00\00'\06\0d\00\00\0a\00\00\00\07\ed\03\00\00\00\00\9f9\03\00\00\02\83\01)\04\ed\00\00\9f0\00\00\00\02\83\01\fd\02\00\00%\99\01\00\00\02\83\01\d4\02\00\00\16T\0c\00\00\0f\0d\00\00\00'\12\0d\00\00\9e\05\00\00\04\ed\00\01\9f~\03\00\00\02\87\01)\04\ed\00\00\9f0\00\00\00\02\87\01\fd\02\00\00*\eb\08\00\00\d9\04\00\00\02\8d\01\17\10\00\00*\0b\09\00\00k\05\00\00\02\89\01\a7\02\00\00*)\09\00\00\f8\02\00\00\02\8c\01\f2\0f\00\00*U\09\00\00\08\03\00\00\02\88\01\bb\02\00\00*\d6\0a\00\00\d5\02\00\00\02\ef\01\a8\10\00\00*7\0c\00\00\10\04\00\00\02\f0\01l\02\00\00#\da\02\00\00/\0d\00\00\07\00\00\00\02\89\01\11\11\04\ed\00\00\9f\e6\02\00\00\00#4\03\00\00]\0d\00\00\03\00\00\00\02\88\01\15\11\04\ed\00\00\9f@\03\00\00\00-g\0d\00\00\83\03\00\00.\03\91\f8\00\81\03\00\00\02\e2\01\dc\00\00\00*\90\09\00\00\d3\06\00\00\02\90\01\8f\01\00\00*\bc\09\00\00\aa\04\00\00\02\97\01\d5\02\00\00*\da\09\00\00\85\03\00\00\02\96\01K\16\00\00#\ab\0b\00\00g\0d\00\00\07\00\00\00\02\90\01+\10s\09\00\00\c2\0b\00\00\00-\c8\0d\00\00\1a\01\00\00*\f8\09\00\00\93\05\00\00\02\9d\01l\02\00\00*\16\0a\00\00\c2\04\00\00\02\b1\01\5c\01\00\00*4\0a\00\00\d6\03\00\00\02\b2\01\04\01\00\00&\b7\04\00\00\02\9e\01&\00\00\00#\11\03\00\00\fe\0d\00\00\0d\00\00\00\02\9e\01.\14\00(\03\00\00\00#\11\03\00\00\87\0e\00\00\0c\00\00\00\02\b1\01/\14\02(\03\00\00\00\00-\eb\0e\00\00\c8\00\00\00*`\0a\00\00\93\05\00\00\02\bb\01l\02\00\00&\b7\04\00\00\02\bc\01&\00\00\00#\11\03\00\00!\0f\00\00\0d\00\00\00\02\bc\01.\14\00(\03\00\00\00-e\0f\00\00N\00\00\00*~\0a\00\00\c2\04\00\00\02\c7\01\5c\01\00\00#\11\03\00\00e\0f\00\00\0a\00\00\00\02\c7\010\14\02(\03\00\00\00\00\00-\b8\0f\00\00\aa\00\00\00.\03\91\f8\00\c6\05\00\00\02\db\01\dc\00\00\00*\aa\0a\00\00\a7\00\00\00\02\d3\01\04\01\00\00\00\00-X\11\00\00\16\01\00\00*\1f\0b\00\00\99\06\00\00\02\f3\01\5c\01\00\00#\11\03\00\00X\11\00\00\0e\00\00\00\02\f3\01(\10\02\0b\00\00(\03\00\00\00#\fe\0b\00\00~\11\00\00\f0\00\00\00\02\f7\01\03\12w\0b\00\00\1c\0c\00\00\12\a3\0b\00\00'\0c\00\00\12\ed\0b\00\002\0c\00\00\12\0b\0c\00\00=\0c\00\00\13\da\02\00\00~\11\00\00\05\00\00\00\02\d0\11\12K\0b\00\00\f1\02\00\00\00\13\11\03\00\00\83\11\00\00\08\00\00\00\02\d4'\10\c1\0b\00\00\1d\03\00\00\14\01(\03\00\00\00\13\11\03\00\00\8b\11\00\00\11\00\00\00\02\d3'\11\04\ed\00\06\9f\1d\03\00\00\14\00(\03\00\00\00\00\00\16\aa\15\00\00D\0d\00\00\16\06\10\00\00L\0d\00\00\16\13\11\00\00\8a\0d\00\00\16S\05\00\00\b2\0d\00\00\16S\05\00\00\dd\0d\00\00\16d\05\00\00\fc\0d\00\00\16\11\05\00\00^\0e\00\00\16n\10\00\00\7f\0e\00\00\16\12\06\00\00\e2\0e\00\00\16S\05\00\00\00\0f\00\00\16d\05\00\00\1f\0f\00\00\16\11\05\00\00_\0f\00\00\16w\09\00\00\b3\0f\00\00\16\e3\10\00\00\be\0f\00\00\16\11\05\00\00\d1\0f\00\00\16\e3\10\00\00\de\0f\00\00\16\11\05\00\00\f1\0f\00\00\16\22\05\00\008\10\00\00\16B\05\00\00E\10\00\00\16\11\05\00\00_\10\00\00\16\22\05\00\00\a2\10\00\00\16B\05\00\00\af\10\00\00\16n\10\00\00\ea\10\00\00\16\06\10\00\00\f9\10\00\00\16\8a\10\00\00\17\11\00\00\16\bb\15\00\00;\11\00\00\16\97\10\00\00E\11\00\00\16\bc\10\00\00M\11\00\00\16)\04\00\00R\12\00\00\16I\04\00\00n\12\00\00\16\bc\10\00\00v\12\00\00\16\f9\10\00\00\8b\12\00\00\16\fa\15\00\00\93\12\00\00\16\13\11\00\00\9d\12\00\00\16$\11\00\00\a3\12\00\00\00\17=\02\00\00\02}\f2\0f\00\00\18\fd\02\00\00\00\19w\06\00\00\02\8c\18\a7\02\00\00\18\d2\15\00\00\18\04\01\00\00\00\03\dd\15\00\00\b9\01\00\00\02\1c\04\02\02\19\05\17\00\00\00\8f\00\00\00\02\1a\00\05\22\00\00\00\8f\00\00\00\02\1b\01\00\19\f7\03\00\00\02y\18\a7\02\00\00\00'\b1\12\00\00\0a\00\00\00\07\ed\03\00\00\00\00\9fy\03\00\00\02\00\02)\04\ed\00\00\9f0\00\00\00\02\00\02\fd\02\00\00%\99\01\00\00\02\00\02\d4\02\00\00\16u\11\00\00\ba\12\00\00\00\02\d4\01\00\00\02\dc\00\00\00\00")
  (@custom ".debug_ranges" (after code) "\1a\08\00\00\83\08\00\00\86\08\00\00\0b\09\00\00\00\00\00\00\00\00\00\00\06\00\00\00\06\01\00\00\08\01\00\00\d7\01\00\00\d9\01\00\00x\04\00\00z\04\00\00J\05\00\00L\05\00\00\ee\07\00\00\f0\07\00\00\05\0d\00\00\06\0d\00\00\10\0d\00\00\12\0d\00\00\b0\12\00\00\b1\12\00\00\bb\12\00\00\00\00\00\00\00\00\00\00")
  (@custom ".debug_str" (after code) "y\00gen_index\00rand_index\00buffer_idx\00field_idx\00raw\00context\00Context\00nest\00unsigned short\00Point\00oxitortoise_next_int\00point_ahead_int\00unsigned int\00uphill_nest_scent\00PointInt\00rand_result\00scent_right\00chemical_right\00scent_left\00chemical_left\00Float\00oxitortoise_patch_at\00uint8_t\00uint16_t\00uint64_t\00uint32_t\00/home/anderiux/data/NetLogo/oxitortoise/oxitortoise/bench/models/ants\00oxitortoise_reset_ticks\00oxitortoise_get_ticks\00args\00oxitortoise_create_turtles\00AgentFieldDescriptor\00PatchIterator\00TurtleIterator\00pcolor\00plabel_color\00oxitortoise_scale_color\00world_to_max_pycor\00world_to_max_pxcor\00oxitortoise_make_all_turtles_iter\00oxitortoise_make_all_patches_iter\00oxitortoise_next_patch_from_iter\00oxitortoise_next_turtle_from_iter\00oxitortoise_drop_patch_iter\00oxitortoise_drop_turtle_iter\00context_to_updater\00Updater\00food_source_number\00unsigned char\00shim_setup\00oxitortoise_distance_euclidean_no_wrap\00who\00TurtleWho\00shim_go\00new_position\00gen\00hidden\00oxitortoise_is_nan\00_Bool\00oxitortoise_clear_all\00plabel\00uphill_chemical\00oxitortoise_update_tick\00oxitortoise_advance_tick\00next_patch\00recolor_patch\00oxitortoise_update_patch\00unsigned long long\00RustString\00oxitortoise_offset_distance_by_heading\00real_heading\00oxitortoise_normalize_heading\00size\00patch_here\00patch2_here\00shape_name\00next_turtle\00oxitortoise_update_turtle\00nest_scent_at_angle\00chemical_at_angle\00double\00distance\00workspace\00Workspace\00food\00oxitortoise_round\00context_to_world\00World\00occupancy_bitfield\00patch_id\00patch_here_id\00oxitortoise_get_default_turtle_breed\00point_ahead\00scent_ahead\00chemical_ahead\00PatchId\00TurtleId\00BreedId\00model_code.c\00world_to_patch_data\00base_data\00world_to_turtle_data\00PatchBaseData\00TurtleBaseData\00__ARRAY_SIZE_TYPE__\00oxitortoise_diffuse_8\00PatchGroup2\00patch2\00PatchGroup1\00patch1\00PatchGroup0\00TurtleGroup0\00patch0\00turtle0\00clang version 21.0.0git (https:/github.com/llvm/llvm-project 0f0079c29da4b4d5bbd43dced1db9ad6c6d11008)\00")
  (@custom ".debug_line" (after code) "g\0a\00\00\04\00\80\00\00\00\01\01\01\fb\0e\0d\00\01\01\01\01\00\00\00\01\00\00\01/home/anderiux\00\00.installs/emsdk/upstream/emscripten/cache/sysroot/include/bits/alltypes.h\00\01\00\00model_code.c\00\00\00\00\00\04\02\00\05\02\06\00\00\00\03\ce\01\01\05E\0a\95\05\19\03\87\7f<\05\09\03\09X\06\82\05E\06\03\ef\00\08\12\05\0e1\05\06\06X\03\aa~<\03\d6\01\ac\03\aa~.\05\15\06\03\d8\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\a7~<\03\d9\01\ac\03\a7~.\05)\06\03\db\01\d6\06\03\a5~<\03\db\01\ac\03\a5~.\06\03\dd\01\ba\06\03\a3~<\03\dd\01\ac\03\a3~.\05A\06\03\e2\01\08.\06\03\9e~<\05\14\03\e2\01\08 \03\9e~f\05%\06\03\d7\00\08 \06\03\a9\7f \05\02\06\03\e4\01f\05\01g\02\01\00\01\01\04\02\00\05\02\08\01\00\00\03\e6\01\01\05=\0a\08=\05\17\06X\05\16\06\83\06\03\97~\02:\01\05%\06\03\eb\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f4~\08X\05E\03\8e\01\9e\05\11/\06\03\8d~<\05\01\06\03\f4\01<\05\00\06\03\8c~\ac\05\01\03\f4\01.\02\01\00\01\01\04\02\00\05\02\d9\01\00\00\03\f5\01\01\05<\0a\08\9f\06\03\89~X\05=\06\03\e8\01\90\05\17\06 \05\16\06\83\06\03\97~\02=\01\05%\06\03\eb\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f4~\08\82\05E\03\8e\01\9e\05\11/\06\03\8d~<\05<\06\03\f8\01t\06\03\88~X\05=\06\03\e8\01\90\05\17\06 \05\16\06\83\06\03\97~\028\01\05%\06\03\eb\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f4~\08\82\05E\03\8e\01\9e\05\11/\06\03\8d~<\05;\06\03\f9\01t\06\03\87~X\05=\06\03\e8\01\90\05\17\06 \05\16\06\83\06\03\97~\02;\01\05%\06\03\eb\01\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\f4~\08X\05E\03\8e\01\9e\05\11/\06\03\8d~<\05\15\06\03\fa\01t\05&\06\90\03\86~\9e\05\16\06\03\fb\01\08\90\05\00\06\03\85~f\05\01\06\03\83\02\ac\02\0d\00\01\01\04\02\00\05\02z\04\00\00\03\84\02\01\05=\0a\08=\05\17\06X\05\16\06\83\06\03\f9}\02:\01\05%\06\03\89\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\d6~\08X\05\11\03\ad\01\ac\06\03\ef}t\05\01\06\03\92\02 \05\00\06\03\ee}\ac\05\01\03\92\02.\02\01\00\01\01\04\02\00\05\02L\05\00\00\03\93\02\01\05;\0a\08\9f\06\03\eb}X\05=\06\03\86\02\90\05\17\06 \05\16\06\83\06\03\f9}\02=\01\05%\06\03\89\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\d6~\08\82\05\11\03\ad\01\ac\06\03\ef}t\05;\06\03\96\02X\06\03\ea}X\05=\06\03\86\02\90\05\17\06 \05\16\06\83\06\03\f9}\028\01\05%\06\03\89\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\d6~\08\82\05\11\03\ad\01\ac\06\03\ef}t\05:\06\03\97\02X\06\03\e9}X\05=\06\03\86\02\90\05\17\06 \05\16\06\83\06\03\f9}\02;\01\05%\06\03\89\02\ac\05\06\06t\05U\06\86\057\06t\05-f\05'.\05\86\01<\05ht\05^f\05'.\05\15\06=\05\09\03\d6~\08X\05\11\03\ad\01\ac\06\03\ef}t\05\12\06\03\98\02X\05 \06\90\03\e8}\9e\05\13\06\03\99\02\08\90\05\00\06\03\e7}f\05\01\06\03\a1\02\ac\02\0d\00\01\01\04\02\00\05\02\f0\07\00\00\03\a2\02\01\05\19\0a\03\b8~\08\9e\05\02\03\cd\01t\05\04\88\a0\05\1a\d2\05\04\08$\05\1a~\05\19Q\05\1a\03y\c8\05\19\c1\06\03\cd}\08X\05%\06\03\d7\00J\05F\03\dc\01<\05\03\06X\05\09\06\03\ad~.\05g\03\d6\01t\05.\91\05\15\e7\06\03\c6}<\05\14\06\03\b9\02\c8\05\04@\05\19\03v\08\d6\06\03\cd}\08\ac\05F\03\b3\02J\05\03 \06\03\0cJ\05\19\87\05\18\a0\06\03\ba}\82\05@\03\c6\02J\05\03 \03\ba}.\05&\06\03\c9\02J\05\09\03\9b~\ac\05F\03\e4\01\e4\06\03\b8}J\05&\06\03\c9\02J\05F\f3\05&\d5\05F\bb\05\15\06J\05\1e\06\02Q\18\05\16\06<\05\1b\06\ef\05\10\06 \05\09\06\03\9f~<~\05O\03\f1\01\82\05\17\06\f2\05Y\02/\12\05O \05\17J\03\a7}\02*\01\05\13\06\03\da\02\90\06\03\a6}J\05!\06\03\db\02\ba\06\03\a5}<\05O\06\03\e1\02 \05q\06\08X\05O \05\17<\05Z\02-\12\05O \05\17J\03\9f}\02(\01\05\13\06\03\e2\02\90\06\03\9e}J\05!\06\03\e3\02\c8\06\03\9d}<\05O\06\03\e9\02 \05p\06\08X\05O \05\17<\05Z\02*\12\05O \05\17J\03\97}\02%\01\05\13\06\03\ea\02\90\06\03\96}J\05!\06\03\eb\02\c8\06\03\95}<\05\11\06\03\f1\02 \05$\06\f2\03\8f}J\05\1d\06\03\f2\02\08t\05\15g\05\13\06 \03\8d}<\05E\06\03\d4\01X\05\19\03\87\7f<\05\09\03\09X\06\82\05E\06\03\ef\00\ba\05\0e?\05\06\06X\03\aa~<\03\d6\01\ac\03\aa~.\05\15\06\03\d8\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\a7~<\03\d9\01\ac\03\a7~.\05)\06\03\db\01\d6\06\03\a5~<\03\db\01\ac\03\a5~.\06\03\dd\01\ba\06\03\a3~<\03\dd\01\ac\03\a3~.\05A\06\03\e2\01\08.\06\03\9e~<\05\14\03\e2\01\08 \03\9e~f\05\02\06\03\e4\01\08X\05\18\03\e2\00f\06\03\ba}\82\05@\03\c6\02\82\05\03 \06\035J\05\02\86\05#\83\05\02\06\9e\05\01\06g\02\0d\00\01\01\04\02\05\02\0a\00\05\02\07\0d\00\00\03\83\03\01\05\01\83\02\01\00\01\01\04\02\00\05\02\12\0d\00\00\03\86\03\01\05\19\0a\03\d4}\08\ba\05\18\03\b3\02t\05\19d\05\18\84\06\03\f2|\08J\05%\06\03\d7\00J\05E\03\b7\02<\05\02\06X\05\09\06\03\d2}.\05b\03\b0\02t\05J\06\90\05\1a\06/\05!\06t\05\07\9e\05\1e<\03\ef|<\06\03\97\03X\06\03\e9|<\05\1f\06\03\96\03X\05\1a@\05\00\06\03\e6|t\05 \03\9a\03\08\c8\05C\06?\05\9e\01\06\82\05\82\01t\05xf\05C.\05\1dJ\05\09\06\03\c7}\08\9e\05\15\03\bd\02\c8\05\1a\06\08 \03\df|f\05\1f\06\03\a3\03\c8\05\00\06\03\dd|t\05\17\06\03\a6\03\ba\05/M\058\06\f2\05\11 \05\0ff\05\06\06?\06\03\d4|\08\c8\05\09\06\03\e4\00\82\05M\03\cd\02\ba\05#/\06\03\ce|<\05\12\06\03\b3\03\ac\05\1a\06 \03\cd|<\03\b3\03\ac\05\06\06L\06\03\cb|\02,\01\05C\06\03\bb\03\90\05\9e\01\06\82\05\82\01t\05xf\05C.\05\1dJ\05\09\06\03\a9}\08\9e\05\15\03\db\02\c8\05\09\06t\03\c1|<\05\1f\06\03\c1\03\c8\05/?\058\06\f2\05\11 \05\0ff\05\05\06=\06\03\bb|.\05\09\06\03\e4\00 \05N\03\e3\02\9e\05\1c/\05\06\08?\06\03\b5|\02.\01\05\1f\06\03\d3\03X\05-\83\05\18s\056=\05\0f\06 \05\0df\05\19\06w\05-\83\05\12s\056=\05\0f\06 \05\0df\05Q\06?\05\18\06t\05'\06\02=\13\05\08\06t\05.\06\91\057\06\f2\05\10 \05\0ef\03\a3|<\05Q\06\03\e2\03 \05\18\06t\05(\06\028\13\05\08\06t\05\07f\05\10\06/\06\03\9c|\08\9e\05\03\06\03\e7\03 \06\03\99|\08\ba\05\18\06\03\8e\03 \06\03\f2|\08\9e\05E\03\8e\03J\05\02 \06\03\db\00J\06\03\97|\82\06\03\ec\03J\05\1f\06t\05\02<\05\1e\06\08[\05\17\a0\06\03\8f|\82\05E\03\f1\03J\05\02 \03\8f|.\05\09\06\03\e4\00J\05F\03\8f\03\d6\05\14K\05\19\03\e7|\08<\05\09\03\09X\06\82\05E\06\03\ef\00\08\12\05\0e1\05\06\06X\03\aa~<\03\d6\01\ac\03\aa~.\05\15\06\03\d8\01 \05\1a\06\f2\05\0f\06K\05\22\06\08 \03\a7~<\03\d9\01\ac\03\a7~.\05)\06\03\db\01\d6\06\03\a5~<\03\db\01\ac\03\a5~.\06\03\dd\01\ba\06\03\a3~<\03\dd\01\ac\03\a3~.\05A\06\03\e2\01\08.\06\03\9e~<\05\14\03\e2\01\08 \03\9e~f\05\02\06\03\e4\01\08X\05\17\03\8d\02f\06\03\8f|\82\05E\03\f1\03\82\05\02 \06R\85\05#\83\05\02\06\9e\05\01\06g\02\0d\00\01\01\04\02\05\02\0a\00\05\02\b2\12\00\00\03\80\04\01\05\01\83\02\01\00\01\01")
  (@custom "target_features" (after code) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)
