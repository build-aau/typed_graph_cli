use std::fmt::{Debug, Display};

use crate::error::{OwnedParserError, ParserError, ParserResult};
use crate::parsers::ParserSerialize;
use crate::CodePreview;

pub const TEST_ITERATION_COUNT: usize = 1000;

#[macro_export]
macro_rules! compose_test {
    ($test_name:ident, $ty:ident<I> with parser $parser:path) => {
        #[test]
        fn $test_name() {
            for _ in 0..$crate::tests::TEST_ITERATION_COUNT {
                use fake::Fake;
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                use $crate::error::ParserResult;

                let value: $ty<String> = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let input = $crate::InputMarker::new(s.as_str());
                let input_end = input.get_end();
                let new_value: ParserResult<_, $ty<_>> = $parser(input);
                let owned_value = new_value.map(|(s, v)| (s, v.map(|marker| marker.to_string())));

                let mut hasher = DefaultHasher::new();
                if let Ok((_, ref v)) = owned_value {
                    v.hash(&mut hasher);
                }
                let old_hash = hasher.finish();

                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let new_hash = hasher.finish();

                let result = Ok((input_end, value));

                $crate::tests::display_parser_test_debug(&owned_value, &result, &s);
                assert_eq!(owned_value, result);
                assert_eq!(old_hash, new_hash);
            }
        }
    };
    ($test_name:ident, $ty:ident<I>) => {
        #[test]
        fn $test_name() {
            for _ in 0..$crate::tests::TEST_ITERATION_COUNT {
                use fake::Fake;
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                use $crate::error::ParserResult;
                use $crate::parsers::ParserDeserialize;

                let value: $ty<String> = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let input = $crate::InputMarker::new(s.as_str());
                let input_end = input.get_end();
                let new_value: ParserResult<_, $ty<_>> = ParserDeserialize::parse(input);
                let owned_value = new_value.map(|(s, v)| (s, v.map(|marker| marker.to_string())));

                let mut hasher = DefaultHasher::new();
                if let Ok((_, ref v)) = owned_value {
                    v.hash(&mut hasher);
                }
                let old_hash = hasher.finish();

                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let new_hash = hasher.finish();

                let result = Ok((input_end, value));

                $crate::tests::display_parser_test_debug(&owned_value, &result, &s);
                assert_eq!(owned_value, result);
                assert_eq!(old_hash, new_hash);
            }
        }
    };

    ($test_name:ident, $ty:ident) => {
        #[test]
        fn $test_name() {
            for _ in 0..$crate::tests::TEST_ITERATION_COUNT {
                use fake::Fake;
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let value: $ty = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let input = $crate::InputMarker::new(s.as_str());
                let input_end = input.get_end();
                let owned_value: $crate::error::ParserResult<_, $ty> = ParserDeserialize::parse(input);

                let mut hasher = DefaultHasher::new();
                if let Ok((_, ref v)) = owned_value {
                    v.hash(&mut hasher);
                }
                let old_hash = hasher.finish();

                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let new_hash = hasher.finish();

                let result = Ok((input_end, value));

                $crate::tests::display_parser_test_debug(&owned_value, &result, &s);
                assert_eq!(owned_value, result);
                assert_eq!(old_hash, new_hash);
            }
        }
    };

    ($test_name:ident, $ty:ident no hash) => {
        #[test]
        fn $test_name() {
            for _ in 0..$crate::tests::TEST_ITERATION_COUNT {
                use fake::Fake;

                let value: $ty = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let input = $crate::InputMarker::new(s.as_str());
                let input_end = input.get_end();
                let owned_value: ParserResult<_, $ty> = ParserDeserialize::parse(input);

                let result = Ok((input_end, value));

                $crate::tests::display_parser_test_debug(&owned_value, &result, &s);
                assert_eq!(owned_value, result);
            }
        }
    };
}

pub use compose_test;

pub fn display_parser_test_debug<I, D>(
    owned_value: &ParserResult<I, D>, 
    result: &ParserResult<I, D>, s: &String
) 
where
    I: ToString + PartialEq + Clone, 
    D: PartialEq + ParserSerialize + Debug,
    OwnedParserError: From<ParserError<I>>
{
    if owned_value == result {
        return;
    }

    let input_with_linenumbers =  s
        .split('\n')
        .enumerate()
        .map(|(i, line)| format!("{i:>5} | {line}"))
        .collect::<Vec<_>>().join("\n");
    
    println!("Source:");
    println!("{input_with_linenumbers}");

    if let Err(ref e) = owned_value {
        let owned_error: crate::error::BUILDScriptError = e.clone().into();
        println!();
        println!("Parsing failed with:");
        println!("{}", owned_error);

        assert!(false, "Parser failed");
    } else if let (Ok((_, left)), Ok((_, right))) = (owned_value, result) {
        display_diff(left, right);
    }
}

pub fn display_diff<I0, I1>(left: &I0, right: &I1) 
where
    I0: Debug + ParserSerialize,
    I1: Debug + ParserSerialize,
{
    println!();
        println!("Dif of compose:");
        let left_s = left.serialize_to_string().unwrap();
        let right_s = right.serialize_to_string().unwrap();
        let diff = CodePreview::diff_string(&left_s, &right_s);
        println!("{diff}");

        println!();
        println!("Dif of debug:");
        let left_s: String = format!("{left:?}").split_inclusive(',').map(|line| format!("{line}\n")).collect();
        let right_s: String = format!("{right:?}").split_inclusive(',').map(|line| format!("{line}\n")).collect();
        let diff = CodePreview::diff_string(&left_s, &right_s);
        println!("{diff}");
}