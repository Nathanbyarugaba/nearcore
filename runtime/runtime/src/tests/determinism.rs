use near_primitives::types::AccountId;
use std::collections::HashMap;

#[test]
fn test_hashmap_determinism_risk() {
    let mut hm = HashMap::new();
    hm.insert("test1".parse::<AccountId>().unwrap(), 100);
    hm.insert("test2".parse::<AccountId>().unwrap(), 200);
    hm.insert("test3".parse::<AccountId>().unwrap(), 300);

    let mut keys: Vec<AccountId> = hm.keys().cloned().collect();
    keys.sort();

    assert_eq!(keys.len(), 3);
    assert_eq!(keys[0].as_str(), "test1");
    assert_eq!(keys[1].as_str(), "test2");
    assert_eq!(keys[2].as_str(), "test3");
}
