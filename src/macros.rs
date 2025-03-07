#[macro_export]
macro_rules! first_stage {
    ($($item:item)*) => {
        $(
            #[link_section = ".first_stage"]
            #[cfg(feature = "16bit")]
            $item
        )*
    };
}

#[macro_export]
macro_rules! second_stage {
    ($($item:item)*) => {
        $(
            #[link_section = ".second_stage"]
            #[cfg(feature = "32bit")]
            $item
        )*
    };
}

