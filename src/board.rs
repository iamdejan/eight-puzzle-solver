#![deny(unused_variables)]
#![deny(unused_imports)]

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Board {
    pub b: [[Option<i64>; 3]; 3],
}

impl Board {
    fn find_number(&self, number: i64) -> Option<(usize, usize)> {
        for r in 0..3 {
            for c in 0..3 {
                if let Some(n) = self.b[r][c]
                    && n == number
                {
                    return Some((r, c));
                }
            }
        }

        // guaranteed to find the number
        return None;
    }
    fn find_empty_cell(&self) -> Option<(usize, usize)> {
        for r in 0..3 {
            for c in 0..3 {
                if self.b[r][c].is_none() {
                    return Some((r, c));
                }
            }
        }

        // guaranteed to find empty cell
        return None;
    }
    pub fn distance(&self, other_board: &Board) -> i64 {
        let mut total_distance: i64 = 0;
        for number in 1..=8 {
            let self_location = self.find_number(number);
            let other_board_location = other_board.find_number(number);

            // use Manhattan distance
            let distance: i64 = (self_location.unwrap().0 as i64
                - other_board_location.unwrap().0 as i64)
                .abs()
                + (self_location.unwrap().1 as i64 - other_board_location.unwrap().1 as i64).abs();
            total_distance += distance;
        }
        return total_distance;
    }
    pub fn copy_and_swap(&self, src_pos: (usize, usize), dest_pos: (usize, usize)) -> Board {
        let mut copied_b = self.b;

        let temp: Option<i64> = copied_b[src_pos.0][src_pos.1];
        copied_b[src_pos.0][src_pos.1] = copied_b[dest_pos.0][dest_pos.1];
        copied_b[dest_pos.0][dest_pos.1] = temp;

        return Board { b: copied_b };
    }
    pub fn get_possible_next_states(&self) -> Vec<Board> {
        let mut list: Vec<Board> = Vec::new();

        let empty_cell_location = self.find_empty_cell().unwrap();
        let neighbors: [(i8, i8); 4] = [(-1, 0), (0, -1), (0, 1), (1, 0)];
        for neighbor in neighbors {
            let new_r: i8 = empty_cell_location.0 as i8 + neighbor.0;
            let new_c: i8 = empty_cell_location.1 as i8 + neighbor.1;

            // if outside, skip
            if new_r < 0 || new_r >= self.b.len().try_into().unwrap() {
                continue;
            }
            if new_c < 0 || new_c >= self.b.len().try_into().unwrap() {
                continue;
            }

            // generate new board by swapping the content
            let new_location = (new_r as usize, new_c as usize);
            list.push(self.copy_and_swap(empty_cell_location, new_location));
        }

        return list;
    }
}
