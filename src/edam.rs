use anyhow::Result;
use bimap::BiMap;
use lazy_static::lazy_static;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, f32::consts::E};

use crate::OutputFormat;

lazy_static! {
    #[derive(Debug)]
    pub static ref EDAM_MAP: EdamMap = EdamMap::new();
}

#[derive(Debug)]
// A struct to validate user specified EDAM information.
pub struct EdamMap {
    // TODO これはBiMapに変えたのであとで消す
    // Map of EDAM ID and Edam struct instance whose id is the key.
    // label_to_edam: HashMap<String, Edam>,
    bimap_id_label: BiMap<String, String>,
}

impl EdamMap {
    pub fn new() -> Self {
        let edam_str = include_bytes!("./EDAM_1.25.id_label.csv");
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&edam_str[..]);

        let mut edam_map: HashMap<String, Edam> = HashMap::new();
        let mut bimap = BiMap::new();
        for result in rdr.deserialize::<Edam>() {
            // resultがErrの時はpanicする
            match result {
                Ok(record) => {
                    // edam_map.insert(record.label.clone(), record.clone());
                    bimap.insert(record.id.clone(), record.label.clone());
                }
                Err(err) => panic!("Failed to initialize EDAM_MAP: {:?}", err),
            }
        }

        // Self { edam_map }
        Self {
            bimap_id_label: bimap,
        }
    }

    pub fn get_id_from_label(&self, label: &str) -> Option<String> {
        let id = self.bimap_id_label.get_by_right(label);

        id.cloned()
    }

    // check if the given pair of id and label exists in the EDAM table.
    pub fn check_id_and_label(&self, id: &str, label: &str) -> Result<bool> {
        let label_from_bimap = self.bimap_id_label.get_by_left(id);

        match label_from_bimap {
            Some(label_from_bimap) => {
                if label_from_bimap == label {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Edam {
    #[serde(rename = "Class ID")]
    id: String,
    #[serde(rename = "Preferred Label")]
    label: String,
}
