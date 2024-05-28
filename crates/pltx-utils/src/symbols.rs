pub const CHECK: &str = "✔";
pub const CROSS: &str = "✘";

pub mod border {
    pub const VERTICAL: &str = "│";
    pub const HORIZONTAL: &str = "─";

    pub const TOP_LEFT: &str = "┌";
    pub const TOP_RIGHT: &str = "┐";
    pub const BOTTOM_RIGHT: &str = "┘";
    pub const BOTTOM_LEFT: &str = "└";

    pub const TOP_LEFT_ROUNDED: &str = "╭";
    pub const TOP_RIGHT_ROUNDED: &str = "╮";
    pub const BOTTOM_RIGHT_ROUNDED: &str = "╯";
    pub const BOTTOM_LEFT_ROUNDED: &str = "╰";

    pub const CENTER_T: &str = "┼";
    pub const TOP_T: &str = "┬";
    pub const RIGHT_T: &str = "┤";
    pub const BOTTOM_T: &str = "┴";
    pub const LEFT_T: &str = "├";
}

pub mod bold {
    pub mod border {
        pub const VERTICAL: &str = "┃";
        pub const HORIZONTAL: &str = "━";

        pub const TOP_LEFT: &str = "┏";
        pub const TOP_RIGHT: &str = "┓";
        pub const BOTTOM_RIGHT: &str = "┛";
        pub const BOTTOM_LEFT: &str = "┗";
    }
}
