use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::{events::Event, test_runner::TestData, TestFailure};

#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Volts,
}

#[derive(Debug, Clone)]
pub struct MeasurementDefinition {
    pub unit: Option<Unit>,
    pub range: Option<(f64, f64)>,
    pub value: Option<f64>,
}

pub struct Measurements {
    definitions: HashMap<String, MeasurementDefinition>,
    test_state: Arc<RwLock<TestData>>,
    event_tx: UnboundedSender<Event>,
}

impl Measurements {
    pub fn new(test_state: Arc<RwLock<TestData>>, event_tx: UnboundedSender<Event>) -> Self {
        Measurements {
            definitions: HashMap::new(),
            test_state,
            event_tx,
        }
    }

    pub fn measure(&mut self, name: &str) -> MeasurementSetter<'_> {
        self.definitions
            .entry(name.to_string())
            .or_insert_with(|| MeasurementDefinition {
                unit: None,
                range: None,
                value: None,
            });

        MeasurementSetter {
            manager: self,
            name: name.to_string(),
            unit: None,
            range: None,
        }
    }

    pub fn set_value(&mut self, name: &str, value: f64) -> Result<(), TestFailure> {
        let Some(mut def) = self.definitions.remove(name) else {
            return Err(TestFailure::MeasurementError);
        };

        def.value = Some(value);

        self.test_state
            .blocking_write()
            .current_test()
            .measurements
            .insert(name.into(), def.clone());
        if self.event_tx.send(Event::UpdatedTestData).is_err() {
            return Err(TestFailure::MeasurementError);
        }

        if let Some((min, max)) = def.range {
            if value < min || value > max {
                return Err(TestFailure::MeasurementError);
            }
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
    unit: Option<Unit>,
    range: Option<(f64, f64)>,
}

impl<'a> MeasurementSetter<'a> {
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn in_range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min, max));
        self
    }

    pub fn set(self, value: f64) -> Result<(), TestFailure> {
        self.manager
            .update_definition(&self.name, self.unit, self.range);
        self.manager.set_value(&self.name, value)
    }
}
