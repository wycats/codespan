#[doc(hidden)]
#[macro_export]
macro_rules! unexpected_token {
    ($message:expr,trace = $trace:tt,tokens = $token:tt $($tokens:tt)*) => {{
        force_mismatch!($token);
        macro_trace!($message, $trace);
    }};

    ($message:expr,trace = $trace:tt,tokens =) => {{
        macro_trace!($message, $trace);
    }};

    ($($rest:tt)*) => {{
        compile_error!("Invalid call to unexpected_token");
    }};
}

#[doc(hidden)]
#[allow(unused_macros)]
#[macro_export]
macro_rules! macro_trace {
    ($message:expr, [ $({ $($trace:tt)* })* ]) => {{
        compile_error!(concat!(
            $message,
            "\nMacro trace: ",

            $(
                $(
                    stringify!($trace),
                    " ",
                )*
                "-> ",
            )*
        ))
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! force_mismatch {
    () => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! unimplemented_branch {
    ($message:expr, trace = $trace:tt,tokens = $($tokens:tt)*) => {{
        unexpected_token!(concat!("Unimplemented branch: ", $message), trace = $trace, tokens = $($tokens)*);
    }};

    ($($rest:tt)*) => {{
        compile_error("Invalid call to unimplemented_branch");
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! unexpected_eof {
     { $message:expr, trace = [ $($trace:tt)* ] } => {
        compile_error!(concat!("Unexpected end of macro: ", $message, "\nMacro trace: ", stringify!($($trace)*)))
    };

    ($($rest:tt)*) => {{
        compile_error("Invalid call to unexpected_eof");
    }}
}

#[macro_export]
macro_rules! tree {
    {
        trace = [ $($trace:tt)* ]
        rest = [[ < $($rest:tt)* ]]
    } => {
        open_angle! {
            trace = [ $($trace)* { open_angle } ]
            rest = [[ $($rest)* ]]
        }
    };

    {
        trace = [ $($trace:tt)* ]
        rest = [[ $token:tt $($rest:tt)* ]]
    } => {{
        let left = $crate::render_tree::Render::into_fragment($token);

        let right = tree! {
            trace = [ $($trace)* { next token } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(left, right)
    }};

    {
        trace = $trace:tt
        rest = [[  ]]
    } => {
        $crate::render_tree::Empty
    };

    {
        trace = $trace:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unimplemented_branch!("tree", trace = $trace, tokens = $($rest)*)
    };

    ($($rest:tt)*) => {
        tree! {
            trace = [ { tree } ]
            rest = [[ $($rest)* ]]
        }
    };
}

#[macro_export]
macro_rules! concat_trees {
    ($left:tt,()) => {
        $left
    };

    ((), $right:tt) => {
        $right
    };

    ($left:tt, $right:tt) => {{
        $crate::render_tree::Document::empty()
            .add($left)
            .add($right)
    }};
}

#[macro_export]
macro_rules! open_angle {
    {
        trace = $trace:tt
        rest = [[ $maybe_section:tt $($rest:tt)* ]]
    } => {
        open_angle! {
            trace = $trace
            double = [[ @double << $maybe_section $maybe_section >> $($rest)* ]]
        }
    };

    {
        trace = [ $($trace:tt)* ]
        double = [[ @double << $name:ident $double:ident >> $($rest:tt)* ]]
    } => {
        tagged_element! {
            trace = [ $($trace)* { tagged_element } ]
            name = $name
            args=[]
            rest=[[ $($rest)* ]]
        }
    };

    {
        trace = $trace:tt
        $kind:ident = [[ $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in open_angle", state="open_angle", trace=$trace, tokens=$($rest)*)
    }
}

#[macro_export]
macro_rules! tagged_element {
    {
        trace = $trace:tt
        name = $name:tt
        args = [ { $key:ident = $value:tt } $({ $keys:ident = $values:tt })* ]
        rest = [[ > $($rest:tt)*]]
    } => {{
        unexpected_token!("Only block-based components take keys and values as arguments. Pass arguments to inline components as `args={...}`", trace = $trace, tokens = $key);
    }};

    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        double = [[ @double << $maybe_block:tt { $(maybe_block2:tt)* } >> $($rest:tt)*  ]]
    } => {{
        unexpected_token!(
            concat!(
                "Pass a block to ",
                stringify!($name),
                " with the `as` keyword: `as` { ... } or pass args with args={ ... }"
            ),
            trace = $trace,
            tokens = $name
        );
    }};

    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        rest = [[ $maybe_block:tt $($rest:tt)* ]]
    } => {{
        tagged_element! {
            trace = $trace
            name = $name
            args = $args
            double = [[ @double << $maybe_block $maybe_block >> $($rest)*  ]]
        }
    }};

    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = $args:tt
        double = [[ @double << $as:tt as >> $($rest:tt)*  ]]
    } => {{
        block_component!(
            trace = [ $($trace)* { block_component } ]
            name = $name
            args = $args
            rest = [[ $($rest)* ]]
        )

    }};

    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = $args:tt
        double = [[ @double << args args >> = $($rest:tt)*  ]]
    } => {{
        component_with_args! {
            trace = [ $($trace)* { component_with_args } ]
            name = $name
            rest = [[ $($rest)* ]]
        }
    }};

    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = $args:tt
        double = [[ @double << $key:ident $key2:ident >> = $($rest:tt)*  ]]
    } => {{
        tagged_element_values! {
            trace = [ $($trace)* { tagged_element_values } ]
            name = $name
            args = $args
            key = $key
            rest = [[ $($rest)* ]]
        }
    }};

    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        double = [[ @double << $token:tt $double:tt >> $($rest:tt)* ]]
    } => {{
        unexpected_token!(concat!("Unexpected tokens after <", stringify!($name), ". Expected `key=value`, `as {` or `as |`"), trace = $trace, tokens = $token);
    }};

    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        rest = [[ ]]
    } => {{
        unexpected_eof!("In tagged_element", trace = $trace);
    }};

    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in tagged_element",trace=$trace, tokens=$($rest)*)
    }
}

#[macro_export]
macro_rules! tagged_element_values {
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $($args:tt)* ]
        key = $key:ident
        rest = [[ $value:tt $($rest:tt)* ]]
    } => {
        tagged_element! {
            trace = [ $($trace)* { tagged_element } ]
            name = $name
            args = [ $($args)* { $key = $value } ]
            rest = [[ $($rest)*]]
        }
    };
}

#[macro_export]
macro_rules! block_component {
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = []
        rest = [[ { $($block:tt)* }> $($rest:tt)* ]]
    } => {{
        let inner = tree! {
            trace = [ $($trace)* { inner tree } ]
            rest = [[ $($block)* ]]
        };

        let component = $name(inner);

        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(component, rest)
    }};

    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $({ $key:ident = $value:tt })* ]
        rest = [[ { $($block:tt)* }> $($rest:tt)* ]]
    } => {{
        use $crate::render_tree::prelude::*;

        let component = $name {
            $(
                $key: $value,
            )*
        };

        let block = $name(
            component, |doc: Document| -> Document { ( tree! {
            trace = [ $($trace)* { inner tree } ]
            rest = [[ $($block)* ]]
        }).render(doc) });


        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(block, rest)
    }};

    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $({ $key:ident = $value:tt })* ]
        rest = [[ |$id:tt| { $($block:tt)* }> $($rest:tt)* ]]
    } => {{
        use $crate::render_tree::prelude::*;

        let component = $name {
            $(
                $key: $value
            ),*
        };

        // TODO: propagate trace
        let block = $name(
            component, |$id, doc: Document| -> Document { (tree! { $($block)* }).render(doc) }
        );

        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(block, rest)
    }};

    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unimplemented_branch!("other tokens", trace = $trace, tokens=$($rest)*)
    };
}

