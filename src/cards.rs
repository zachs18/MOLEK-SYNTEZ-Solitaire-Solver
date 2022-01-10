use std::collections::{VecDeque, HashSet, BinaryHeap, HashMap, BTreeSet};
use std::rc::Rc;
use std::num::NonZeroUsize;
use crate::moves::*;
#[cfg(feature = "thread")]
use std::sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, Ordering}};

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
    pub fn to_str(&self) -> &'static str {
        use Card::*;
        match self {
            Six => "6",
            Seven => "7",
            Eight => "8",
            Nine => "9",
            Ten => "X",
            V => "V",
            D => "D",
            K => "K",
            T => "T",
        }
    }
    pub fn to_str_cheat(&self) -> &'static str {
        use Card::*;
        match self {
            Six => "\x1b[30;107m6\x1b[0m",
            Seven => "\x1b[30;107m7\x1b[0m",
            Eight => "\x1b[30;107m8\x1b[0m",
            Nine => "\x1b[30;107m9\x1b[0m",
            Ten => "\x1b[30;107mX\x1b[0m",
            V => "\x1b[30;107mV\x1b[0m",
            D => "\x1b[30;107mD\x1b[0m",
            K => "\x1b[30;107mK\x1b[0m",
            T => "\x1b[30;107mT\x1b[0m",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Board {
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Column {
    Solved,
    Unsolved {
        cards: Vec<Card>,
        cheat: Option<Card>,
    },
}

impl Board {
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
        while let Some((board, moves)) = queue.pop_front() {
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

        #[cfg(feature = "thread")]
        type Rc<T> = Arc<T>;

        #[derive(Debug, Clone, PartialEq, Eq)]
        struct QueueItem {
            board: Rc<Board>,
            moves: Vec<Move>,
        }

        impl std::cmp::PartialOrd for QueueItem {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl std::cmp::Ord for QueueItem {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.board.score().cmp(&other.board.score()).reverse() // max-heap
                    .then_with(|| self.moves.len().cmp(&other.moves.len()).reverse())
                    .then_with(|| self.board.cmp(&other.board))
                    .then_with(|| self.moves.cmp(&other.moves))
            }
        }

        impl From<(Rc<Board>, Vec<Move>)> for QueueItem {
            fn from((board, moves): (Rc<Board>, Vec<Move>)) -> Self {
                Self { board, moves }
            }
        }

        impl From<QueueItem> for (Rc<Board>, Vec<Move>) {
            fn from(QueueItem { board, moves }: QueueItem) -> Self {
                (board, moves)
            }
        }
        #[cfg(not(feature = "thread"))]
        {
            let mut seen: HashSet<Rc<Self>> = HashSet::new();
            let mut queue = BinaryHeap::<QueueItem>::with_capacity(1024);
            let all_moves = &Move::all_moves();
            queue.push((Rc::new(self), vec![]).into());
//            let mut counter = 0;
            while let Some((board, moves)) = queue.pop().map(Into::into) {
                if seen.contains(&board) { continue; }
                seen.insert(Rc::clone(&board));
//                if counter % 256 == 0 {
//                    println!("\x1b[H\x1b[2J\x1b[3J{} ({}): \n{}\n{:?}", queue.len(), board.score(), board.to_string(), moves);
//                }
//                counter += 1;
                for move_ in all_moves { //board.possible_moves() {
                    if let Some(board) = move_.apply(&*board) {
                        let moves = moves.iter().copied().chain(Some(*move_)).collect();
                        if board.is_solved() { return Some((board, moves)); }
                        queue.push((Rc::new(board), moves).into());
                    }
                }
            }
        }
        #[cfg(feature = "thread")]
        {
            let seen: Arc<RwLock<HashSet<Rc<Self>>>> = Arc::new(RwLock::new(HashSet::new()));
            let queue: Arc<Mutex<BinaryHeap<QueueItem>>> = Arc::new(Mutex::new({
                let mut queue = BinaryHeap::<QueueItem>::with_capacity(1024);
                queue.push((Rc::new(self), vec![]).into());
                queue
            }));
            let result: Arc<Mutex<Option<(Self, Vec<Move>)>>> = Arc::new(Mutex::new(None));
            let finished: Arc<AtomicBool> = Arc::new(false.into());
            let all_moves = Arc::new(Move::all_moves());
            let make_worker = |thread| {
                let seen = Arc::clone(&seen);
                let queue = Arc::clone(&queue);
                let result = Arc::clone(&result);
                let finished = Arc::clone(&finished);
                let all_moves = Arc::clone(&all_moves);
                move || {
//                    let mut counter = 0;
                    while !finished.load(Ordering::Relaxed) {
                        let top = { queue.lock().unwrap().pop().map(Into::into) };
                        if let Some((board, moves)) = top {
                            if seen.read().unwrap().contains(&board) { continue; }
                            seen.write().unwrap().insert(Rc::clone(&board));
//                            if counter % 256 == 0 {
//                                let queue = queue.lock().unwrap();
//                                println!("\x1b[H\x1b[2J\x1b[3J{} ({}): \n{}\n{:?}", queue.len(), board.score(), board.to_string(), moves);
//                            }
//                            counter += 1;
                            for move_ in &**all_moves {
                                if let Some(board) = move_.apply(&*board) {
                                    let moves = moves.iter().copied().chain(Some(*move_)).collect();
                                    if board.is_solved() {
                                        finished.store(true, Ordering::Relaxed);
                                        *result.lock().unwrap() = Some((board, moves));
                                        return;
                                    }
                                    queue.lock().unwrap().push((Rc::new(board), moves).into());
                                }
                            }
                        } else {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                }
            };
            let workers: Vec<_> = (0..num_cpus::get()).map(|thread| std::thread::spawn(make_worker(thread))).collect();
            workers.into_iter().for_each(|t| t.join().unwrap());
            return Arc::try_unwrap(result).unwrap().into_inner().unwrap();
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
                    let mut depth = 1;
                    for w in cards.windows(2).rev() {
                        if let [c1, c2] = w {
                            if c2.goes_on() == Some(*c1) { depth += 1; }
                        }
                    }
                    score -= depth * depth;
                },
                Column::Unsolved { cards, cheat: Some(_) } => {
                    // Intentionally different from no cheat, since having a cheat on top of a lot of cards is bad(?)
                    score += (cards.len() * cards.len()) as i64;
                    score += 256;
                },
            }
        }
        score
    }
    pub fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(100);
        for (from, from_col) in self.columns.iter().enumerate() {
            match from_col {
                Column::Solved => {},
                Column::Unsolved { cheat: Some(cheat), .. } => {
                    for (to, to_col) in self.columns.iter().enumerate() {
                        match to_col {
                            Column::Solved => {},
                            Column::Unsolved { cheat: Some(_), .. } => {},
                            Column::Unsolved { cards: to_cards, cheat: None } => {
                                if &to_cards[..] == &[] || to_cards.last() == Some(cheat) {
                                    moves.push(Move::UnCheat { from, to });
                                }
                            },
                        };
                    }
                },
                Column::Unsolved { cards: from_cards, cheat: None } => {
                    for (to, to_col) in self.columns.iter().enumerate() {
                        match to_col {
                            Column::Solved => {},
                            Column::Unsolved { cheat: Some(_), .. } => {},
                            Column::Unsolved { cards: to_cards, cheat: None } => {
                                // TODO
                                moves.push(Move::Cheat { from, to });
                                moves.extend((1..=9).map(|count: usize| Move::Normal { from, to, count: NonZeroUsize::try_from(count).unwrap() }));
                            },
                        };
                    }
                },
            };
        }
        moves
    }
    pub fn to_string(&self) -> String {
        let mut columns: Vec<Box<dyn Iterator<Item=&'static str>>> = self.columns.iter().map(
            |column| -> Box<dyn Iterator<Item=&'static str>> { match column {
                Column::Solved => Box::new(std::iter::once("S")),
                Column::Unsolved { cards, cheat } => Box::new(
                    cards.iter().map(Card::to_str).chain(cheat.iter().map(Card::to_str_cheat))
                ),
            }}
        ).collect();
        let mut result = String::with_capacity(256);
        loop {
            let row: Vec<Option<&'static str>> = columns.iter_mut().map(Iterator::next).collect();
            if row.iter().all(Option::is_none) { break; }
            for card in row {
                match card {
                    Some(card) => { result += card; result += " "; },
                    None => { result += "  "; },
                }
            }
            result += "\n";
        }
        result
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

        let mut found: HashMap<(u32, u32), Card> = HashMap::with_capacity(36+6); // +6 for possible bottom numbers

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
//        for ((x, y), card) in &found {
//            println!("({},{}): {:?}", x, y, card);
//        }
        let y_values: BTreeSet<u32> = found.iter().map(|((_x, y), _card)| *y).collect();
        let y_values: BTreeSet<u32> = y_values.into_iter().take(6).collect(); // take(6) to ignore numbers on the bottom of cards, since the values are sorted top->bottom
        let x_values: BTreeSet<u32> = found.iter().filter_map(
            // Ensure that the bottom number on the top card is not counted
            |((x, y), _card)| if y_values.contains(&y) { Some(*x) } else { None }
        ).collect();
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
