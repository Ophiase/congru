use std::io::Write;
use std::path::PathBuf;

use crate::catalog::{Family, METHODS, Model};
use crate::measure::{Summary, register, track};
use crate::report::{self, Results, Row};
use crate::store::Store;
use crate::suite::{SITUATIONS, Seeds, sequences};

/// One run of the benchmark: what is computed, what is already known, and
/// the page rewritten after every method.
#[derive(Debug)]
pub struct Session {
    models: Vec<Model>,
    pool: Seeds,
    seeds: u64,
    store: Store,
    page: PathBuf,
    results: Results,
}

impl Session {
    pub fn new(models: Vec<Model>, pool: Seeds, seeds: u64, store: Store, page: PathBuf) -> Self {
        Self {
            models,
            pool,
            seeds,
            store,
            page,
            results: Results {
                seeds,
                registration: Vec::new(),
                tracking: Vec::new(),
            },
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        for index in 0..self.models.len() {
            self.registration(index)?;
            self.tracking(index)?;
        }
        self.publish()
    }

    fn registration(&mut self, index: usize) -> anyhow::Result<()> {
        for method in METHODS
            .iter()
            .filter(|method| method.family == Family::OneShot)
        {
            {
                let Self {
                    models,
                    pool,
                    seeds,
                    store,
                    results,
                    ..
                } = &mut *self;
                let model = &models[index];
                for situation in &SITUATIONS {
                    let key = format!(
                        "registration\t{}\t{}\t{}",
                        model.name, method.name, situation.name
                    );
                    let summary = cell(store, key, || {
                        Summary::of(&register(method, model, situation, pool.range(*seeds)))
                    })?;
                    let row = Row {
                        model: model.name,
                        situation: situation.name,
                        class: situation.class,
                        method: method.name,
                        summary,
                    };
                    results.registration.push(row);
                }
            }
            self.publish()?;
        }
        Ok(())
    }

    fn tracking(&mut self, index: usize) -> anyhow::Result<()> {
        for method in METHODS.iter().filter(|method| method.follows) {
            {
                let Self {
                    models,
                    pool,
                    seeds,
                    store,
                    results,
                    ..
                } = &mut *self;
                let model = &models[index];
                for sequence in sequences(model.span) {
                    let key = format!(
                        "tracking\t{}\t{}\t{}",
                        model.name, method.name, sequence.name
                    );
                    let range = pool.range(seeds.div_ceil(10));
                    let summary = cell(store, key, || {
                        Summary::of(&track(method, model, &sequence, range))
                    })?;
                    let row = Row {
                        model: model.name,
                        situation: sequence.name,
                        class: sequence.name,
                        method: method.name,
                        summary,
                    };
                    results.tracking.push(row);
                }
            }
            self.publish()?;
        }
        Ok(())
    }

    fn publish(&self) -> anyhow::Result<()> {
        std::fs::write(&self.page, report::page(&self.results, &self.models)?)?;
        std::fs::write(self.page.with_extension("csv"), report::csv(&self.results)?)?;
        Ok(())
    }
}

/// The summary under `key`, from the store when it is there, computed and
/// stored otherwise.
fn cell(
    store: &mut Store,
    key: String,
    compute: impl FnOnce() -> Summary,
) -> anyhow::Result<Summary> {
    if let Some(known) = store.get(&key) {
        return Ok(known);
    }
    let _ = writeln!(std::io::stderr().lock(), "{}", key.replace('\t', " · "));
    let summary = compute();
    store.put(key, summary)?;
    Ok(summary)
}
