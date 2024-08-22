macro_rules! make_tags {
    ($($(#[$($attrss:tt)*])*($tag:literal, $id:ident, $ifd:expr $(,xmp = $xmp_ns:ident)?)),*$(,)?) => {
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
                impl $crate::xmp::Field for $id {
                    const NAME: &'static str = stringify!($id);
                    const NAMESPACE: $crate::xmp::Namespace = $crate::xmp::Namespace::$xmp_ns;
                }
            )*
        )*

        pub(crate) static TAG_NAMES: std::sync::LazyLock<std::collections::HashMap<(u16, Ifd), &'static str>> =
         std::sync::LazyLock::new(|| std::collections::HashMap::from([
            $(
                (($tag, $ifd), stringify!($id)),
            )*
        ]));
    };
}

macro_rules! make_exif_tags {
    ($($(#[$($attrss:tt)*])*($tag:literal, $id:ident, $ifd:expr)),*$(,)?) => {
        $(
            $(#[$($attrss)*])*
            #[derive(Copy, Clone, Debug)]
            pub struct $id;

            impl $crate::exif::Field for $id {
                const NAME: &'static str = stringify!($id);
                const TAG: crate::exif::Tag = crate::exif::Tag($tag);
                const IFD: Ifd = $ifd;
            }
        )*
    };
}

macro_rules! make_xmp_tags {
    ($($(#[$($attrss:tt)*])*($id:ident, $namespace:ident)),*$(,)?) => {
        $(
            $(#[$($attrss)*])*
            #[derive(Copy, Clone, Debug)]
            pub struct $id;


            impl $crate::xmp::Field for $id {
                const NAME: &'static str = stringify!($id);
                const NAMESPACE: $crate::xmp::Namespace = $crate::xmp::Namespace::$namespace;
            }
        )*
    };

    ($($(#[$($attrss:tt)*])*($id:ident, $name:literal, $namespace:ident)),*$(,)?) => {
        $(
            $(#[$($attrss)*])*
            #[derive(Copy, Clone, Debug)]
            pub struct $id;


            impl $crate::xmp::Field for $id {
                const NAME: &'static str = $name;
                const NAMESPACE: $crate::xmp::Namespace = $crate::xmp::Namespace::$namespace;
            }
        )*
    };
}

pub(crate) use {make_exif_tags, make_tags, make_xmp_tags};
