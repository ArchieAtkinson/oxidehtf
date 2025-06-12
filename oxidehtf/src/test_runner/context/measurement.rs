use std::collections::HashMap;

use crate::test_runner::{CurrentTestData, TestFailure};

#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Volts,
}

#[derive(Debug, Clone)]
pub enum DataTypes {
    F64(f64),
    String(String),
}

#[derive(Debug, Clone)]
pub struct MeasurementDefinition {
    pub unit: Option<String>,
    pub range: Option<(f64, f64)>,
    pub value: Option<DataTypes>,
}

pub struct Measurements {
    definitions: HashMap<String, MeasurementDefinition>,
    current_test: CurrentTestData,
}

impl Measurements {
    pub fn new(current_test: CurrentTestData) -> Self {
        Measurements {
            definitions: HashMap::new(),
            current_test,
        }
    }

    pub fn measure(&mut self, name: impl Into<String>) -> MeasurementSetter<'_> {
        let name = name.into();
        self.definitions
            .entry(name.clone())
            .or_insert_with(|| MeasurementDefinition {
                unit: None,
                range: None,
                value: None,
            });

        MeasurementSetter {
            manager: self,
            name,
            unit: None,
            range: None,
        }
    }

    fn set_value_internal(&mut self, name: &str, value: DataTypes) -> Result<(), TestFailure> {
        let Some(mut def) = self.definitions.remove(name) else {
            return Err(TestFailure::MeasurementError);
        };

        def.value = Some(value.clone());

        self.current_test
            .insert_measurement(name, def.clone())
            .unwrap();

        if let DataTypes::F64(value) = value {
            if let Some((min, max)) = def.range {
                if value < min || value > max {
                    return Err(TestFailure::MeasurementError);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn update_definition(
        &mut self,
        name: &str,
        unit: Option<String>,
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
    unit: Option<String>,
    range: Option<(f64, f64)>,
}

impl<'a> MeasurementSetter<'a> {
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    pub fn in_range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min, max));
        self
    }

    fn set_internal(self, value: DataTypes) -> Result<(), TestFailure> {
        self.manager
            .update_definition(&self.name, self.unit, self.range);
        self.manager.set_value_internal(&self.name, value)
    }

    pub fn set(self, value: f64) -> Result<(), TestFailure> {
        self.set_internal(DataTypes::F64(value))
    }

    pub fn set_str(self, value: impl Into<String>) -> Result<(), TestFailure> {
        self.set_internal(DataTypes::String(value.into()))
    }
}

impl std::fmt::Display for DataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F64(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
        }
    }
}
