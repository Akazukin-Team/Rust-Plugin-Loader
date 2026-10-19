pub trait Matcher {
    type Item;
    fn matches(&self, other: &Self::Item) -> bool;
}
