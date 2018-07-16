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
    (Add { left: { $($left:tt)* }, right: {} }) => {
        $($left)*
    };

    (Add { left: {}, right: { $($right:tt)* } }) => {
        $($right)*
    };

    (Add { left: { $($left:tt)* }, right: { $($right:tt)* } }) => {
        $($left)* + tree!($($right)*)
    };

    (<line { $($inner:tt)* }> $($rest:tt)*) => {
        tree!(Add {
            left: {
                $crate::render_tree::Line(
                    tree!($($inner)*)
                )
            },
            right: { $($rest)* }
        })
    };

    (<section name=$token:tt { $($inner:tt)* }> $($rest:tt)*) => {
        tree!(Add {
            left: {
                $crate::render_tree::Section(
                    $token,
                    { tree!($($inner)*) }
                )
            },
            right: {
                $($rest)*
            }
        })
    };

    (<section name=$token:tt $unexpected:tt $($rest:tt)*) => {
        {
            force_mismatch!($unexpected);
            unexpected_token!("The nesting syntax for section is <section name=name { ... }>");
        }
    };

    (< $name:ident $args:block > $($rest:tt)*) => {
        tree!(Add {
            left: {
                $crate::render_tree::Component($name, $args)
            },
            right: {
                $($rest)*
            }
        })
    };

    (< $name:ident $args:block $unexpected:tt $($rest:tt)*) => {
        {
            force_mismatch!($unexpected);
            unexpected_token!("The component syntax is <ComponentName {args}>");
        }
    };

    ($token:tt $($rest:tt)*) => {
        tree!(Add {
            left: {
                $crate::render_tree::Render::render($token, $crate::render_tree::Document::empty())
            },
            right: {
                $($rest)*
            }
        })
    }
}
