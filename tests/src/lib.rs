//! Test of nested macro

use ::nested_attr::nested;

nested! {
    #![derive(Debug)]
    //! Part of nested struct.
    /// nested test struct
    pub struct Nested {
        /// mem a
        pub a: i32,
        /// struct mem b/B
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
        /// mem struct D
        D {
            /// Nested struct E
            pub struct E {
                /// mem mt
                pub mt: ()
            },
            /// Unit.
            pub struct F
        }
    }
}
