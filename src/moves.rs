use crate::cards::*;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Move {
    Normal { from: usize, to: usize, count: NonZeroUsize },
    Cheat { from: usize, to: usize },
    UnCheat { from: usize, to: usize },
}

impl Move {
    pub fn all_moves() -> Vec<Move> {
        use Move::*;
        let position_pairs: Vec<(usize, usize)> =
            (0..6)
            .flat_map(
                |f| (0..6).filter_map(move |t| if f != t { Some((f, t)) } else { None })
            ).collect();
        position_pairs.iter().copied()
            .flat_map(
                |(from, to)| (1..=9).map(
                    move |count| Normal { from, to, count: NonZeroUsize::new(count).unwrap() }
                )
            ).chain(
                position_pairs.iter().copied()
                    .map(|(from, to)| Cheat { from, to })
            ).chain(
                position_pairs.iter().copied()
                    .map(|(from, to)| UnCheat { from, to })
            ).collect()
    }
    pub fn apply(&self, board: &Board) -> Option<Board> {
        use Move::*;
        fn get_mut_two<'a, T>(slice: &'a mut [T], i1: usize, i2: usize) -> Option<(&'a mut T, &'a mut T)> {
            if i1 == i2 { return None; }
            let a = slice as *mut [T];
            unsafe { Some((
                (*a).get_mut(i1)?,
                (*a).get_mut(i2)?,
            )) }
        }
        let mut board = board.clone();
        let mut board = match *self {
            Normal { from, to, count } => {
//                let from = board.columns.get_mut(from)?;
//                let to = board.columns.get_mut(to)?;
                {
                    let (from, to) = get_mut_two(&mut board.columns, from, to)?;
                    let from = match from {
                        Column::Unsolved { ref mut cards, cheat: None } => cards,
                        _ => return None,
                    };
                    let to = match to {
                        Column::Unsolved { ref mut cards, cheat: None } => cards,
                        _ => return None,
                    };
                    let range = from.len().checked_sub(count.into())?..;
                    let to_be_moved = from.drain(range);
                    for pair in to_be_moved.as_slice().windows(2) {
                        if pair[1].goes_on() != Some(pair[0]) { return None; }
                    }
                    let goes_on = to.last().copied();
                    // Any card can be placed on empty column
                    if goes_on != None && goes_on != to_be_moved.as_slice()[0].goes_on() { return None; }
                    to.extend_from_slice(to_be_moved.as_slice());
                }
                Some(board)
            },
            Cheat { from, to } => {
                {
                    let (from, to) = get_mut_two(&mut board.columns, from, to)?;
                    let from = match from {
                        Column::Unsolved { ref mut cards, cheat: None } => cards,
                        _ => return None,
                    };
                    let (to, to_cheat) = match to {
                        Column::Unsolved { ref mut cards, cheat: cheat@None } => (cards, cheat),
                        _ => return None,
                    };
                    let goes_on = to.last().copied();
                    let card = from.pop()?;
                    // Any card can be placed on empty column (so it wouldn't be cheating
                    if goes_on == None || goes_on == card.goes_on() { return None; }
                    *to_cheat = Some(card);
                }
                Some(board)
            },
            UnCheat { from, to } => {
                {
                    let (from, to) = get_mut_two(&mut board.columns, from, to)?;
                    let from_cheat = match from {
                        Column::Unsolved { cheat: cheat@Some(_), .. } => cheat,
                        _ => return None,
                    };
                    let to = match to {
                        Column::Unsolved { ref mut cards, cheat: None } => cards,
                        _ => return None,
                    };
                    let goes_on = to.last().copied();
                    let card = from_cheat.take()?;
                    // Any card can be placed on empty column
                    if goes_on != None && goes_on != card.goes_on() { return None; }
                    to.push(card);
                }
                Some(board)
            },
        }?;
        // Check if any column is solved
        for column in board.columns.iter_mut() {
            if let Column::Unsolved { cards, cheat: None } = column {
                use Card::*;
                if &cards[..] == &[T, K, D, V, Ten, Nine, Eight, Seven, Six] {
                    *column = Column::Solved;
                }
            }
        }
        Some(board)
    }
}
