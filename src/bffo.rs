use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    #[derive(Debug)]
    pub static ref BFFO_MAP: BffoMap = BffoMap::new();
}

const EDAM_ID_PREFIX: &str = "http://edamontology.org/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BffoFormat {
    pub label: String,
    pub id: String,
}

#[derive(Debug)]
pub struct BffoMap {
    by_tataki_key: HashMap<String, BffoFormat>,
    by_edam_id: HashMap<String, BffoFormat>,
}

// A internal struct to deserialize the BFFO table.
// The `edam_label` column is not declared here; it exists for human readers and
// for the test that checks the table against the EDAM ontology.
#[derive(Debug, Deserialize)]
struct BffoRecord {
    tataki_key: String,
    edam_id: String,
    bffo_slug: String,
    bffo_url: String,
}

impl BffoMap {
    fn new() -> Self {
        let bffo_str = include_bytes!("./tataki_formats_edam_bffo.csv");
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&bffo_str[..]);

        let mut by_tataki_key = HashMap::new();
        let mut by_edam_id = HashMap::new();

        for result in rdr.deserialize::<BffoRecord>() {
            // Not returning a Result: the table is embedded at compile time, so a
            // failure here is a defect in the build artifact, not in the user's input.
            let record = match result {
                Ok(record) => record,
                Err(err) => panic!("Failed to initialize BFFO_MAP: {:?}", err),
            };

            // Rows without a BFFO term are left out of the map rather than falling
            // back to their EDAM term, which would mix the two id namespaces in one field.
            if record.bffo_url.is_empty() {
                continue;
            }

            let format = BffoFormat {
                label: record.bffo_slug,
                // The url is taken as written instead of being built from the slug,
                // so a change to the BFFO url scheme is a change to the table alone.
                id: record.bffo_url,
            };

            if !record.edam_id.is_empty() {
                by_edam_id.insert(
                    format!("{}{}", EDAM_ID_PREFIX, record.edam_id),
                    format.clone(),
                );
            }
            by_tataki_key.insert(record.tataki_key, format);
        }

        Self {
            by_tataki_key,
            by_edam_id,
        }
    }

    pub fn get_by_tataki_key(&self, tataki_key: &str) -> Option<&BffoFormat> {
        self.by_tataki_key.get(&tataki_key.to_lowercase())
    }

    pub fn get_by_edam_id(&self, edam_id: &str) -> Option<&BffoFormat> {
        self.by_edam_id.get(edam_id)
    }

    /// Looks up a BFFO term by the tataki module name, then by the EDAM id.
    ///
    /// - `tataki_key`: a built-in parser name or a compression format name
    /// - `edam_id`: the EDAM Class ID the module reported, if any
    pub fn resolve(&self, tataki_key: Option<&str>, edam_id: Option<&str>) -> Option<&BffoFormat> {
        // The EDAM id is not the only key, because BFFO covers formats EDAM has no
        // term for (BGZF); it is not the first key either, because falling back to it
        // resolves parser aliases such as `gff` without listing them a second time.
        tataki_key
            .and_then(|key| self.get_by_tataki_key(key))
            .or_else(|| edam_id.and_then(|id| self.get_by_edam_id(id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The columns the map itself does not read are verified here, so that a drift
    // between the table, the EDAM ontology and the parsers breaks the build instead
    // of silently changing what tataki reports.
    #[derive(Debug, Deserialize)]
    struct TableRow {
        tataki_key: String,
        edam_id: String,
        edam_label: String,
    }

    fn read_table() -> Vec<TableRow> {
        let bffo_str = include_bytes!("./tataki_formats_edam_bffo.csv");
        csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&bffo_str[..])
            .deserialize::<TableRow>()
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to read the BFFO table")
    }

    fn bffo(label: &str, id: &str) -> BffoFormat {
        BffoFormat {
            label: label.to_string(),
            id: id.to_string(),
        }
    }

    #[test]
    fn resolves_a_bffo_term_from_a_tataki_parser_name() {
        let result = BFFO_MAP.get_by_tataki_key("bam");

        assert_eq!(result, Some(&bffo("BAM", "https://bffo.org/format/BAM/")));
    }

    #[test]
    fn resolves_a_bffo_term_from_an_uppercase_parser_name() {
        let result = BFFO_MAP.get_by_tataki_key("BAM");

        assert_eq!(result, Some(&bffo("BAM", "https://bffo.org/format/BAM/")));
    }

    #[test]
    fn returns_none_for_a_format_that_has_no_bffo_term() {
        let result = BFFO_MAP.get_by_tataki_key("html");

        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_an_unknown_parser_name() {
        let result = BFFO_MAP.get_by_tataki_key("no_such_format");

        assert_eq!(result, None);
    }

    #[test]
    fn resolves_a_bffo_term_from_an_edam_id() {
        let result = BFFO_MAP.get_by_edam_id("http://edamontology.org/format_1975");

        assert_eq!(result, Some(&bffo("GFF3", "https://bffo.org/format/GFF3/")));
    }

    #[test]
    fn resolves_a_format_that_has_no_edam_term() {
        let result = BFFO_MAP.get_by_tataki_key("bgzf");

        assert_eq!(result, Some(&bffo("BGZF", "https://bffo.org/format/BGZF/")));
    }

    #[test]
    fn falls_back_to_the_edam_id_when_the_parser_name_is_an_alias() {
        let result = BFFO_MAP.resolve(Some("gff"), Some("http://edamontology.org/format_1975"));

        assert_eq!(result, Some(&bffo("GFF3", "https://bffo.org/format/GFF3/")));
    }

    #[test]
    fn prefers_the_tataki_parser_name_over_the_edam_id() {
        let result = BFFO_MAP.resolve(Some("bgzf"), Some("http://edamontology.org/format_1929"));

        assert_eq!(result, Some(&bffo("BGZF", "https://bffo.org/format/BGZF/")));
    }

    #[test]
    fn resolves_a_bffo_term_from_the_edam_id_alone() {
        let result = BFFO_MAP.resolve(None, Some("http://edamontology.org/format_2573"));

        assert_eq!(result, Some(&bffo("SAM", "https://bffo.org/format/SAM/")));
    }

    #[test]
    fn returns_none_when_neither_key_matches() {
        let result = BFFO_MAP.resolve(Some("bzip2"), None);

        assert_eq!(result, None);
    }

    #[test]
    fn every_edam_term_in_the_table_matches_the_edam_ontology() {
        for row in read_table() {
            if row.edam_id.is_empty() {
                continue;
            }

            let id = format!("{}{}", EDAM_ID_PREFIX, row.edam_id);
            let matches = crate::edam::EDAM_MAP
                .correspondence_check_id_and_label(&id, &row.edam_label)
                .unwrap();

            assert!(
                matches,
                "row '{}' gives {} the label {:?}, which is not its EDAM preferred label",
                row.tataki_key, row.edam_id, row.edam_label
            );
        }
    }

    #[test]
    fn every_format_key_in_the_table_is_a_parser_or_a_compression_format() {
        const COMPRESSION_KEYS: [&str; 3] = ["gzip", "bzip2", "bgzf"];

        for row in read_table() {
            if COMPRESSION_KEYS.contains(&row.tataki_key.as_str()) {
                continue;
            }

            assert!(
                crate::parser::from_str_to_parser(&row.tataki_key).is_ok(),
                "row '{}' is neither a built-in parser name nor a compression format",
                row.tataki_key
            );
        }
    }
}
