#![deny(unused_variables)]
#![deny(unused_imports)]

mod a_star;
mod board;

fn main() {
    let starting_board = board::Board {
        b: [
            [Some(5), Some(1), Some(2)],
            [Some(3), Some(7), Some(8)],
            [Some(4), Some(6), None],
        ],
    };

    a_star::search(starting_board);
}
