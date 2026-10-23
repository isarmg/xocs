
    use super::*;

    #[test]
    fn loop_test() {
        #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Demo {
            a: u32,
            b: u8,
        }

        let mut raw_buf = [0u8; 64];
        let mut cobs_buf: CobsAccumulator<64> = CobsAccumulator::new();

        let ser = crate::to_slice_cobs(&Demo { a: 10, b: 20 }, &mut raw_buf).unwrap();

        if let FeedResult::Success { data, remaining } = cobs_buf.feed(ser) {
            assert_eq!(Demo { a: 10, b: 20 }, data);
            assert_eq!(remaining.len(), 0);
        } else {
            panic!()
        }
    }

    #[test]
    #[cfg(feature = "heapless")]
    fn double_loop_test() {
        #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Demo {
            a: u32,
            b: u8,
        }

        let mut cobs_buf: CobsAccumulator<64> = CobsAccumulator::new();

        let mut ser = crate::to_vec_cobs::<_, 128>(&Demo { a: 10, b: 20 }).unwrap();
        let ser2 = crate::to_vec_cobs::<_, 128>(&Demo {
            a: 256854231,
            b: 115,
        })
        .unwrap();
        ser.extend(ser2);

        let (demo1, ser) = if let FeedResult::Success { data, remaining } = cobs_buf.feed(&ser[..])
        {
            (data, remaining)
        } else {
            panic!()
        };

        assert_eq!(Demo { a: 10, b: 20 }, demo1);

        let demo2 = if let FeedResult::Success { data, remaining } = cobs_buf.feed(ser) {
            assert_eq!(remaining.len(), 0);
            data
        } else {
            panic!()
        };

        assert_eq!(Demo { a: 10, b: 20 }, demo1);
        assert_eq!(
            Demo {
                a: 256854231,
                b: 115
            },
            demo2
        );
    }

    #[test]
    #[cfg(feature = "heapless")]
    fn loop_test_ref() {
        #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Demo<'a> {
            a: u32,
            b: u8,
            c: &'a str,
        }

        let mut cobs_buf: CobsAccumulator<64> = CobsAccumulator::new();

        let ser = crate::to_vec_cobs::<_, 128>(&Demo {
            a: 10,
            b: 20,
            c: "test",
        })
        .unwrap();

        if let FeedResult::Success { data, remaining } = cobs_buf.feed_ref(&ser[..]) {
            assert_eq!(
                Demo {
                    a: 10,
                    b: 20,
                    c: "test"
                },
                data
            );
            assert_eq!(remaining.len(), 0);
        } else {
            panic!()
        }
    }

    #[test]
    #[cfg(feature = "heapless")]
    fn double_loop_test_ref() {
        #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Demo<'a> {
            a: u32,
            b: u8,
            c: &'a str,
        }

        let mut cobs_buf: CobsAccumulator<64> = CobsAccumulator::new();

        let mut ser = crate::to_vec_cobs::<_, 128>(&Demo {
            a: 10,
            b: 20,
            c: "test",
        })
        .unwrap();
        let ser2 = crate::to_vec_cobs::<_, 128>(&Demo {
            a: 256854231,
            b: 115,
            c: "different test",
        })
        .unwrap();
        ser.extend(ser2);

        let (data, ser) =
            if let FeedResult::Success { data, remaining } = cobs_buf.feed_ref(&ser[..]) {
                (data, remaining)
            } else {
                panic!()
            };

        assert!(
            Demo {
                a: 10,
                b: 20,
                c: "test"
            } == data
        );

        let demo2 = if let FeedResult::Success { data, remaining } = cobs_buf.feed_ref(ser) {
            assert!(remaining.is_empty());
            data
        } else {
            panic!()
        };

        // Uncommenting the below line causes the test to no-longer compile, as cobs_buf would then be mutably borrowed twice
        //assert!(Demo { a: 10, b: 20, c : "test" } == data);

        assert!(
            Demo {
                a: 256854231,
                b: 115,
                c: "different test"
            } == demo2
        );
    }

    #[test]
    #[cfg(feature = "heapless")]
    fn extend_unchecked_in_bounds_test() {
        // Test bug present in revision abcb407:
        // extend_unchecked may be passed slice with size 1 greater than accumulator buffer causing panic

        #[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Demo {
            data: [u8; 10],
        }

        let data = crate::to_vec_cobs::<_, 128>(&Demo { data: [0xcc; 10] }).unwrap();
        assert_eq!(data.len(), 12); // 1 byte for offset + 1 sentinel byte appended

        // Accumulator has 1 byte less space than encoded message
        let mut acc: CobsAccumulator<11> = CobsAccumulator::new();
        assert!(matches!(
            acc.feed::<Demo>(&data[..]),
            FeedResult::OverFull(_)
        ));

        // Accumulator is juuuuust right
        let mut acc: CobsAccumulator<12> = CobsAccumulator::new();
        assert!(matches!(
            acc.feed::<Demo>(&data[..]),
            FeedResult::Success { .. }
        ));
    }
