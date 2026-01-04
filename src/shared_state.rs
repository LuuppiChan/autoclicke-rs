use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64},
    },
};

use crate::{cli::Cli, spammer::Spammer};

/// Error given when spammer is not found at the given index.
pub const SPAMMER_NOT_FOUND: &str = "Spammer not found";

pub fn init(cli: &Cli) -> SharedState {
    SharedState {
        left_enabled: Arc::new(AtomicBool::new(cli.enable_left)),
        right_enabled: Arc::new(AtomicBool::new(cli.enable_right)),
        fast_enabled: Arc::new(AtomicBool::new(cli.enable_fast)),
        click_counter: Arc::new(AtomicU64::new(0)),
        cps: Arc::new(AtomicU64::new(0)),
        spammers: Arc::new(Mutex::new(HashMap::new())),
    }
}

#[derive(Clone)]
pub struct SharedState {
    pub left_enabled: Arc<AtomicBool>,
    pub right_enabled: Arc<AtomicBool>,
    pub fast_enabled: Arc<AtomicBool>,
    pub click_counter: Arc<AtomicU64>,
    pub cps: Arc<AtomicU64>,
    pub spammers: Arc<Mutex<HashMap<String, Arc<Mutex<Spammer>>>>>,
}

impl SharedState {
    /// Toggles given spammer and returns the new state.
    /// Returns None if no spammer is at that location.
    pub fn toggle_spammer(&self, key: &str) -> Option<bool> {
        if let Some(spammer) = self.spammers.lock().unwrap().get(key) {
            let spammer = spammer.lock().unwrap();
            if spammer.is_enabled() {
                spammer.disable();
            } else {
                spammer.enable();
            }
            Some(spammer.is_enabled())
        } else {
            None
        }
    }

    /// Enables a spammer at the given index.
    /// Returns an error if there's no spammer at that index.
    pub fn enable_spammer(&self, key: &str) -> Result<(), &str> {
        if let Some(spammer) = self.spammers.lock().unwrap().get(key) {
            spammer.lock().unwrap().enable();
            Ok(())
        } else {
            Err(SPAMMER_NOT_FOUND)
        }
    }

    /// Disables a spammer at the given index.
    /// Returns an error if there's no spammer at that index.
    pub fn disable_spammer(&self, key: &str) -> Result<(), &str> {
        if let Some(spammer) = self.spammers.lock().unwrap().get(key) {
            spammer.lock().unwrap().disable();
            Ok(())
        } else {
            Err(SPAMMER_NOT_FOUND)
        }
    }

    pub fn add_spammer(&self, key: &str, spammer: Spammer) {
        let mut spammers = self.spammers.lock().unwrap();
        spammers.insert(key.to_owned(), Arc::new(Mutex::new(spammer)));
    }

    pub fn get_spammer(&self, key: &str) -> Option<Arc<Mutex<Spammer>>> {
        self.spammers.lock().unwrap().get(key).cloned()
    }

    pub fn is_enabled_spammer(&self, spammer_key: &str) -> Option<bool> {
        self.spammers
            .lock()
            .unwrap()
            .get(spammer_key)
            .map(|spammer| spammer.lock().unwrap().is_enabled())
    }
}
