//! Test of nested macro

use ::nested_attr::nested;

nested! {
    /// nested test struct
    #[derive(Debug)]
    pub struct Nested {
        /// mem a
        pub a: i32,
        /// struct mem b/B
        #[derive(Debug)]
        pub struct B{
            /// mem b1
            pub b1: i32,
            ///mem b3
            pub b3: char,
        },

        /// mem c
        pub c: usize,

        /// struct mem d
        pub struct d:
        #[derive(Debug)]
        /// mem struct D
        D {
            /// Nested struct E
            #[derive(Debug)]
            pub struct E {
                /// mem mt
                pub mt: ()
            }
        }
    }
}
