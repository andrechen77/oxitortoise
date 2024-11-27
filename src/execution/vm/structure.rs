//! Facilities for supporting structure programming.

use derive_more::derive::{From, TryInto};

use super::functionality;

pub use functionality::create_turtles::CrtAskFrame;

/// A stack frame holding information about the current location of execution.
/// This may have children frames that represent nested constructs; these are
/// represented by [`StructureFrame`]s on top of the current `StructureFrame` in
/// the frame stack.
#[derive(Debug, From, TryInto)]
#[try_into(owned, ref, ref_mut)]
pub enum StructureFrame {
    CrtAsk(CrtAskFrame),
}

/// Pops structure frames from the frame stack until a frame of the given type
/// is popped. That last frame is returned.
pub(super) fn pop_structure_frame<F>(frames: &mut Vec<StructureFrame>) -> Option<F>
where
    StructureFrame: TryInto<F>,
{
    while let Some(frame) = frames.pop() {
        if let Ok(frame) = frame.try_into() {
            return Some(frame);
        }
    }
    None
}

/// Pops structure frames from the frame stack until a frame of the given type
/// is found. The frame is left on the stack and a reference is returned.
pub(super) fn pop_until_structure_frame<F>(frames: &mut Vec<StructureFrame>) -> Option<&mut F>
where
    for<'a> &'a mut StructureFrame: TryInto<&'a mut F>,
{
    loop {
        let Some(frame) = frames.last_mut() else {
            return None;
        };
        if <&mut StructureFrame as TryInto<&mut F>>::try_into(frame).is_err() {
            frames.pop();
        } else {
            break;
        }
    }
    return Some(
        frames
            .last_mut()
            .expect("a frame should exist")
            .try_into()
            .unwrap_or_else(|_| panic!("previously checked that the frame was the correct type")),
    );
}

// TODO write tests for the pop structure frame methods
