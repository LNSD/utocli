//! Builder pattern macros for OpenCLI types.
//!
//! This module provides macros to generate builder types that work alongside
//! the direct builder methods on OpenCLI structs.

/// Constructs a build function for a builder type.
///
/// Generates a `build()` method that consumes the builder and returns
/// the final struct with all fields transferred.
macro_rules! build_fn {
    ( $vis:vis $name:ident $( $field:ident ),+ ) => {
        #[doc = concat!("Constructs a new [`", stringify!($name),"`] taking all fields values from this object.")]
        $vis fn build(self) -> $name {
            $name {
                $(
                    $field: self.$field,
                )*
            }
        }
    };
}

pub(crate) use build_fn;

/// Sets a field value in a builder using method chaining.
///
/// Returns `self` to enable fluent API.
macro_rules! set_value {
    ( $self:ident $field:ident $value:expr ) => {{
        $self.$field = $value;
        $self
    }};
}

pub(crate) use set_value;

/// Generates `From` implementations between builder and target type.
///
/// Creates bidirectional conversions:
/// - `From<Builder> for Type` - calls `build()`
/// - `From<Type> for Builder` - transfers fields
macro_rules! from {
    ( $builder_name:ident $type_name:ident $( $field:ident ),+ ) => {
        impl From<$builder_name> for $type_name {
            fn from(value: $builder_name) -> Self {
                value.build()
            }
        }

        impl From<$type_name> for $builder_name {
            fn from(value: $type_name) -> Self {
                Self {
                    $( $field: value.$field, )*
                }
            }
        }
    };
}

pub(crate) use from;

/// Main builder macro that generates both struct and builder types.
///
/// This macro creates:
/// 1. The main struct with all its attributes and fields
/// 2. A corresponding builder struct with the same fields
/// 3. A `builder()` method on the main struct
/// 4. A `build()` method on the builder struct
/// 5. Default implementation for the builder
///
/// # Example
///
/// ```ignore
/// builder! {
///     InfoBuilder;
///
///     #[derive(Debug, Clone)]
///     pub struct Info {
///         pub title: String,
///         pub version: String,
///     }
/// }
/// ```
macro_rules! builder {
    ( $( #[$builder_meta:meta] )* $builder_name:ident; $(#[$meta:meta])* $vis:vis $key:ident $name:ident $( $tt:tt )* ) => {
        builder!( @type_impl $builder_name $( #[$meta] )* $vis $key $name $( $tt )* );
        builder!( @builder_impl $( #[$builder_meta] )* $builder_name $( #[$meta] )* $vis $key $name $( $tt )* );
    };

    ( @type_impl $builder_name:ident $( #[$meta:meta] )* $vis:vis $key:ident $name:ident
        { $( $( #[$field_meta:meta] )* $field_vis:vis $field:ident: $field_ty:ty, )* }
    ) => {
        $( #[$meta] )*
        $vis $key $name {
            $( $( #[$field_meta] )* $field_vis $field: $field_ty, )*
        }

        impl $name {
            #[doc = concat!("Construct a new ", stringify!($builder_name), ".")]
            #[doc = ""]
            #[doc = concat!("This is effectively same as calling [`", stringify!($builder_name), "::new`]")]
            $vis fn builder() -> $builder_name {
                $builder_name::new()
            }
        }
    };

    ( @builder_impl $( #[$builder_meta:meta] )* $builder_name:ident $( #[$meta:meta] )* $vis:vis $key:ident $name:ident
        { $( $( #[$field_meta:meta] )* $field_vis:vis $field:ident: $field_ty:ty, )* }
    ) => {
        $( #[$builder_meta] )*
        $vis $key $builder_name {
            $( $field_vis $field: $field_ty, )*
        }

        impl $builder_name {
            build_fn!($vis $name $( $field ),*);
        }

        from!($builder_name $name $( $field ),*);
    };
}

pub(crate) use builder;
