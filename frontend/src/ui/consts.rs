use const_format::concatcp;

// google fonts icon names
pub const NAVIGATE_BACK: &str = "arrow_back_ios";
pub const CHECKBOX_UNCHECKED: &str = "check_box_outline_blank";
pub const CHECKBOX_CHECKED: &str = "check_box";
pub const ADD: &str = "add";
pub const SEARCH: &str = "search";
pub const DELETE: &str = "delete";

// Tailwindcss colours
pub const PRIMARY_BG: &str = "dark:bg-zinc-900 bg-white ";
pub const SECONDARY_BG: &str = "dark:bg-zinc-800 bg-gray-100 ";
pub const TERTIARY_BG: &str = "dark:bg-zinc-700 bg-gray-200 ";
pub const BORDER_SMALL: &str = "dark:border-zinc-500 border-gray-300 border p-1 rounded ";
pub const INPUT_PRIMARY: &str = concatcp!(SECONDARY_BG, BORDER_SMALL, "hover:border-2 ");