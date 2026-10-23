
  extern crate test;
  use super::BloomFilter;

  #[derive(Default)]
  struct HashGenerator(u32);

  impl HashGenerator {
    fn next(&mut self) -> u32 {
      // Each hash is split into two twelve-bit segments, which are used
      // as an index into an array. We increment each by 64 so that we
      // hit the next cache line, and then another 1 so that our wrapping
      // behavior leads us to different entries each time.
      //
      // Trying to simulate cold caches is rather difficult with the cargo
      // benchmarking setup, so it may all be moot depending on the number
      // of iterations that end up being run. But we might as well.
      self.0 += (65) + (65 << super::KEY_SIZE);
      self.0
    }
  }

  #[bench]
  fn create_insert_1000_remove_100_lookup_100(b: &mut test::Bencher) {
    b.iter(|| {
      let mut gen1 = HashGenerator::default();
      let mut gen2 = HashGenerator::default();
      let mut bf = BloomFilter::new();
      for _ in 0_usize..1000 {
        bf.insert_hash(gen1.next());
      }
      for _ in 0_usize..100 {
        bf.remove_hash(gen2.next());
      }
      for _ in 100_usize..200 {
        test::black_box(bf.might_contain_hash(gen2.next()));
      }
    });
  }

  #[bench]
  fn might_contain_10(b: &mut test::Bencher) {
    let bf = BloomFilter::new();
    let mut r#gen = HashGenerator::default();
    b.iter(|| {
      for _ in 0..10 {
        test::black_box(bf.might_contain_hash(r#gen.next()));
      }
    });
  }

  #[bench]
  fn clear(b: &mut test::Bencher) {
    let mut bf = Box::new(BloomFilter::new());
    b.iter(|| test::black_box(&mut bf).clear());
  }

  #[bench]
  fn insert_10(b: &mut test::Bencher) {
    let mut bf = BloomFilter::new();
    let mut r#gen = HashGenerator::default();
    b.iter(|| {
      for _ in 0..10 {
        test::black_box(bf.insert_hash(r#gen.next()));
      }
    });
  }

  #[bench]
  fn remove_10(b: &mut test::Bencher) {
    let mut bf = BloomFilter::new();
    let mut r#gen = HashGenerator::default();
    // Note: this will underflow, and that's ok.
    b.iter(|| {
      for _ in 0..10 {
        bf.remove_hash(r#gen.next())
      }
    });
  }