#[macro_export]
macro_rules! component_with_args {
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        rest = [[ $value:tt $($rest:tt)* ]]
    } => {
        component_with_args_and_value! {
            trace = [ $($trace)* { component_with_args_and_value } ]
            name = $name
            value = $value
            rest = [[ $($rest)*]]
        }
    };

    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ as { $($rest:tt)* } > $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in component_with_args", trace = $trace, tokens = $($rest)*)
    };

    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in component_with_args", trace = $trace, tokens = $($rest)*)
    };

    {
        $($rest:tt)*
    } => {
        compile_error!(concat!("Unexpected call to component_with_args", stringify!($($rest)*)))
    }
}

#[macro_export]
macro_rules! component_with_args_and_value {
    // terminal
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        value = $value:tt
        rest = [[ > $($rest:tt)* ]]
    } => {{
        let left = $crate::render_tree::Component($name, $value);

        let right = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(left, right)
    }};

    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        value = $value:tt
        rest = [[ as $($rest:tt)* ]]
    } => {
        component_with_args_and_block! {
            trace = [ $($trace)* { component_with_args_and_block } ]
            name = $name
            value = $value
            rest = [[ $($rest)* ]]
        }
    };

    // terminal
    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ as { $($rest:tt)* } > $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in component_with_args_and_value", trace = $trace, tokens = $($rest)*)
    };

    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ @double << $pipe:tt | >> $param:ident| { $($block:tt)* } > $($rest:tt)* ]]
    } => {{
        unexpected_token!(
            concat!(
                "Block arguments (`|",
                stringify!($param),
                "|`) must come after the `as` keyword. Try `as |",
                stringify!($param),
                "|`"
            ),
            trace = $trace,
            tokens = $pipe
        )
    }};

    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ $maybe_pipe:tt $param:ident| { $($block:tt)* } > $($rest:tt)* ]]
    } => {{
        component_with_args_and_value! {
            trace = $trace
            name = $name
            value = $value
            rest = [[ @double << $maybe_pipe $maybe_pipe >> $param| { $($block)* } > $($rest)* ]]
        }
    }};

    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ as |$param:ident| { $($rest:tt)* } > $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in component_with_args_and_value", trace = $trace, tokens = $($rest)*)
    };

    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in component_with_args_and_value", trace = $trace, tokens = $($rest)*)
    };
}

#[macro_export]
macro_rules! component_with_args_and_block {
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        value = $args:tt
        rest = [[ |$id:ident| { $($inner:tt)* } > $($rest:tt)* ]]
    } => {{
        use $crate::render_tree::prelude::*;

        // TODO: propagate trace
        let block = $crate::render_tree::IterBlockComponent(
            $name::args($args), |$id, doc: Document| -> Document {
                (tree! {
                    trace = [ $($trace)* { inner tree } ]
                    rest = [[ $($inner)* ]]
                }).render(doc)
            }
        );

        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(block, rest)
    }};

    {
        trace = $trace:tt
        name = $name:tt
        value = $value:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unimplemented_branch!("in component_with_args_and_block", trace = $trace, tokens = $($rest)*)
    };
}
