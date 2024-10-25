use flagset::{flags, FlagSet};

use crate::turtle::Turtle;

pub trait Update {
    /// Records in the updater that the specified properties of a turtle have
    /// changed to their new values. If this is called on a turtle that the
    /// updater hasn't seen before, the updater also records that the turtle has
    /// been created.
    fn update_turtle(&mut self, turtle: &Turtle, properties_to_update: FlagSet<TurtleProperty>);

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
        XCor,
        YCor,
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
            TurtleProperty::XCor => format!("XCor: {:?}", turtle.xcor()),
            TurtleProperty::YCor => format!("YCor: {:?}", turtle.ycor()),
            };
            updated_properties.push(value);
        }
        println!("Turtle {} updated: {{ {} }}", turtle.id(), updated_properties.join(", "));
    }

    fn get_update(&mut self) -> ! {
        todo!()
    }
}
