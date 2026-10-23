
    use super::*;
    use crate::GraphemeClusterSegmenter;
    use icu_provider::prelude::*;
    use serde::Deserialize;

    /// `TestCase` is a struct used to store a single test case.
    /// Each test case has two attributes: `unseg` which denotes the unsegmented line, and `true_bies` which indicates the Bies
    /// sequence representing the true segmentation.
    #[derive(PartialEq, Debug, Deserialize)]
    struct TestCase {
        unseg: String,
        expected_bies: String,
        true_bies: String,
    }

    /// `TestTextData` is a struct to store a vector of `TestCase` that represents a test text.
    #[derive(PartialEq, Debug, Deserialize)]
    struct TestTextData {
        testcases: Vec<TestCase>,
    }

    #[derive(Debug)]
    struct TestText {
        data: TestTextData,
    }

    #[test]
    fn segment_file_by_lstm() {
        let lstm: DataResponse<SegmenterLstmAutoV1> = Baked
            .load(DataRequest {
                id: DataIdentifierBorrowed::for_marker_attributes(
                    DataMarkerAttributes::from_str_or_panic(
                        "Thai_codepoints_exclusive_model4_heavy",
                    ),
                ),
                ..Default::default()
            })
            .unwrap();
        let lstm = LstmSegmenter::new(lstm.payload.get(), GraphemeClusterSegmenter::new());

        // Importing the test data
        let test_text_data = serde_json::from_str(if lstm.grapheme.is_some() {
            include_str!("../../../../../integration/testdata/test_text_graphclust.json")
        } else {
            include_str!("../../../../../integration/testdata/test_text_codepoints.json")
        })
        .expect("JSON syntax error");
        let test_text = TestText {
            data: test_text_data,
        };

        // Testing
        for test_case in &test_text.data.testcases {
            let lstm_output = lstm
                .segment_str(&test_case.unseg)
                .bies
                .map(|is_e| if is_e { 'e' } else { '?' })
                .collect::<String>();
            println!("Test case      : {}", test_case.unseg);
            println!("Expected bies  : {}", test_case.expected_bies);
            println!("Estimated bies : {lstm_output}");
            println!("True bies      : {}", test_case.true_bies);
            println!("****************************************************");
            assert_eq!(
                test_case.expected_bies.replace(['b', 'i', 's'], "?"),
                lstm_output
            );
        }
    }
