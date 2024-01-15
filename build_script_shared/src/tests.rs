
#[macro_export]
macro_rules! compose_test {
    ($test_name:ident, $ty:ident<I> with parser $parser:path) => {
        #[test]
        fn $test_name() {
            for _ in 0..500 {
                use std::hash::{Hash, Hasher};
                use std::collections::hash_map::DefaultHasher;
                use fake::Fake;
                use $crate::error::ParserResult;
    
                let value: $ty<String> = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let new_value: ParserResult<&str, $ty<&str>> = $parser(s.as_str());
                let owned_value = new_value.map(|(s, v)| (s, v.map(ToString::to_string)));
                
                let mut hasher = DefaultHasher::new();
                if let Ok((_, ref v)) = owned_value {
                    v.hash(&mut hasher);
                }
                let old_hash = hasher.finish();
    
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let new_hash = hasher.finish();

                let result = Ok(("", value));

                if &owned_value != &result {
                    println!("{}", s);
                }
                
                assert_eq!(owned_value, result);
                assert_eq!(old_hash, new_hash);
            }
        }
    };
    ($test_name:ident, $ty:ident<I>) => {
        #[test]
        fn $test_name() {
            for _ in 0..500 {
                use std::hash::{Hash, Hasher};
                use std::collections::hash_map::DefaultHasher;
                use fake::Fake;
                use $crate::error::ParserResult;
                use $crate::parsers::ParserDeserialize;
    
                let value: $ty<String> = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let new_value: ParserResult<&str, $ty<&str>> = ParserDeserialize::parse(s.as_str());
                let owned_value = new_value.map(|(s, v)| (s, v.map(ToString::to_string)));
                
                let mut hasher = DefaultHasher::new();
                if let Ok((_, ref v)) = owned_value {
                    v.hash(&mut hasher);
                }
                let old_hash = hasher.finish();
    
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let new_hash = hasher.finish();

                let result = Ok(("", value));

                if &owned_value != &result {
                    println!("{}", s);
                }
                
                assert_eq!(owned_value, result);
                assert_eq!(old_hash, new_hash);
            }
        }
    };

    ($test_name:ident, $ty:ident) => {
        #[test]
        fn $test_name() {
            for _ in 0..500 {
                use std::hash::{Hash, Hasher};
                use std::collections::hash_map::DefaultHasher;
                use fake::Fake;
    
                let value: $ty = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let owned_value: ParserResult<&str, $ty> = ParserDeserialize::parse(s.as_str());
                
                let mut hasher = DefaultHasher::new();
                if let Ok((_, ref v)) = owned_value {
                    v.hash(&mut hasher);
                }
                let old_hash = hasher.finish();
    
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let new_hash = hasher.finish();

                let result = Ok(("", value));

                if &owned_value != &result {
                    println!("{}", s);
                }
                
                assert_eq!(owned_value, result);
                assert_eq!(old_hash, new_hash);
            }
        }
    };

    ($test_name:ident, $ty:ident no hash) => {
        #[test]
        fn $test_name() {
            for _ in 0..500 {
                use fake::Fake;
    
                let value: $ty = fake::Faker.fake();
                let s: String = value.serialize_to_string().unwrap();
                let owned_value: ParserResult<&str, $ty> = ParserDeserialize::parse(s.as_str());
                
                let result = Ok(("", value));

                if &owned_value != &result {
                    println!("{}", s);
                }

                assert_eq!(owned_value, result);
            }
        }
    };
}

pub use compose_test; 