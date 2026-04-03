pub mod grouper;
pub mod matcher;
pub mod types;

pub use grouper::{group_search_results, parse_rg_json_match, ParsedRgMatch};
pub use matcher::{
    do_match, do_match_positions, exact_match, exact_match_positions, fuzzy_match,
    fuzzy_match_positions, regex_match, regex_match_positions,
};
pub use types::{
    ContentMatch, FileGroup, GlobalSearchResult, GlobalSearchType, GroupedItem, MatchMode,
    SearchMode, SearchState,
};
