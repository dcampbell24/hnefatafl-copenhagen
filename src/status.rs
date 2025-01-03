#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Status {
    BlackWins,
    Draw,
    WhiteWins,
    #[default]
    Ongoing,
}
