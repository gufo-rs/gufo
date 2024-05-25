macro_rules! old {
    ($id:ident) => {
        impl $crate::xmp::Field for $id {
            const NAME: &'static str = stringify!($id);
        }
    };
}

macro_rules! new {
    ($id:ident) => {
        impl $crate::xmp::Field for $id {
            const NAME: &'static str = stringify!($id);
            const EX: bool = true;
        }
    };
}

macro_rules! make_tags {
    ($($(#[$($attrss:tt)*])*($tag:literal, $id:ident, $ifd:expr $(,xmp = $xmp:ident)?)),*$(,)?) => {
        $(
            $(#[$($attrss)*])*
            #[derive(Copy, Clone, Debug)]
            pub struct $id;

            impl $crate::exif::Field for $id {
                const NAME: &'static str = stringify!($id);
                const TAG: crate::exif::Tag = crate::exif::Tag($tag);
                const IFD: Ifd = $ifd;
            }

            $(
                $xmp!($id);
            )*
        )*

        pub(crate) static TAG_NAMES: once_cell::sync::Lazy<std::collections::HashMap<(u16, Ifd), &'static str>> =
         once_cell::sync::Lazy::new(|| std::collections::HashMap::from([
            $(
                (($tag, $ifd), stringify!($id)),
            )*
        ]));
    };
}

pub(crate) use {make_tags, new, old};
