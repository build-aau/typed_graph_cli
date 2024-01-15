pub trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl<I> ToSnakeCase for I 
where
    I: AsRef<str>
{
    /// Convert a string into a snake case notation
    /// 
    /// `HelloWorld => hello_world`
    /// 
    /// To better support accronyms multiple capital letters 
    /// in a row will not be intepreted as multiple words
    /// 
    /// `LOL => lol`
    /// 
    /// If the acronym ends in a lower case letter then the last letter is intepreted as being part of a new word
    /// 
    /// `BUILDGraph => build_graph`
    /// 
    /// Numbers and special charachters are ignored and inserted as is
    /// 
    /// `Today.theDateIs:D.12/12/2023 12:00:00 => today.the_date_is:_d.12/12/2023_12:00:00`
    /// 
    /// Any number of whitespaces will be joined into a single space
    /// 
    /// `How  AreYou => how_are_you`
    fn to_snake_case(&self) -> String {
        let s = self.as_ref();
        let chars: Vec<_> = s.chars().into_iter().collect();
        let mut new_s: Vec<char> = Vec::new();

        let mut last_upper = true;
        let mut last_whitespace = false;

        for i in 0..chars.len() {
            let c = chars[i];
            let next_c = chars.get(i + 1);
            
            if c.is_whitespace() {
                // Handle multiple whitespaces in a row
                if last_whitespace {
                    continue;
                }

                last_whitespace = true;
                new_s.push('_');
                last_upper = false;
                continue;
            }

            match (c, next_c) {
                // Prevent a rising or falling edge from making double _
                // `How  AreYou => how_are_you`
                // Where if this check is not here it would be
                // `How  AreYou => how__are_you`
                (_, _) if last_whitespace => (),
                
                (c, _) if !c.is_alphabetic() => (),
                // Look for falling edge
                // AAABcccc
                // in order to insert
                // AAA_Bccc
                // This does not apply to the first character
                (c, Some(n_c)) if c.is_uppercase() && n_c.is_lowercase() && i != 0 => {
                    new_s.push('_');
                }
                // Look for rising edge
                // aaaBCCCC
                // in order to insert
                // aaa_BCCC
                // This does not apply if the last charachter was a upper case letter
                (c, _) if c.is_uppercase() && !last_upper => {
                    new_s.push('_');
                },
                _ => ()
            }

            last_whitespace = false;

            if c.is_alphabetic() {
                last_upper = c.is_uppercase();
            }

            new_s.push(c.to_lowercase().next().unwrap());

        }

        new_s.into_iter().collect()
    }
}

#[test]
fn snake_case_test() {
    assert_eq!("HelloW;orld    How!Are  you".to_snake_case(), "hello_w;orld_how!_are_you");
    assert_eq!("ABCDEFGHIJK".to_snake_case(), "abcdefghijk");
    assert_eq!("ABCDefGHIJK".to_snake_case(), "abc_def_ghijk");
    assert_eq!("LOL".to_snake_case(), "lol");
    assert_eq!("Today.theDateIs:D.12/12/2023 12:00:00".to_snake_case(), "today.the_date_is:_d.12/12/2023_12:00:00");
    assert_eq!("BUILDGraph".to_snake_case(), "build_graph");
}