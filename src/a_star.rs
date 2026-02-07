#![deny(unused_variables)]
#![deny(unused_imports)]

use crate::board::{self};

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
};

const FINISHED: board::Board = board::Board {
    b: [
        [Some(1), Some(2), Some(3)],
        [Some(4), Some(5), Some(6)],
        [Some(7), Some(8), None],
    ],
};

#[derive(Clone, Eq, PartialEq)]
pub struct State {
    pub board: board::Board,
    pub path: Vec<board::Board>,
    pub g: i64, // g(n) = the cost so far
    pub f: i64, // f(n) = g(n) + h(n)
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        return other.f.cmp(&self.f).then_with(|| {
            other
                .board
                .distance(&FINISHED)
                .cmp(&self.board.distance(&FINISHED))
        });
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

pub fn search(starting_board: board::Board) -> Option<Vec<board::Board>> {
    // A* algorithm
    let mut queue: BinaryHeap<State> = BinaryHeap::new();

    let f = starting_board.distance(&FINISHED);
    let path: Vec<board::Board> = vec![starting_board];
    queue.push(State {
        board: starting_board,
        path,
        f,
        g: 0,
    });

    let mut visited: HashSet<board::Board> = HashSet::new();
    while !queue.is_empty() {
        let current_opt: Option<State> = queue.pop();
        current_opt.as_ref()?;

        let current = current_opt.unwrap();
        if visited.contains(&current.board) {
            continue;
        }
        if current.board.distance(&FINISHED) == 0 {
            return Some(current.path);
        }

        visited.insert(current.board);

        let next_boards: Vec<board::Board> = current.board.get_possible_next_states();
        for next_board in next_boards {
            let g = current.g + 1;
            let h = next_board.distance(&FINISHED);
            let f = g + h;

            let mut next_path = current.path.clone();
            next_path.push(next_board);
            let next_state = State {
                board: next_board,
                path: next_path,
                g,
                f,
            };
            queue.push(next_state);
        }
    }

    return None;
}
