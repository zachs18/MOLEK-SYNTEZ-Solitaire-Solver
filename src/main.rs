pub mod cards;
pub mod moves;

fn main() {
    use cards::*;
    use Card::*;
//    let board = Board { columns: vec![
//        Column::Unsolved {
//            cards: (&[Nine, Six, Nine, V, T, V][..]).into(),
//            cheat: None,
//        },
//        Column::Unsolved {
//            cards: (&[Six, T, V, Ten, Eight, Seven][..]).into(),
//            cheat: None,
//        },
//        Column::Unsolved {
//            cards: (&[K, D, T, Seven, Six, Seven][..]).into(),
//            cheat: None,
//        },
//        Column::Unsolved {
//            cards: (&[T, Nine, Eight, V, K, Ten][..]).into(),
//            cheat: None,
//        },
//        Column::Unsolved {
//            cards: (&[D, Eight, Ten, Six, Nine, K][..]).into(),
//            cheat: None,
//        },
//        Column::Unsolved {
//            cards: (&[Seven, Eight, D, D, K, Ten][..]).into(),
//            cheat: None,
//        },
//    ] };
    let image_name = std::env::args().nth(1);
    let board = match image_name {
        Some(image_name) => Board::from_image(
            image::open(image_name).unwrap().into_luma8()
        ).unwrap(),
        None => Board { columns: vec![
            Column::Unsolved {
                cards: (&[Eight, Seven, K, V, K, Six][..]).into(),
                cheat: None,
            },
            Column::Unsolved {
                cards: (&[Six, Ten, V, Ten, Seven, Eight][..]).into(),
                cheat: None,
            },
            Column::Unsolved {
                cards: (&[Six, V, Nine, Nine, K, D][..]).into(),
                cheat: None,
            },
            Column::Unsolved {
                cards: (&[Six, Nine, Ten, V, D, T][..]).into(),
                cheat: None,
            },
            Column::Unsolved {
                cards: (&[D, Eight, K, T, T, D][..]).into(),
                cheat: None,
            },
            Column::Unsolved {
                cards: (&[Seven, Nine, Ten, T, Seven, Eight][..]).into(),
                cheat: None,
            },
        ] }
    };
    match board.solve() {
        Some((_board, moves)) => {
            println!("");
            println!("");
            println!("Solved: [");
            let moves = IntoIterator::into_iter(moves);
            for r#move in moves {
                println!("\t{:?}", r#move);
            }
            println!("]");
        },
        None => {
            println!("");
            println!("");
            println!("Could not solve");
        }
    };
}
