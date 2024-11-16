use std::{
    collections::{hash_map::Entry, HashMap},
    mem,
    ops::{Index, IndexMut},
    rc::Rc,
};

use crate::value;

/// Describes the location of a certain variable in an agent of type `A`.
#[derive(Debug, PartialEq, Eq, Hash)] // TODO equality derives don't work
pub enum VariableDescriptor<A> {
    BuiltIn(fn(&A) -> value::Value),
    Custom(VarIndex),
}

impl<A> Clone for VariableDescriptor<A> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<A> Copy for VariableDescriptor<A> {}

/// Describes the location of a certain variable in the custom variables of
/// an agent of type `A`.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct VarIndex(usize);

/// Maps between variable names and variable indices for an agent of type A,
/// allowing the engine to work directly with indices instead of strings. This
/// struct should be associated with a subcategory of agent.
#[derive(Debug, Default)]
pub struct VariableMapper<A> {
    /// Maps from a variable name to its ID.
    name_to_descriptor: HashMap<Rc<str>, VariableDescriptor<A>>,
    /// Maps a custom variable's ID to its name in source code. The length of
    /// this vector is the variable ID that will be assigned to the next
    /// declared custom variable.
    custom_idx_to_name: Vec<Rc<str>>,
}

impl<A> VariableMapper<A> {
    pub fn new() -> Self {
        Self {
            name_to_descriptor: HashMap::new(),
            custom_idx_to_name: Vec::new(),
        }
    }

    /// Sets the custom variable mapping to have the names in `names`.
    ///
    /// Returns a vector with the same length as `names`, where each element is
    /// either `None` if the corresponding new variable is new (i.e. newly
    /// declared) or some `VarIndex` if it should take the value of the existing
    /// variable at that index.
    pub fn declare_custom_variables(&mut self, names: Vec<Rc<str>>) -> Vec<Option<VarIndex>> {
        let mut new_name_to_descriptor = HashMap::new();

        let mut new_idx_to_old_idx = Vec::new();

        for name in &names {
            // get the new and old indices for the variable with this name
            let new_idx = VarIndex(new_name_to_descriptor.len());
            let old_idx = match self.name_to_descriptor.get(name) {
                None => None,
                Some(VariableDescriptor::Custom(old_id)) => Some(*old_id),
                Some(VariableDescriptor::BuiltIn(_)) => {
                    panic!("Attempted to set custom variable mapping with built-in variable name \"{}\"", name);
                }
            };

            // create the mapping from new to old index
            new_idx_to_old_idx.push(old_idx);

            // add the mapping new_name_to_descriptor
            match new_name_to_descriptor.entry(name.clone()) {
                Entry::Occupied(e) => {
                    // TODO should probably just return an error or something?
                    // we can also probably assume that the compiler has statically
                    // make sure that custom variables have no name conflicts.
                    panic!(
                        "Attempted to declare custom variable with conflicting name \"{}\"",
                        e.key()
                    );
                }
                Entry::Vacant(e) => {
                    e.insert(VariableDescriptor::Custom(new_idx));
                }
            }
        }

        // replace the old mappings with the new mappings
        self.name_to_descriptor
            .retain(|_, descriptor| !matches!(descriptor, VariableDescriptor::Custom(_)));
        self.name_to_descriptor.extend(new_name_to_descriptor);
        self.custom_idx_to_name = names;

        new_idx_to_old_idx
    }

    /// # Panics
    ///
    /// Panics if the variable name has already been used.
    pub fn declare_built_in_variable(&mut self, name: Rc<str>, getter: fn(&A) -> value::Value) {
        match self.name_to_descriptor.entry(name) {
            Entry::Occupied(e) => {
                panic!(
                    "Attempted to declare built-in variable with conflicting name \"{}\"",
                    e.key()
                );
            }
            Entry::Vacant(e) => {
                e.insert(VariableDescriptor::BuiltIn(getter));
            }
        }
    }

    pub fn look_up_variable(&self, name: &str) -> Option<VariableDescriptor<A>> {
        self.name_to_descriptor.get(name).copied()
    }

    /// Panics if the `VariableId` does not correspond to a valid variable.
    pub fn get_name(&self, id: VarIndex) -> &str {
        self.custom_idx_to_name[id.0].as_ref()
    }
}

/// Holds the values of the non-built-in variables for an agent. An instance of
/// this struct is associated with each agent.
#[derive(Debug, Default)]
pub struct CustomAgentVariables {
    values: Vec<value::Value>,
}

impl CustomAgentVariables {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn reset_all(&mut self) {
        for value in &mut self.values {
            value.reset();
        }
    }

    /// Sets the variable mapping.
    ///
    /// The new number of custom variables is the length of `new_to_old_idxs`.
    /// Each element of `new_to_old_idxs` is either `None` if the corresponding
    /// variable is new (i.e. newly declared) or some [`VarIndex`] if it should
    /// take the value of the existing variable at that index.
    pub fn set_variable_mapping(&mut self, new_to_old_idxs: &[Option<VarIndex>]) {
        let mut new_values = Vec::with_capacity(new_to_old_idxs.len());
        for &mapping in new_to_old_idxs {
            match mapping {
                Some(id) => new_values.push(mem::take(&mut self.values[id.0])),
                None => new_values.push(value::Value::default()),
            }
        }
        self.values = new_values;
    }
}

