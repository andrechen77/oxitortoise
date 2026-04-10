use std::collections::BTreeSet;

use crate::mir::{MirType, MirTypeContents, Place};

/// Tracks within an MIR function which places are initialized.
#[derive(Debug, Default)]
pub struct InitTracker {
    init_places: BTreeSet<Place>,
}

impl InitTracker {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_init(&self, place: &Place) -> bool {
        // start with just the local and incrementally look for more fine-grained places
        let mut current_place = place.local.place();
        let mut remaining_projections = place.projections.iter();

        loop {
            if self.init_places.contains(&current_place) {
                return true;
            }
            if let Some(projection) = remaining_projections.next() {
                current_place = current_place.proj(*projection);
            } else {
                return false;
            }
        }
    }

    pub fn mark_init(&mut self, place: Place, type_of_place: impl Fn(&Place) -> MirType) {
        let local = place.local;

        self.init_places.retain(|p| !place.contains(p));
        self.init_places.insert(place);

        // if all children places are also init then we can mark the parent
        // place as init too. do this for the entire local variable
        if let Some(unconsolidated) = consolidate_init(local.place(), type_of_place(&local.place()))
        {
            self.init_places.extend(unconsolidated);
        }

        // attempts to consolidate initialization states of a place by
        // recursively checking if all the children of a place are initialized.
        // Returns None if the specified place is fully initialized, and a list
        // of initialized children if the specified place is not fully
        // initialized
        fn consolidate_init(place: Place, ty: MirType) -> Option<Vec<Place>> {
            match &ty.contents {
                MirTypeContents::Ptr(pointee_ty) => {
                    // if the pointee is not fully initialized, then neither is the
                    // current place, so forward the unconsolidated places. if the
                    // pointee is initialized, then the current place must be too.
                    // in either case, forwarding the return value is correct
                    consolidate_init(place.clone().proj_deref(), pointee_ty.clone())
                }
                MirTypeContents::Struct { fields, fields_are_complete, overall: _ } => {
                    let mut total_unconsolidated_places: Option<Vec<_>> = None;
                    for (offset, field_ty) in fields {
                        let field_place = place.clone().proj_field(*offset);
                        if let Some(unconsolidated_places) =
                            consolidate_init(field_place, field_ty.clone())
                        {
                            total_unconsolidated_places
                                .get_or_insert_default()
                                .extend(unconsolidated_places);
                        }
                    }
                    if *fields_are_complete {
                        total_unconsolidated_places
                    } else {
                        // even if all the fields are initialized, there might be
                        // more fields that we don't know about, so don't mark the
                        // place as init
                        Some(total_unconsolidated_places.unwrap_or_default())
                    }
                }
                _ => unimplemented!(""),
            }
        }
    }

    pub fn mark_deinit(&mut self, place: Place, type_of_place: impl Fn(&Place) -> MirType) {
        let mut current_place = place.local.place();
        let mut remaining_projections = place.projections.iter();
        loop {
            let maybe_next_proj = remaining_projections.next();
            let current_place_was_removed = self.init_places.remove(&current_place);
            let next_proj = match (current_place_was_removed, maybe_next_proj) {
                (false, Some(next_proj)) => {
                    // the place was not removed but we still have some projections to go, so keep searching
                    next_proj
                }
                (true, Some(next_proj)) => {
                    // we have removed a place that contains the given place.
                    // this means we need to break it apart into its own subplaces
                    // and continue
                    let ty = type_of_place(&current_place);
                    match &ty.contents {
                        MirTypeContents::Ptr(_) => {
                            self.init_places.insert(current_place.clone().proj_deref());
                        }
                        MirTypeContents::Struct { fields, .. } => {
                            for (offset, _) in fields {
                                self.init_places.insert(current_place.clone().proj_field(*offset));
                            }
                        }
                        _ => unimplemented!("can't handle this case"),
                    }
                    // return with the next projection to continue searching
                    next_proj
                }
                (true, None) => {
                    // there is no next projection, meaning current_place is the
                    // entire place. since we just removed current_place, we are
                    // done.
                    return;
                }
                (false, None) => {
                    // the place was not removed and there are no remaining projections
                    // so if the place being deinit is represented in the structure,
                    // it can be at most partially initialized.
                    panic!("place was deinit when it wasn't fully initialized")
                }
            };
            current_place = current_place.proj(*next_proj);
        }
    }
}
