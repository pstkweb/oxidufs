use std::{collections::HashMap, fmt};

use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct MountOptions(HashMap<String, Option<String>>);

impl MountOptions {
    pub fn get(&self, key: &str) -> Option<&Option<String>> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: String, value: Option<String>) {
        self.0.insert(key, value);
    }

    pub fn to_string_list(&self) -> Vec<String> {
        self.0
            .iter()
            .map(|(k, v)| match v {
                Some(val) => format!("{k}={val}"),
                None => k.clone(),
            })
            .collect()
    }
}

impl From<&str> for MountOptions {
    fn from(value: &str) -> Self {
        let map = value
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|opt| match opt.split_once('=') {
                Some((k, v)) => (k.to_string(), Some(v.to_string())),
                None => (opt.to_string(), None),
            })
            .collect();

        Self(map)
    }
}

impl From<HashMap<String, Option<String>>> for MountOptions {
    fn from(value: HashMap<String, Option<String>>) -> Self {
        Self(value)
    }
}

impl From<Vec<String>> for MountOptions {
    fn from(value: Vec<String>) -> Self {
        let map = value
            .iter()
            .filter(|s| !s.is_empty())
            .map(|opt| match opt.split_once('=') {
                Some((k, v)) => (k.to_string(), Some(v.to_string())),
                None => (opt.to_string(), None),
            })
            .collect();

        Self(map)
    }
}

impl fmt::Display for MountOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = self.to_string_list();
        parts.sort();

        write!(f, "{}", parts.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_empty_yields_no_keys() {
        let opts = MountOptions::from("");
        assert!(opts.to_string_list().is_empty());
    }

    #[test]
    fn from_str_flag_only_has_no_value() {
        let opts = MountOptions::from("rw");
        assert_eq!(opts.get("rw"), Some(&None));
    }

    #[test]
    fn from_str_key_value_keeps_value() {
        let opts = MountOptions::from("uid=1000");
        assert_eq!(opts.get("uid"), Some(&Some("1000".to_string())));
    }

    #[test]
    fn from_str_mixed_parses_each_entry() {
        let opts = MountOptions::from("rw,uid=1000,relatime");
        assert_eq!(opts.get("rw"), Some(&None));
        assert_eq!(opts.get("uid"), Some(&Some("1000".to_string())));
        assert_eq!(opts.get("relatime"), Some(&None));
    }

    #[test]
    fn from_str_filters_empty_segments() {
        let opts = MountOptions::from("rw,,relatime");
        assert_eq!(opts.to_string_list().len(), 2);
    }

    #[test]
    fn from_hashmap_round_trip() {
        let mut map = HashMap::new();
        map.insert("rw".to_string(), None);
        map.insert("uid".to_string(), Some("1000".to_string()));

        let opts = MountOptions::from(map);
        assert_eq!(opts.get("uid"), Some(&Some("1000".to_string())));
    }

    #[test]
    fn display_is_sorted_csv() {
        let opts = MountOptions::from("rw,uid=1000,relatime");
        assert_eq!(opts.to_string(), "relatime,rw,uid=1000");
    }

    #[test]
    fn to_string_list_formats_kv_with_equals() {
        let opts = MountOptions::from("rw,uid=1000");
        let mut list = opts.to_string_list();
        list.sort();
        assert_eq!(list, vec!["rw".to_string(), "uid=1000".to_string()]);
    }

    #[test]
    fn from_vec_empty_yields_no_keys() {
        let opts = MountOptions::from(Vec::<String>::new());
        assert!(opts.to_string_list().is_empty());
    }

    #[test]
    fn from_vec_flag_only_has_no_value() {
        let opts = MountOptions::from(vec!["rw".to_string()]);
        assert_eq!(opts.get("rw"), Some(&None));
    }

    #[test]
    fn from_vec_key_value_keeps_value() {
        let opts = MountOptions::from(vec!["uid=1000".to_string()]);
        assert_eq!(opts.get("uid"), Some(&Some("1000".to_string())));
    }

    #[test]
    fn from_vec_mixed_parses_each_entry() {
        let opts = MountOptions::from(vec![
            "rw".to_string(),
            "uid=1000".to_string(),
            "relatime".to_string(),
        ]);
        assert_eq!(opts.get("rw"), Some(&None));
        assert_eq!(opts.get("uid"), Some(&Some("1000".to_string())));
        assert_eq!(opts.get("relatime"), Some(&None));
    }

    #[test]
    fn from_vec_filters_empty_strings() {
        let opts = MountOptions::from(vec![
            "rw".to_string(),
            String::new(),
            "relatime".to_string(),
        ]);
        assert_eq!(opts.to_string_list().len(), 2);
    }

    #[test]
    fn from_vec_round_trips_with_to_string_list() {
        let original = MountOptions::from("rw,uid=1000,relatime");
        let list = original.to_string_list();
        let rebuilt = MountOptions::from(list);

        assert_eq!(rebuilt.get("rw"), Some(&None));
        assert_eq!(rebuilt.get("uid"), Some(&Some("1000".to_string())));
        assert_eq!(rebuilt.get("relatime"), Some(&None));
    }
}
