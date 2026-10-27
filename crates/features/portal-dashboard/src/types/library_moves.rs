#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryMoves {
    pub defaults: Vec<String>,
    pub moves: Vec<(usize, String)>,
}
