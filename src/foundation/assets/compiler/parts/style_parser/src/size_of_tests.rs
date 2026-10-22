/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::style_parser::cow_rc_str::CowRcStr;
use crate::style_parser::tokenizer::Token;

macro_rules! size_of_test {
    ($testname: ident, $t: ty, $expected_min_size: expr, $expected_max_size: expr) => {
        #[test]
        fn $testname() {
            let new = ::std::mem::size_of::<$t>();
            if new < $expected_min_size {
                panic!(
                    "Your changes have decreased the stack size of {} from {} to {}. \
                     Good work! Please update the expected size in {}.",
                    stringify!($t),
                    $expected_min_size,
                    new,
                    file!()
                )
            } else if new > $expected_max_size {
                panic!(
                    "Your changes have increased the stack size of {} from {} to {}. \
                     Please consider choosing a design which avoids this increase. \
                     If you feel that the increase is necessary, update the size in {}.",
                    stringify!($t),
                    $expected_max_size,
                    new,
                    file!()
                )
            }
        }
    };
    ($testname: ident, $t: ty, $expected_size: expr) => {
        size_of_test!($testname, $t, $expected_size, $expected_size);
    };
}

// Some of these assume 64-bit
size_of_test!(token, Token, 32);
size_of_test!(std_cow_str, std::borrow::Cow<'static, str>, 24, 32);
size_of_test!(cow_rc_str, CowRcStr, 16);

size_of_test!(tokenizer, crate::style_parser::tokenizer::Tokenizer, 72);
size_of_test!(parser_input, crate::style_parser::parser::ParserInput, 136);
size_of_test!(parser, crate::style_parser::parser::Parser, 16);
size_of_test!(source_position, crate::style_parser::SourcePosition, 8);
size_of_test!(parser_state, crate::style_parser::ParserState, 24);

size_of_test!(basic_parse_error, crate::style_parser::BasicParseError, 40, 48);
size_of_test!(parse_error_lower_bound, crate::style_parser::ParseError<()>, 40, 48);
