use std::borrow::Cow;
use std::collections::{VecDeque, HashSet, BinaryHeap, HashMap, BTreeSet};
use std::rc::Rc;
use crate::moves::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Card {
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    V,
    D,
    K,
    T,
}

impl Card {
    pub fn goes_on(&self) -> Option<Self> {
        use Card::*;
        Some(match self {
            Six => Seven,
            Seven => Eight,
            Eight => Nine,
            Nine => Ten,
            Ten => V,
            V => D,
            D => K,
            K => T,
            T => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Board<'a> {
    pub columns: Vec<Column<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Column<'a> {
    Solved,
    Unsolved {
        cards: Cow<'a, [Card]>,
        cheat: Option<Card>,
    },
}

impl<'a> Board<'a> {
    pub fn is_solved(&self) -> bool {
        for column in self.columns.iter() {
            match column {
                Column::Solved => {},
                Column::Unsolved { cards, cheat: None } if &cards[..] == &[] => {}
                _ => { return false; }
            }
        }
        true
    }
    pub fn solve_naive(self) -> Option<(Self, Vec<Move>)> {
        use crate::moves::*;
        let mut seen: HashSet<Rc<Self>> = HashSet::new();
        let mut queue = VecDeque::with_capacity(1024);
        let all_moves = &Move::all_moves();
        queue.push_back((Rc::new(self), vec![]));
        let mut counter = 0;
        while let Some((board, moves)) = queue.pop_front() {
            if counter % 128 == 0 {
                print!("{} {:?}\x1b[0K\n\x1b[A", queue.len(), moves);
            }
            counter += 1;
            if seen.contains(&board) { continue; }
            seen.insert(Rc::clone(&board));
            for move_ in all_moves {
                if let Some(board) = move_.apply(&*board) {
                    let moves = moves.iter().copied().chain(Some(*move_)).collect();
                    if board.is_solved() { return Some((board, moves)); }
                    // breadth-first
                    queue.push_back((Rc::new(board), moves));
                    // depth-first
//                    queue.push_front((Rc::new(board), moves));
                }
            }
        }
        None
    }
    pub fn solve(self) -> Option<(Self, Vec<Move>)> {
        use crate::moves::*;

        #[derive(Debug, Clone, PartialEq, Eq)]
        struct QueueItem<'a> {
            board: Rc<Board<'a>>,
            moves: Vec<Move>,
        }

        impl<'a> std::cmp::PartialOrd for QueueItem<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<'a> std::cmp::Ord for QueueItem<'a> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.board.score().cmp(&other.board.score()).reverse() // max-heap
                    .then_with(|| self.board.cmp(&other.board))
                    .then_with(|| self.moves.cmp(&other.moves))
            }
        }

        impl<'a> From<(Rc<Board<'a>>, Vec<Move>)> for QueueItem<'a> {
            fn from((board, moves): (Rc<Board<'a>>, Vec<Move>)) -> Self {
                Self { board, moves }
            }
        }

        impl<'a> From<QueueItem<'a>> for (Rc<Board<'a>>, Vec<Move>) {
            fn from(QueueItem { board, moves }: QueueItem<'a>) -> Self {
                (board, moves)
            }
        }

        let mut seen: HashSet<Rc<Self>> = HashSet::new();
        let mut queue = BinaryHeap::<QueueItem>::with_capacity(1024);
        let all_moves = &Move::all_moves();
        queue.push((Rc::new(self), vec![]).into());
        while let Some((board, moves)) = queue.pop().map(Into::into) {
            if seen.contains(&board) { continue; }
            seen.insert(Rc::clone(&board));
            for move_ in all_moves {
                if let Some(board) = move_.apply(&*board) {
                    let moves = moves.iter().copied().chain(Some(*move_)).collect();
                    if board.is_solved() { return Some((board, moves)); }
                    queue.push((Rc::new(board), moves).into());
                }
            }
        }
        None
    }
    /// Lower is better
    pub fn score(&self) -> i64 {
        let mut score = 0;
        for column in self.columns.iter() {
            match column {
                Column::Solved => {
                    score -= 1024;
                },
                Column::Unsolved { cards, cheat: None } => {
                    score -= (cards.len() * cards.len()) as i64; // intentionally diff from Shenzhen IO
                },
                Column::Unsolved { cards, cheat: Some(_) } => {
                    score -= (cards.len() * cards.len()) as i64; // intentionally diff from Shenzhen IO
                    score += 256;
                },
            }
        }
        score
    }
    #[cfg(feature = "image")]
    pub fn from_image(image: image::GrayImage) -> Option<Self> {
        use Card::*;
        use image::*;
        lazy_static::lazy_static! {
            static ref CARDS: HashMap<Card, GrayImage> = [
                (Six, &include_bytes!("images/six.pbm")[..]),
                (Seven, &include_bytes!("images/seven.pbm")[..]),
                (Eight, &include_bytes!("images/eight.pbm")[..]),
                (Nine, &include_bytes!("images/nine.pbm")[..]),
                (Ten, &include_bytes!("images/ten.pbm")[..]),
                (V, &include_bytes!("images/v.pbm")[..]),
                (D, &include_bytes!("images/d.pbm")[..]),
                (K, &include_bytes!("images/k.pbm")[..]),
                (T, &include_bytes!("images/t.pbm")[..]),
            ].iter().map(
                |(card, slice)| (
                    *card,
                    io::Reader::with_format(
                        std::io::Cursor::new(&slice[..]),
                        ImageFormat::Pnm,
                    ).decode().unwrap().into_luma8()
                )
            ).collect();
        }

        let mut found: HashMap<(u32, u32), Card> = HashMap::with_capacity(36);

        fn images_same<G1, G2, P>(i1: &G1, i2: &G2) -> bool
            where
                G1: GenericImageView<Pixel = P>,
                G2: GenericImageView<Pixel = P>,
                P: PartialEq,
        {
            i1.dimensions() == i2.dimensions() &&
                i1.pixels().zip(i2.pixels()).all(
                    |(p1, p2)| p1 == p2
                )
        }

        for y in 0..image.height() {
            for x in 0..image.width() {
                for (card, card_image) in CARDS.iter() {
                    if image.width() < x + card_image.width() { continue; }
                    if image.height() < y + card_image.height() { continue; }
                    let sub_image = image.view(x, y, card_image.width(), card_image.height());
                    if images_same(&sub_image, &*card_image) {
                        found.insert((x, y), *card);
                    }
                }
            }
        }
        if found.len() != 36 { return None; }
        let x_values: BTreeSet<u32> = found.iter().map(|((x, _y), _card)| *x).collect();
        let y_values: BTreeSet<u32> = found.iter().map(|((_x, y), _card)| *y).take(6).collect(); // take(6) to ignore numbers on the bottom of cards, since the values are sorted top->bottom
        let mut columns: [Vec<Card>; 6] = [(); 6].map(|_| Vec::with_capacity(6));
        for y_value in y_values {
            for (i, x_value) in x_values.iter().copied().enumerate() {
                let card = found.get(&(x_value, y_value))?;
                columns[i].push(*card);
            }
        }
        Some(Board {
            columns: columns.into_iter().map(
                |cards| Column::Unsolved { cards: cards.into(), cheat: None }
            ).collect()
        })
    }
}
