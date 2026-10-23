#[test]
fn create_and_insert_some_stuff() {
  use rustc_hash::FxHasher;
  use std::hash::{Hash, Hasher};
  use std::mem::transmute;

  fn hash_as_str(i: usize) -> u32 {
    let mut hasher = FxHasher::default();
    let s = i.to_string();
    s.hash(&mut hasher);
    let hash: u64 = hasher.finish();
    (hash >> 32) as u32 ^ (hash as u32)
  }

  let mut bf = BloomFilter::new();

  // Statically assert that ARRAY_SIZE is a multiple of 8, which
  // BloomStorageBool relies on.
  unsafe {
    transmute::<[u8; ARRAY_SIZE % 8], [u8; 0]>([]);
  }

  for i in 0_usize..1000 {
    bf.insert_hash(hash_as_str(i));
  }

  for i in 0_usize..1000 {
    assert!(bf.might_contain_hash(hash_as_str(i)));
  }

  let false_positives = (1001_usize..2000).filter(|i| bf.might_contain_hash(hash_as_str(*i))).count();

  assert!(false_positives < 190, "{} is not < 190", false_positives); // 19%.

  for i in 0_usize..100 {
    bf.remove_hash(hash_as_str(i));
  }

  for i in 100_usize..1000 {
    assert!(bf.might_contain_hash(hash_as_str(i)));
  }

  let false_positives = (0_usize..100).filter(|i| bf.might_contain_hash(hash_as_str(*i))).count();

  assert!(false_positives < 20, "{} is not < 20", false_positives); // 20%.

  bf.clear();

  for i in 0_usize..2000 {
    assert!(!bf.might_contain_hash(hash_as_str(i)));
  }
}
