use flagset::{flags, FlagSet};

use crate::{patch::Patch, turtle::Turtle};

// TODO is there a better way to send the updates without having to create
// a reference to the agent being updated? for example, we could just save the
// agent id and OR all the changed flags together to find all the properties
// that were changed

pub trait Update {
    /// Records in the updater that the specified properties of a turtle have
    /// changed to their new values. If this is called on a turtle that the
    /// updater hasn't seen before, the updater also records that the turtle has
    /// been created.
    fn update_turtle(&mut self, turtle: &Turtle, properties_to_update: FlagSet<TurtleProperty>);

    /// Records in the updater that the specified properties of a patch have
    /// changed to their new values.
    fn update_patch(&mut self, patch: &Patch, properties_to_update: FlagSet<PatchProperty>);

    /// TODO Gets all the updates recorded in this updater since the last time
    /// this method was called.
    fn get_update(&mut self) -> !;
}

flags! {
    pub enum TurtleProperty: u32 {
        Breed,
        Color,
        Heading,
        LabelColor,
        Hidden,
        PenSize,
        PenMode,
        Shape,
        Size,
        Position,
    }
}

flags! {
    pub enum PatchProperty: u32 {
        Pcolor,
        Plabel,
        PlabelColor,
    }
}

pub struct PrintUpdate;

impl Update for PrintUpdate {
    fn update_turtle(&mut self, turtle: &Turtle, properties_to_update: FlagSet<TurtleProperty>) {
        let mut updated_properties = vec![];
        for property in properties_to_update {
            let value = match property {
                TurtleProperty::Breed => format!("Breed: {:?}", turtle.breed()),
                TurtleProperty::Color => format!("Color: {:?}", turtle.color()),
                TurtleProperty::Heading => format!("Heading: {:?}", turtle.heading()),
                TurtleProperty::LabelColor => format!("LabelColor: {:?}", turtle.label_color()),
                TurtleProperty::Hidden => format!("Hidden: {:?}", turtle.hidden()),
                TurtleProperty::PenSize => todo!(),
                TurtleProperty::PenMode => todo!(),
                TurtleProperty::Shape => format!("Shape: {:?}", turtle.shape()),
                TurtleProperty::Size => format!("Size: {:?}", turtle.size()),
                TurtleProperty::Position => format!("Position: {:?}", turtle.position()),
            };
            updated_properties.push(value);
        }
        println!(
            "Turtle {} updated: {{ {} }}",
            turtle.who(),
            updated_properties.join(", ")
        );
    }

    fn update_patch(&mut self, patch: &Patch, properties_to_update: FlagSet<PatchProperty>) {
        let mut updated_properties: Vec<String> = vec![];
        for property in properties_to_update {
            let value = match property {
                PatchProperty::Pcolor => format!("Color: {:?}", patch.get_pcolor()),
                PatchProperty::Plabel => format!("Label: {:?}", patch.get_plabel()),
                PatchProperty::PlabelColor => format!("LabelColor: {:?}", patch.get_plabel_color()),
            };
            updated_properties.push(value);
        }
        println!(
            "Patch {} updated: {{ {} }}",
            patch.position(),
            updated_properties.join(", ")
        );
    }

    fn get_update(&mut self) -> ! {
        todo!()
    }
}
