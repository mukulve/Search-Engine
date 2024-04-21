use anyhow::{Ok, Result};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
};

pub struct Dictionary {
    words: HashSet<String>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            words: HashSet::new(),
        }
    }

    pub fn read_dictionary(&mut self) -> Result<()> {
        let file = File::open("words.csv")?;
        let reader = BufReader::new(file);
        let words_mutex = Arc::new(Mutex::new(&mut self.words));

        reader
            .lines()
            .par_bridge()
            .for_each_with(words_mutex, |words_mutex, line| {
                let mut words = words_mutex.lock().unwrap();
                words.insert(line.unwrap().to_lowercase());
            });

        Ok(())
    }

    pub fn is_word(&self, word: &str) -> bool {
        self.words.contains(word)
    }
}
