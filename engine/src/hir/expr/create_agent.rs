//! The `create-turtles` command and friends.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, TurtleBreedId,
        build_mir::translate_expr,
    },
    mir,
    sim::turtle::TurtleId,
};

#[derive(Debug, Clone)]
pub struct CreateTurtles {
    pub workspace: Box<ExprKind>,
    pub rng: Box<ExprKind>,
    /// The breed of turtles to create.
    pub breed: TurtleBreedId,
    /// The number of turtles to create.
    pub num_turtles: Box<ExprKind>,
    /// The closure representing the commands to run for each created turtle.
    pub body: Box<ExprKind>,
}

impl Expr for CreateTurtles {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.rng);
        visitor(&self.num_turtles);
        visitor(&self.body);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.rng.as_mut());
        visitor(self.num_turtles.as_mut());
        visitor(self.body.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let CreateTurtles { workspace, rng, breed, num_turtles, body } = self;
        p.add_fn_call("create_turtles", |p| {
            p.add_fn_arg_with(|p| {
                if let Some(b) = names.turtle_breeds().get(breed) {
                    write!(p, "{}#{}", breed, b.name)
                } else {
                    write!(p, "{:?}", breed)
                }
            })?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| num_turtles.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| body.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl CreateTurtles {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = translate_expr(builder, &self.workspace)?;
        let rng_local = translate_expr(builder, &self.rng)?;
        let num_turtles_local = translate_expr(builder, &self.num_turtles)?;

        // it is statically known that the body is a closure taking turtles
        let ExprKind::Closure(closure) = self.body.as_ref() else {
            panic!("expected body to be a closure literal, got: {:?}", self.body);
        };
        let body_local = closure.write_mir_execution_with_static_types::<TurtleId, ()>(builder);

        let operation = mir::Operation::CallHostFunction {
            function: &create_turtles::FN_INFO,
            args: vec![
                mir::PlaceOperand::Copy(workspace_local.place()),
                mir::PlaceOperand::Copy(rng_local.place()),
                mir::PlaceOperand::Copy(num_turtles_local.place()),
                mir::PlaceOperand::Move(body_local),
            ],
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

mod create_turtles {
    use crate::{
        exec::jit::JitCallback,
        mir::HostFunctionInfo,
        sim::{
            topology::Point,
            turtle::{TurtleBreedId, TurtleId},
            value::NlFloat,
        },
        util::{reflection::Reflect, rng::CanonRng},
        workspace::Workspace,
    };

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "create_turtles",
        parameter_types: &[
            <&mut Workspace>::TYPE,
            <&mut CanonRng>::TYPE,
            TurtleBreedId::TYPE,
            NlFloat::TYPE,
            <JitCallback<'static, TurtleId, ()>>::TYPE,
        ],
        return_type: <()>::TYPE,
        link_name: "create_turtles",
        link_addr: call as *const u8,
    };

    pub fn call(
        workspace: &mut Workspace,
        rng: &mut CanonRng,
        breed: TurtleBreedId,
        count: NlFloat,
        mut birth_command: JitCallback<TurtleId, ()>,
    ) {
        let new_turtles = workspace.world.turtles.create_turtles(
            breed,
            count.to_u64_round_to_zero(),
            Point { x: NlFloat::new(0.0), y: NlFloat::new(0.0) },
            rng,
        );

        let mut iter = new_turtles.into_shuffler();
        while let Some(turtle) = iter.next(rng) {
            birth_command.call_mut(workspace, rng, turtle);
        }
    }
}
