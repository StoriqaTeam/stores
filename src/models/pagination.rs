#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Reverse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ordering {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug)]
pub struct PaginationParams<Cursor: Ord> {
    pub direction: Direction,
    pub limit: i64,
    pub ordering: Ordering,
    pub skip: i64,
    pub start: Option<Cursor>,
}
