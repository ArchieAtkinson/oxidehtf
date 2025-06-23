use std::collections::HashMap;
use std::panic::Location;

use crate::test_runner::{data::suite::SuiteDataCollection, TestFailure};

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
    suites_data: SuiteDataCollection,
}

impl Measurements {
    pub fn new(suites_data: SuiteDataCollection) -> Self {
        Measurements {
            definitions: HashMap::new(),
            suites_data,
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

    #[track_caller]
    fn set_value_internal(&mut self, name: &str, value: DataTypes) -> Result<(), TestFailure> {
        let Some(mut def) = self.definitions.remove(name) else {
            return Err(TestFailure::MeasurementDoesntExist(name.into()));
        };

        def.value = Some(value.clone());

        self.suites_data
            .blocking_write(|f| Ok(f.current_suite_mut().insert_measurement(name, def.clone())))
            .expect("Failed to write measuremnt");

        if let DataTypes::F64(value) = value {
            if let Some((min, max)) = def.range {
                if value < min || value > max {
                    return Err(TestFailure::MeasurementNotInRange {
                        name: name.into(),
                        expected: (min, max),
                        found: value,
                        file: Location::caller().file(),
                        line: Location::caller().line(),
                    });
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

    #[track_caller]
    fn set_internal(self, value: DataTypes) -> Result<(), TestFailure> {
        self.manager
            .update_definition(&self.name, self.unit, self.range);
        self.manager.set_value_internal(&self.name, value)
    }

    #[track_caller]
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
