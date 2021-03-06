mod fuzzy;
mod re_based;

#[cfg(all(test, not(windows)))]
mod tests;
#[cfg(all(test, windows))]
mod tests_windows;


use std::iter;
use std::path;

use regex;


pub struct Matcher<'a> {
    fuzzy_matcher: fuzzy::FuzzyMatcher<'a>,
    re_anywhere: regex::Regex,
    re_consecutive: regex::Regex,
}


/// Returns whether matches should ignore case based on uppercase letter's
/// presence in the needles.
fn detect_smartcase(needles: &[&str]) -> bool {
    for s in needles {
        for ch in s.chars() {
            if ch.is_uppercase() {
                return false;
            }
        }
    }

    true
}


// Iterator sadness...
macro_rules! filter_path_with_re {
    ($l: expr, $re: expr) => {
        $l
            .iter()
            .filter(move |&p| $re.is_match(p.as_ref().to_string_lossy().to_mut()))
    };
}


impl<'a> Matcher<'a> {
    pub fn new_smartcase(needles: Vec<&'a str>) -> Matcher<'a> {
        let ignore_case = detect_smartcase(&needles);
        Matcher::new(needles, ignore_case)
    }

    pub fn new(needles: Vec<&'a str>, ignore_case: bool) -> Matcher<'a> {
        let fuzzy_matcher = fuzzy::FuzzyMatcher::defaults(needles[needles.len() - 1]);
        let re_anywhere =
            re_based::prepare_regex(&needles, re_based::re_match_anywhere, ignore_case);
        let re_consecutive =
            re_based::prepare_regex(&needles, re_based::re_match_consecutive, ignore_case);

        Matcher {
            fuzzy_matcher: fuzzy_matcher,
            re_anywhere: re_anywhere,
            re_consecutive: re_consecutive,
        }
    }


    #[cfg(feature = "nightly")]
    pub fn execute<'p, P>(&'a self, haystack: &'p [P]) -> impl iter::Iterator<Item = &'p P> + 'a
    where
        P: AsRef<path::Path>,
        'p: 'a,
    {
        filter_path_with_re!(haystack, self.re_consecutive)
            .chain(self.fuzzy_matcher.filter_path(haystack))
            .chain(filter_path_with_re!(haystack, self.re_anywhere))
    }


    #[cfg(not(feature = "nightly"))]
    pub fn execute<'p, P>(&'a self, haystack: &'p [P]) -> Box<iter::Iterator<Item = &'p P> + 'a>
    where
        P: AsRef<path::Path>,
        'p: 'a,
    {
        Box::new(
            filter_path_with_re!(haystack, self.re_consecutive)
                .chain(self.fuzzy_matcher.filter_path(haystack))
                .chain(filter_path_with_re!(haystack, self.re_anywhere)),
        )
    }
}
