use log::{debug, info};
use reqwest::header::ValueDrain;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

pub struct Bundle {
    resource: String,
    entries: BTreeMap<String, Entry>,
}

impl Bundle {
    pub fn get_amount() {}

    pub fn get(&self, id: &String) -> Option<&Entry> {
        self.entries.get(id)
    }

    pub fn get_entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values().into_iter()
    }
}

impl From<(String, &Value)> for Bundle {
    fn from(value: (String, &Value)) -> Self {
        let empty: Vec<Value> = Vec::new();

        let entries = value
            .1
            .get("entry")
            .and_then(|entry| entry.as_array())
            .unwrap_or(&empty);

        let entries = entries
            .iter()
            .fold(BTreeMap::<String, Entry>::new(), |mut acc, entry| {
                let request = entry.get("request").unwrap();

                let version = u64::from_str(
                    request
                        .get("url")
                        .and_then(|url| url.as_str())
                        .unwrap()
                        .split('/')
                        .last()
                        .unwrap(),
                )
                .unwrap();

                let id = entry
                    .get("fullUrl")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .split('/')
                    .last()
                    .unwrap()
                    .to_string();

                acc.entry(id.clone())
                    .or_insert(Entry::new(id))
                    .insert(version, entry.clone());
                acc
            });

        Self {
            resource: value.0,
            entries,
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.versions
            .clone()
            .into_values()
            .zip(other.versions.clone().into_values())
            .all(|(s, o)| compare_entries(&normalize_json(&s), &normalize_json(&o)))
    }
}
fn compare_entries(s: &serde_json::Value, o: &serde_json::Value) -> bool {
    // Extract and compare fullUrl
    let full_url = s.get("fullUrl").zip(o.get("fullUrl"));
    let full_url_match = match full_url {
        Some((a, b)) => {
            let is_match = a == b;
            log_mismatch(is_match, "fullUrl", a, b);
            is_match
        }
        None => {
            eprintln!("Warning: 'fullUrl' field is missing in one or both entries.");
            false
        }
    };

    // Extract and compare resource after removing ignored fields
    let resource_match = s
        .get("resource")
        .zip(o.get("resource"))
        .map(|(a, b)| (clean_resource(a), clean_resource(b)))
        .and_then(|resource| {
            let resource_match = resource.0 == resource.1;
            log_mismatch(resource_match, "resouce", &resource.0, &resource.1);
            Some(resource_match)
        })
        .unwrap_or(true);

    full_url_match && resource_match
}
// Cleans a resource JSON object by removing specified fields
fn clean_resource(resource: &serde_json::Value) -> serde_json::Value {
    let mut cleaned = resource.clone();

    if let Some(obj) = cleaned.as_object_mut() {
        obj.remove("authored");

        if let Some(meta) = obj.get_mut("meta").and_then(|m| m.as_object_mut()) {
            meta.remove("lastUpdated");
            meta.remove("source");
        }
    }

    cleaned
}
fn extract_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value.get(field).and_then(|v| v.as_str()).map(String::from)
}

/// Recursively normalizes a JSON value by sorting arrays
fn normalize_json(value: &Value) -> Value {
    match value {
        Value::Array(arr) => {
            let mut sorted_arr: Vec<Value> = arr.iter().map(normalize_json).collect();
            sorted_arr.sort_by(|a, b| a.to_string().cmp(&b.to_string())); // Sort elements lexicographically
            Value::Array(sorted_arr)
        }
        Value::Object(map) => {
            let normalized_map: serde_json::Map<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), normalize_json(v)))
                .collect();
            Value::Object(normalized_map)
        }
        _ => value.clone(), // Return primitive values unchanged
    }
}

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    versions: HashMap<u64, Value>,
}

impl Entry {
    pub fn get_current(&self) -> Option<&Value> {
        let mut versions: Vec<(&u64, &Value)> = self.versions.iter().collect();
        versions.sort_by_key(|k| k.0);
        versions.last().map(|entry| entry.1)
    }

    pub fn get_versions(&self) -> impl Iterator<Item = (&u64, &Value)> {
        let mut versions: Vec<(&u64, &Value)> = self.versions.iter().collect();
        versions.sort_by_key(|k| k.0);
        versions.into_iter()
    }

    pub fn get_version(&self, version: &u64) -> Option<&Value> {
        self.versions.get(version)
    }

    pub fn len(&self) -> usize {
        self.versions.len()
    }

    pub fn new(id: String) -> Self {
        Self {
            id,
            versions: HashMap::new(),
        }
    }

    fn insert(&mut self, version: u64, clone: Value) -> Option<Value> {
        self.versions.insert(version, clone)
    }
}

fn log_mismatch(
    is_match: bool,
    field: &str,
    expected: &serde_json::Value,
    actual: &serde_json::Value,
) {
    if !is_match {
        if !is_match {
            eprintln!(
                "Mismatch in field: {}\nExpected:\n{}\n\nActual:\n{}\n\n",
                field,
                serde_json::to_string_pretty(&expected).unwrap(),
                serde_json::to_string_pretty(&actual).unwrap()
            );
        }
    }
}
