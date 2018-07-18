#[doc(hidden)]
#[macro_export]
macro_rules! unexpected_token {
    ($message:expr) => {
        compile_error!($message)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! force_mismatch {
    () => {};
}

#[macro_export]
macro_rules! tree {
    (< $($rest:tt)*) => {
        open_angle! {
            $($rest)*
        }
    };
}

macro_rules! open_angle {
    (section name= $($rest:tt)*) => {
        open_section!($($rest)*)
    };

    ($name:ident $($rest:tt)*) => {
        tagged_element! {
            name=$name
            args=[]
            rest=[[ $($rest)* ]]
        }
    };
}

macro_rules! open_section {
    ($name:tt $($rest:tt)*) => {
        section_body! {
            name=$name
            rest=[[ $($rest)* ]]
        }
    };

    ($(rest: tt)*) => {
        unimplemented!()
    };
}

macro_rules! section_body {
    (
        name=$name:tt
        rest=[[ { $($tokens:tt)* }> ]]
    ) => {
        $crate::render_tree::Section(
            $name,
            tree!($($tokens)*)
        )
    };
}

macro_rules! tagged_element {
    {
        name = $name:tt
        args = [ $({ $key:ident = $value:tt })* ]
        rest = [[ > $($rest:tt)*]]
    } => {{
        $(
            force_mismatch!($key);
        )*

        unexpected_token!("Only block-based components take keys and values as arguments. Pass arguments to inline components as `args={...}`");
    }};

    {
        name = $name:tt
        args = []
        rest = [[ args = $($rest:tt)*]]
    } => {
        component_with_args! {
            name = $name
            rest = [[ $($rest)* ]]
        }
    };

    {
        name = $name:tt
        args = $args:tt
        rest = [[ $key:ident = $($rest:tt)*]]
    } => {
        tagged_element_values! {
            name = $name
            args = $args
            key = $key
            rest = [[ $($rest)* ]]
        }
    };

    {
        name = $name:tt
        args = $args:tt
        rest = [[ as $($rest:tt)*]]
    } => {
        unimplemented!()
    };

    {
        name = $name:tt
        args = $args:tt
        rest = [[ as $($rest:tt)*]]
    } => {
        unimplemented!()
    };

    {
        name = $name:tt
        args = $args:tt
        rest = [[ $($rest:tt)* ]]
    } => {{
        force_mismatch!($($rest)*);
        compile_error!(concat!("Unexpected tokens after <", $name, ". Expected `key=value`, `as {` or `as |`" ))
    }};
}

macro_rules! tagged_element_values {
    {
        name = $name:tt
        args = [ $($args:tt)* ]
        key = $key:tt
        rest = [[ $value:tt $($rest:tt)* ]]
    } => {
        tagged_element! {
            name = $name
            args = [ $($args)* { $key = $value } ]
            rest = [[ $($rest:tt)*]]
        }
    };
}

macro_rules! component_with_args {
    {
        name = $name:tt
        rest = [[ $value:tt $($rest:tt)* ]]
    } => {
        component_with_args_and_value! {
            name = $name
            value = $value
            rest = [[ $($rest:tt)*]]
        }
    };
}

macro_rules! component_with_args_and_value {
    {
        name = $name:tt
        value = $value:tt
        rest = [[ > $($rest:tt)* ]]
    } => {
        $crate::render_tree::Component($name, $value)

        // TODO: Combine with rest
    };
}

// (Add { left: { $($left:tt)* }, right: {} }) => {
//     $crate::render_tree::Document::with({ $($left)* })
// };

// (Add { left: {}, right: { $($right:tt)* } }) => {
//     $crate::render_tree::Document::with({ $($right)* })
// };

// (Add { left: { $($left:tt)* }, right: { $($right:tt)* } }) => {
//      $crate::render_tree::Document::empty()
//         .add({{ $($left)* }})
//         .add(tree!($($right)*))
// };

// (<section name=$token:tt { $($inner:tt)* }> $($rest:tt)*) => {
//     tree!(Add {
//         left: {
//             $crate::render_tree::Section(
//                 $token,
//                 { tree!($($inner)*) }
//             )
//         },
//         right: {
//             $($rest)*
//         }
//     })
// };

// (<section name=$token:tt $unexpected:tt $($rest:tt)*) => {
//     {
//         force_mismatch!($unexpected);
//         unexpected_token!("The nesting syntax for section is <section name=name { ... }>");
//     }
// };

// (< $name:ident args = $args:block > $($rest:tt)*) => {
//     tree!(Add {
//         left: {
//             $crate::render_tree::Component($name, $args)
//         },
//         right: {
//             $($rest)*
//         }
//     })
// };

// (<$name:ident as { $($inner:tt)* }> $($rest:tt)*) => {
//     tree!(Add {
//         left: {
//             $crate::render_tree::SimpleBlockComponent(
//                 $name, |doc: Document| -> Document { (tree! { $($inner)* }).render(doc) }
//             )
//         },
//         right: { $($rest)* }
//     })
// };

// (<$name:ident $args:block |$item:ident| { $($inner:tt)* }> $($rest:tt)*) => {
//     tree!(Add {
//         left: {
//             use $crate::render_tree::IterBlockHelper;

//             $crate::render_tree::IterBlockComponent(
//                 $name::args($args), |$item, doc: Document| -> Document { (tree! { $($inner)* }).render(doc) }
//             )
//         },
//         right: { $($rest)* }
//     })
// };

// (<$name:ident $($arg:ident = $value:tt)* |$item:ident| { $($inner:tt)* }> $($rest:tt)*) => {
//     tree!(Add {
//         left: {
//             let component = $name {
//                 $($arg: $value),*
//             };

//             $crate::render_tree::IterBlockComponent(
//                 component, |$item, doc: Document| -> Document { (tree! { $($inner)* }).render(doc) }
//             )
//         },
//         right: { $($rest)* }
//     })
// };

// (<$name:ident $args:block |$item:ident : $itemty:ty| { $($inner:tt)* }> $($rest:tt)*) => {
//     tree!(Add {
//         left: {
//             use $crate::render_tree::BlockHelper;

//             $crate::render_tree::BlockComponent(
//                 $name::args($args), |$item: $itemty, doc: Document| -> Document { (tree! { $($inner)* }).render(doc) }
//             )
//         },
//         right: { $($rest)* }
//     })
// };

// (< $name:ident $mismatch:tt > $($rest:tt)*) => {{
//     force_mismatch!($mismatch);
//     unexpected_token!(concat!("Unexpected block ", stringify!($mismatch), " for ", stringify!($name), ". Either you forgot `as` or you forgot args={{...}}"));
// }};

// // (< $name:ident $args:block $unexpected:tt $($rest:tt)*) => {
// //     {
// //         force_mismatch!($unexpected);
// //         unexpected_token!("The component syntax is <ComponentName {args}> or <ComponentName {args} |name| { ... }>");
// //     }
// // };

// ($token:ident $($rest:tt)*) => {
//     {
//         force_mismatch!($token);
//         compile_error!("Content must either be a string literal or enclosed in {}");
//     }
// };

// ($token:tt $($rest:tt)*) => {
//     tree!(Add {
//         left: {
//             $token
//         },
//         right: {
//             $($rest)*
//         }
//     })
// }
