use std::fmt::Display;
use std::iter::{FlatMap, Map};
use std::ops::Range;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct Coordinate {
    x: usize,
    y: usize,
}

type Player = u32;
type Cell = Option<Token>;

#[derive(Debug, Clone)]
struct Token {
    player: Player,
    locked: bool,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let player = match self.player {
            0 => 'x',
            1 => 'o',
            _ => 'h',
        };

        if self.locked {
            write!(f, "{}", player.to_uppercase())?;
        } else {
            write!(f, "{}", player)?;
        }

        Ok(())
    }
}

fn c(x: usize, y: usize) -> Coordinate {
    Coordinate { x, y }
}

#[derive(Debug, Eq, PartialEq)]
enum WinState {
    NotOver,
    Draw,
    Winner(Player),
}

/// Current state of the game board,
///
/// plus a method to advance the state by playing a move
#[derive(Debug, Clone)]
struct Board {
    current_turn: Player,
    cells: Vec<Cell>,
    size: (usize, usize),
}

impl Board {
    pub fn new(size: (usize, usize)) -> Board {
        Board {
            current_turn: 0,
            cells: vec![None; size.0 * size.1],
            size,
        }
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        let unlocked_populated_cells: Vec<Coordinate> = self
            .get_cells()
            .filter_map(|(cell, coordinate)| match cell {
                None => None,
                Some(Token { locked: false, .. }) => Some(coordinate),
                Some(Token { locked: true, .. }) => None,
            })
            .collect();

        self.get_cells()
            .flat_map(|(cell, coordinate)| match cell {
                None => vec![Move::Place(coordinate)],
                Some(Token { locked: false, .. }) => unlocked_populated_cells
                    .iter()
                    .filter(|c| **c != coordinate)
                    .map(|c| Move::Swap(coordinate, *c))
                    .collect(),
                Some(Token { locked: true, .. }) => vec![],
            })
            .collect()
    }

    pub fn advance(&self, move_: Move) -> Result<Board, ()> {
        let mut new_state = match move_ {
            Move::Place(coordinate) => {
                if self.get_cell(coordinate).is_some() {
                    return Err(());
                }

                Ok(self.set_cell(
                    coordinate,
                    Token {
                        locked: false,
                        player: self.current_turn,
                    },
                ))
            }
            Move::Swap(pos1, pos2) => {
                if self.get_cell(pos1).is_none() {
                    return Err(());
                }
                if self.get_cell(pos2).is_none() {
                    return Err(());
                }

                let mut new_state = self.clone();
                let index1 = self.cell_index(pos1);
                let index2 = self.cell_index(pos2);
                new_state.cells.swap(index1, index2);
                new_state.cells[index1].as_mut().unwrap().locked = true;
                new_state.cells[index2].as_mut().unwrap().locked = true;

                Ok(new_state)
            }
        }?;

        new_state.update_locked_cells();
        new_state.current_turn = (new_state.current_turn + 1) % 2;
        Ok(new_state)
    }

