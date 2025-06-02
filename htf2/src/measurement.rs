use std::collections::HashMap;

use crate::TestFailure;

#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Volts,
}

#[derive(Debug, Clone)]
pub struct MeasurementDefinition {
    pub _name: String,
    pub unit: Option<Unit>,
    pub range: Option<(f64, f64)>,
}

pub struct Measurements {
    definitions: HashMap<String, MeasurementDefinition>,
    values: HashMap<String, f64>,
}

impl Measurements {
    pub fn new() -> Self {
        Measurements {
            definitions: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn measure(&mut self, name: &str) -> MeasurementSetter<'_> {
        self.definitions
            .entry(name.to_string())
            .or_insert_with(|| MeasurementDefinition {
                _name: name.to_string(),
                unit: None,
                range: None,
            });

        MeasurementSetter {
            manager: self,
            name: name.to_string(),
            temp_unit: None,
            temp_range: None,
        }
    }

    pub fn set_value(&mut self, name: &str, value: f64) -> Result<(), TestFailure> {
        if let Some(def) = self.definitions.get(name) {
            if let Some((min, max)) = def.range {
                if value < min || value > max {
                    return Err(TestFailure::MeasurementError);
                }
            }
            self.values.insert(name.to_string(), value);
        } else {
            return Err(TestFailure::MeasurementError);
        }
        Ok(())
    }

    pub(crate) fn update_definition(
        &mut self,
        name: &str,
        unit: Option<Unit>,
        range: Option<(f64, f64)>,
    ) {
        if let Some(def) = self.definitions.get_mut(name) {
            def.unit = unit.or(def.unit.clone());
            def.range = range.or(def.range);
        }
    }
}

pub struct MeasurementSetter<'a> {
    manager: &'a mut Measurements,
    name: String,
    temp_unit: Option<Unit>,
    temp_range: Option<(f64, f64)>,
}

impl<'a> MeasurementSetter<'a> {
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.temp_unit = Some(unit);
        self
    }

    pub fn in_range(mut self, min: f64, max: f64) -> Self {
        self.temp_range = Some((min, max));
        self
    }

    pub fn set(self, value: f64) -> Result<(), TestFailure> {
        self.manager
            .update_definition(&self.name, self.temp_unit, self.temp_range);
        self.manager.set_value(&self.name, value)
    }
}

impl Default for Measurements {
    fn default() -> Self {
        Self {
            definitions: Default::default(),
            values: Default::default(),
        }
    }
}