impl Index<VarIndex> for CustomAgentVariables {
    type Output = value::Value;

    fn index(&self, index: VarIndex) -> &Self::Output {
        &self.values[index.0]
    }
}

impl IndexMut<VarIndex> for CustomAgentVariables {
    fn index_mut(&mut self, index: VarIndex) -> &mut Self::Output {
        &mut self.values[index.0]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct Agent {
        built_in_var: f64,
    }

    impl Agent {
        fn get_built_in_var(&self) -> value::Value {
            value::Float::new(self.built_in_var).into()
        }
    }

    #[test]
    fn test_variable_mapper() {
        let mut mapper = VariableMapper::<Agent>::new();

        // declare built-in variable
        let built_in_name: Rc<str> = Rc::from("builtin");
        mapper.declare_built_in_variable(built_in_name.clone(), Agent::get_built_in_var);

        // declare custom variables
        let custom_names = vec![Rc::from("custom0"), Rc::from("custom1")];
        let custom_mapping = mapper.declare_custom_variables(custom_names.clone());

        let agent = Agent { built_in_var: 42.0 };

        // check built-in variable
        let Some(VariableDescriptor::BuiltIn(getter)) = mapper.look_up_variable(&built_in_name)
        else {
            panic!("Expected built-in variable descriptor");
        };
        assert_eq!(getter(&agent), value::Float::new(42.0).into());

        // check custom variables
        for (i, name) in custom_names.iter().enumerate() {
            let Some(VariableDescriptor::Custom(var_index)) = mapper.look_up_variable(name) else {
                panic!("Expected custom variable descriptor");
            };
            assert_eq!(var_index, VarIndex(i));
        }

        // check new-to-old mapping
        assert_eq!(custom_mapping, vec![None, None]);
    }

    #[test]
    fn test_change_custom_variable_mapping() {
        let mut mapper = VariableMapper::<Agent>::new();

        // declare built-in variable
        let built_in_name: Rc<str> = Rc::from("builtin");
        mapper.declare_built_in_variable(built_in_name.clone(), Agent::get_built_in_var);

        // declare initial custom variables
        let initial_custom_names = vec![Rc::from("custom0"), Rc::from("custom1")];
        let initial_custom_mapping = mapper.declare_custom_variables(initial_custom_names);

        // check initial custom variable mapping
        assert_eq!(initial_custom_mapping, vec![None, None]);

        // declare new custom variables, reusing one old name and adding two new
        // ones
        let new_custom_names = vec![
            Rc::from("custom1"),
            Rc::from("custom2"),
            Rc::from("custom3"),
        ];
        let new_custom_mapping = mapper.declare_custom_variables(new_custom_names.clone());

        // check new custom variable mapping
        assert_eq!(new_custom_mapping, vec![Some(VarIndex(1)), None, None]);

        // ensure built-in mappings are not affected
        let Some(VariableDescriptor::BuiltIn(getter)) = mapper.look_up_variable(&built_in_name)
        else {
            panic!("expected built-in variable descriptor");
        };
        let agent = Agent { built_in_var: 42.0 };
        assert_eq!(getter(&agent), value::Float::new(42.0).into());

        // ensure old mappings that do not appear in new mappings are forgotten
        assert!(mapper.look_up_variable("custom0").is_none());

        // ensure new mappings are correct
        for (i, name) in new_custom_names.iter().enumerate() {
            let Some(VariableDescriptor::Custom(var_index)) = mapper.look_up_variable(name) else {
                panic!("expected custom variable descriptor");
            };
            assert_eq!(var_index, VarIndex(i));
        }
    }

    #[test]
    fn test_change_custom_variable_mapping_preserves_values() {
        let mut mapper = VariableMapper::<Agent>::new();

        // declare initial custom variables
        let initial_custom_names = vec![Rc::from("custom0"), Rc::from("custom1")];
        let initial_custom_mapping = mapper.declare_custom_variables(initial_custom_names.clone());

        // create CustomAgentVariables and set initial values
        let mut custom_vars = CustomAgentVariables::new();
        custom_vars.set_variable_mapping(&initial_custom_mapping);
        custom_vars[VarIndex(0)] = value::Float::new(1.0).into();
        custom_vars[VarIndex(1)] = value::Float::new(2.0).into();

        // declare new custom variables, reusing one old name and adding a new one
        let new_custom_names = vec![Rc::from("custom1"), Rc::from("custom2")];
        let new_custom_mapping = mapper.declare_custom_variables(new_custom_names.clone());

        // change the variable mapping in CustomAgentVariables
        custom_vars.set_variable_mapping(&new_custom_mapping);

        // ensure values are preserved for variables with the same name
        assert_eq!(custom_vars[VarIndex(0)], value::Float::new(2.0).into()); // custom1
        assert_eq!(custom_vars[VarIndex(1)], value::Value::default()); // custom2 is new
    }
}