    fn update_locked_cells(&mut self) {
        let cells_to_lock: Vec<Coordinate> = self
            .cells_and_neighbors()
            .filter_map(|((cell, coordinate), neighbors)| {
                if let Some(Token {
                    player,
                    locked: false,
                }) = cell
                {
                    if Self::cell_is_victory_point(neighbors, player) {
                        Some(coordinate)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for coordinate in cells_to_lock {
            let cell_index = self.cell_index(coordinate);
            let token = self.cells[cell_index].as_mut().unwrap();
            token.locked = true;
        }
    }

    fn cell_is_victory_point(neighbors: Vec<(Cell, Coordinate)>, player: Player) -> bool {
        neighbors
            .iter()
            .filter_map(|(neighbor, _)| neighbor.as_ref())
            .map(|token| if token.player == player { 1 } else { 0 })
            .sum::<usize>()
            > 3
    }

    fn cell_index(&self, coordinate: Coordinate) -> usize {
        self.size.0 * coordinate.y + coordinate.x
    }

    pub fn check_win_condition(&self) -> WinState {
        if !self.get_legal_moves().is_empty() {
            return WinState::NotOver;
        }

        let victory_points_per_player = self.count_victory_points();
        if victory_points_per_player[0] == victory_points_per_player[1] {
            return WinState::Draw;
        }

        if victory_points_per_player[0] > victory_points_per_player[1] {
            WinState::Winner(0)
        } else {
            WinState::Winner(1)
        }
    }

    pub fn count_victory_points(&self) -> Vec<usize> {
        let mut points_per_player = Vec::new();

        for player in 0..2 {
            let player_points = self
                .cells_and_neighbors()
                .filter_map(|((cell, _), neighbors)| cell.map(|t| (t, neighbors)))
                .filter(|(token, _)| token.player == player)
                .filter_map(|(_, neighbors)| {
                    Self::cell_is_victory_point(neighbors, player).then_some(1)
                })
                .sum();

            points_per_player.push(player_points);
        }

        points_per_player
    }

    pub fn cells_and_neighbors(
        &self,
    ) -> impl Iterator<Item = ((Cell, Coordinate), Vec<(Cell, Coordinate)>)> + '_ {
        self.get_cells()
            .map(|(cell, coordinate)| {
                (
                    (cell, coordinate),
                    self.cells_neighbor_coordinates(coordinate),
                )
            })
            .map(|(cell_and_coord, neighbor_coordinates)| {
                (
                    cell_and_coord,
                    neighbor_coordinates
                        .into_iter()
                        .map(|neighbor_coordinate| {
                            (self.get_cell(neighbor_coordinate), neighbor_coordinate)
                        })
                        .collect(),
                )
            })
    }

    fn get_cells(&self) -> impl Iterator<Item = (Cell, Coordinate)> + '_ {
        (0..self.size.1)
            .flat_map(move |y| (0..self.size.0).map(move |x| (x, y)))
            .map(|(x, y)| c(x, y))
            .map(|c| (self.get_cell(c), c))
    }

    fn set_cell(&self, coordinate: Coordinate, token: Token) -> Board {
        let mut new_state = self.clone();
        new_state.cells[self.cell_index(coordinate)] = Some(token);

        new_state
    }

    fn get_cell(&self, c: Coordinate) -> Cell {
        self.cells[self.cell_index(c)].clone()
    }

    fn cells_neighbor_coordinates(&self, cell_coordinates: Coordinate) -> Vec<Coordinate> {
        let mut neighbors = Vec::new();
        if cell_coordinates.x > 0 {
            neighbors.push(c(cell_coordinates.x - 1, cell_coordinates.y));
        }
        if cell_coordinates.x < self.size.0 - 1 {
            neighbors.push(c(cell_coordinates.x + 1, cell_coordinates.y));
        }
        if cell_coordinates.y > 0 {
            neighbors.push(c(cell_coordinates.x, cell_coordinates.y - 1));
        }
        if cell_coordinates.y < self.size.0 - 1 {
            neighbors.push(c(cell_coordinates.x, cell_coordinates.y + 1));
        }

        neighbors
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " - Board: (current turn: {})", self.current_turn)?;
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                match self.get_cell(c(x, y)) {
                    None => write!(f, ".")?,
                    Some(token) => write!(f, "{token}")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum Move {
    Place(Coordinate),
    Swap(Coordinate, Coordinate),
}

#[derive(Debug, Default)]
struct Solver {}

impl Solver {
    pub fn find_best_move(&self, board: &Board, player: u32) -> Option<Move> {
        if board.check_win_condition() != WinState::NotOver {
            return None;
        }

        let legal_moves = board.get_legal_moves();
        if legal_moves.is_empty() {
            return None;
        }

        for move_ in legal_moves {
            let new_state = board.advance(move_).expect("game logic failed");
            match new_state.check_win_condition() {
                WinState::NotOver => {}
                WinState::Draw => {}
                WinState::Winner(winner) => {
                    if winner == player {
                        return Some(move_);
                    }
                }
            }

            if self.find_best_move(&new_state, player).is_some() {
                return Some(move_);
            }
        }

        None
    }

    fn grade(&self, board: &Board) -> i32 {
        todo!()
    }
}

fn main() {
    let mut board = Board::new((5, 5));

    let solver = Solver::default();

    loop {
        println!("{board}");

        match board.check_win_condition() {
            WinState::NotOver => {}
            WinState::Draw => {
                println!("Game ended in a draw!");
                break;
            }
            WinState::Winner(winner) => {
                println!("Game over, player {} won!", winner);
                break;
            }
        }

        let best_move = solver.find_best_move(&board, 0);
        if let Some(best_move) = best_move {
            board = board.advance(best_move).expect("game logic failed");
        } else {
            assert_eq!(
                board.check_win_condition(),
                WinState::NotOver,
                "game is not over but we did not find good moves"
            );
            break;
        }
    }

    println!("{board}");
}
