pub mod ordered_list_serde {
    use serde::{Serializer, Deserializer, Serialize, Deserialize};
    use std::collections::BTreeMap;

    type Container<T> = BTreeMap<usize, T>;

    pub fn serialize<S, T>(list: &Container<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize
    {
        let new_container: Vec<_> = list.values().collect();
        new_container.serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Container<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>
    {
        let new_container: Vec<T> = Deserialize::deserialize(deserializer)?;
        Ok(new_container.into_iter().enumerate().collect())
    }
}