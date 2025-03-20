use std::{collections::HashMap, fmt};

use rustc_hash::{FxBuildHasher, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    color::Color,
    game::PreviousBoards,
    play::{BOARD_LETTERS, Plae, Play, Vertex},
    space::Space,
    status::Status,
};

pub const STARTING_POSITION: [&str; 11] = [
    "...XXXXX...",
    ".....X.....",
    "...........",
    "X....O....X",
    "X...OOO...X",
    "XX.OOKOO.XX",
    "X...OOO...X",
    "X....O....X",
    "...........",
    ".....X.....",
    "...XXXXX...",
];

const EXIT_SQUARES: [Vertex; 4] = [
    Vertex { x: 0, y: 0 },
    Vertex { x: 10, y: 0 },
    Vertex { x: 0, y: 10 },
    Vertex { x: 10, y: 10 },
];

const THRONE: Vertex = Vertex { x: 5, y: 5 };

const RESTRICTED_SQUARES: [Vertex; 5] = [
    Vertex { x: 0, y: 0 },
    Vertex { x: 10, y: 0 },
    Vertex { x: 0, y: 10 },
    Vertex { x: 10, y: 10 },
    THRONE,
];

#[serde_as]
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Board {
    #[serde_as(as = "[_; 121]")]
    pub spaces: [Space; 11 * 11],
}

impl Default for Board {
    fn default() -> Self {
        STARTING_POSITION.try_into().unwrap()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for y in 0..11 {
            write!(f, r#"""#)?;

            for x in 0..11 {
                match self.spaces[(y * 10) + x] {
                    Space::Black => write!(f, "X")?,
                    Space::Empty => write!(f, ".")?,
                    Space::King => write!(f, "K")?,
                    Space::White => write!(f, "O")?,
                }
            }
            writeln!(f, r#"""#)?;
        }

        Ok(())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut letters = " ".repeat(3).to_string();
        letters.push_str(BOARD_LETTERS);
        let bar = "─".repeat(11);

        writeln!(f, "\n{letters}\n  ┌{bar}┐")?;
        for y in 0..11 {
            let y_label = 11 - y;
            write!(f, "{y_label:2}│",)?;

            for x in 0..11 {
                if ((y, x) == (0, 0)
                    || (y, x) == (10, 0)
                    || (y, x) == (0, 10)
                    || (y, x) == (10, 10)
                    || (y, x) == (5, 5))
                    && self.spaces[y * 11 + x] == Space::Empty
                {
                    write!(f, "■")?;
                } else {
                    write!(f, "{}", self.spaces[y * 11 + x])?;
                }
            }
            writeln!(f, "│{y_label:2}")?;
        }
        write!(f, "  └{bar}┘\n{letters}")
    }
}

impl TryFrom<[&str; 11]> for Board {
    type Error = anyhow::Error;

    fn try_from(value: [&str; 11]) -> anyhow::Result<Self> {
        let mut spaces = [Space::Empty; 11 * 11];
        let mut kings = 0;

        for (y, row) in value.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let space = ch.try_into()?;
                match space {
                    Space::Black | Space::White => {
                        let vertex = Vertex { x, y };
                        if RESTRICTED_SQUARES.contains(&vertex) {
                            return Err(anyhow::Error::msg(
                                "Only the king is allowed on restricted squares!",
                            ));
                        }
                    }
                    Space::Empty => {}
                    Space::King => {
                        kings += 1;
                        if kings > 1 {
                            return Err(anyhow::Error::msg("You can only have one king!"));
                        }
                    }
                }

                spaces[y * 11 + x] = space;
            }
        }

        Ok(Self { spaces })
    }
}

impl Board {
    fn able_to_move(&self, play_from: &Vertex) -> bool {
        if let Some(vertex) = play_from.up() {
            if self.get(&vertex) == Space::Empty {
                return true;
            }
        }

        if let Some(vertex) = play_from.left() {
            if self.get(&vertex) == Space::Empty {
                return true;
            }
        }

        if let Some(vertex) = play_from.down() {
            if self.get(&vertex) == Space::Empty {
                return true;
            }
        }

        if let Some(vertex) = play_from.right() {
            if self.get(&vertex) == Space::Empty {
                return true;
            }
        }

        false
    }

    #[must_use]
    pub fn a_legal_move_exists(
        &self,
        status: &Status,
        turn: &Color,
        previous_boards: &PreviousBoards,
    ) -> bool {
        let mut possible_vertexes = Vec::new();

        for y in 0..11 {
            for x in 0..11 {
                let vertex = Vertex { x, y };
                if self.get(&vertex).color() == *turn {
                    possible_vertexes.push(vertex);
                }
            }
        }

        if possible_vertexes.is_empty() {
            return false;
        }

        for vertex_from in possible_vertexes {
            for y in 0..11 {
                for x in 0..11 {
                    let vertex_to = Vertex { x, y };
                    let play = Play {
                        color: turn.clone(),
                        from: vertex_from.clone(),
                        to: vertex_to,
                    };

                    if let Ok(_board_captures_status) =
                        self.play_internal(&Plae::Play(play), status, turn, previous_boards)
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    #[allow(clippy::collapsible_if)]
    fn captures(&mut self, play_to: &Vertex, color_from: &Color, captures: &mut Vec<Vertex>) {
        if let Some(up_1) = play_to.up() {
            let space = self.get(&up_1);
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(up_2) = up_1.up() {
                    if (RESTRICTED_SQUARES.contains(&up_2) && self.get(&up_2) != Space::King)
                        || self.get(&up_2).color() == *color_from
                    {
                        if self.set_if_not_king(&up_1, Space::Empty) {
                            captures.push(up_1);
                        }
                    }
                }
            }
        }

        if let Some(left_1) = play_to.left() {
            let space = self.get(&left_1);
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(left_2) = left_1.left() {
                    if (RESTRICTED_SQUARES.contains(&left_2) && self.get(&left_2) != Space::King)
                        || self.get(&left_2).color() == *color_from
                    {
                        if self.set_if_not_king(&left_1, Space::Empty) {
                            captures.push(left_1);
                        }
                    }
                }
            }
        }

        if let Some(down_1) = play_to.down() {
            let space = self.get(&down_1);
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(down_2) = down_1.down() {
                    if (RESTRICTED_SQUARES.contains(&down_2) && self.get(&down_2) != Space::King)
                        || self.get(&down_2).color() == *color_from
                    {
                        if self.set_if_not_king(&down_1, Space::Empty) {
                            captures.push(down_1);
                        }
                    }
                }
            }
        }

        if let Some(right_1) = play_to.right() {
            let space = self.get(&right_1);
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(right_2) = right_1.right() {
                    if (RESTRICTED_SQUARES.contains(&right_2) && self.get(&right_2) != Space::King)
                        || self.get(&right_2).color() == *color_from
                    {
                        if self.set_if_not_king(&right_1, Space::Empty) {
                            captures.push(right_1);
                        }
                    }
                }
            }
        }
    }

    // y counts up going down.
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::collapsible_if)]
    fn captures_shield_wall(
        &mut self,
        color_from: &Color,
        vertex_to: &Vertex,
        captures: &mut Vec<Vertex>,
    ) {
        // bottom row
        for x_1 in 0..11 {
            let vertex_1 = Vertex { x: x_1, y: 10 };
            if self.get(&vertex_1).color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if x_1 == 10 {
                    break;
                }
                let start = x_1 + 1;

                for x_2 in start..11 {
                    let vertex_2 = Vertex { x: x_2, y: 10 };
                    let vertex_3 = Vertex { x: x_2, y: 9 };
                    let color_2 = self.get(&vertex_2).color();
                    let color_3 = self.get(&vertex_3).color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: finish, y: 10 };
                let color = self.get(&vertex).color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    if vertex_to
                        == &(Vertex {
                            x: start - 1,
                            y: 10,
                        })
                        || vertex_to == &(Vertex { x: finish, y: 10 })
                    {
                        for x_2 in start..finish {
                            let vertex = Vertex { x: x_2, y: 10 };
                            if self.set_if_not_king(&vertex, Space::Empty) {
                                captures.push(vertex);
                            }
                        }
                    }
                }
            }
        }

        // top row
        for x_1 in 0..11 {
            let vertex_1 = Vertex { x: x_1, y: 0 };
            if self.get(&vertex_1).color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if x_1 == 10 {
                    break;
                }
                let start = x_1 + 1;

                for x_2 in start..11 {
                    let vertex_2 = Vertex { x: x_2, y: 0 };
                    let vertex_3 = Vertex { x: x_2, y: 1 };
                    let color_2 = self.get(&vertex_2).color();
                    let color_3 = self.get(&vertex_3).color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: finish, y: 0 };
                let color = self.get(&vertex).color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    if vertex_to == &(Vertex { x: start - 1, y: 0 })
                        || vertex_to == &(Vertex { x: finish, y: 0 })
                    {
                        for x_2 in start..finish {
                            let vertex = Vertex { x: x_2, y: 0 };
                            if self.set_if_not_king(&vertex, Space::Empty) {
                                captures.push(vertex);
                            }
                        }
                    }
                }
            }
        }

        // left row
        for y_1 in 0..11 {
            let vertex_1 = Vertex { x: 0, y: y_1 };
            if self.get(&vertex_1).color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if y_1 == 10 {
                    break;
                }
                let start = y_1 + 1;

                for y_2 in start..11 {
                    let vertex_2 = Vertex { x: 0, y: y_2 };
                    let vertex_3 = Vertex { x: 1, y: y_2 };
                    let color_2 = self.get(&vertex_2).color();
                    let color_3 = self.get(&vertex_3).color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: 0, y: finish };
                let color = self.get(&vertex).color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    if vertex_to == &(Vertex { x: 0, y: start - 1 })
                        || vertex_to == &(Vertex { x: 0, y: finish })
                    {
                        for y_2 in start..finish {
                            let vertex = Vertex { x: 0, y: y_2 };
                            if self.set_if_not_king(&vertex, Space::Empty) {
                                captures.push(vertex);
                            }
                        }
                    }
                }
            }
        }

        // right row
        for y_1 in 0..11 {
            let vertex_1 = Vertex { x: 10, y: y_1 };
            if self.get(&vertex_1).color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if y_1 == 10 {
                    break;
                }
                let start = y_1 + 1;

                for y_2 in start..11 {
                    let vertex_2 = Vertex { x: 10, y: y_2 };
                    let vertex_3 = Vertex { x: 9, y: y_2 };
                    let color_2 = self.get(&vertex_2).color();
                    let color_3 = self.get(&vertex_3).color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: 10, y: finish };
                let color = self.get(&vertex).color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    if vertex_to
                        == &(Vertex {
                            x: 10,
                            y: start - 1,
                        })
                        || vertex_to == &(Vertex { x: 10, y: finish })
                    {
                        for y_2 in start..finish {
                            let vertex = Vertex { x: 10, y: y_2 };
                            if self.set_if_not_king(&vertex, Space::Empty) {
                                captures.push(vertex);
                            }
                        }
                    }
                }
            }
        }
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    pub fn find_the_king(&self) -> anyhow::Result<Option<Vertex>> {
        for y in 0..11 {
            for x in 0..11 {
                let v = Vertex { x, y };
                if self.get(&v) == Space::King {
                    return Ok(Some(v));
                }
            }
        }

        Ok(None)
    }

    fn capture_the_king(
        &self,
        play_to: &Vertex,
        captures: &mut Vec<Vertex>,
    ) -> anyhow::Result<bool> {
        let mut played_to_capture = false;

        match self.find_the_king()? {
            Some(kings_vertex) => {
                if let Some(vertex) = kings_vertex.up() {
                    if play_to == &vertex {
                        played_to_capture = true;
                    }

                    if vertex != THRONE && self.get(&vertex) != Space::Black {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }

                if let Some(vertex) = kings_vertex.left() {
                    if play_to == &vertex {
                        played_to_capture = true;
                    }

                    if vertex != THRONE && self.get(&vertex) != Space::Black {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }

                if let Some(vertex) = kings_vertex.down() {
                    if play_to == &vertex {
                        played_to_capture = true;
                    }

                    if vertex != THRONE && self.get(&vertex) != Space::Black {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }

                if let Some(vertex) = kings_vertex.right() {
                    if play_to == &vertex {
                        played_to_capture = true;
                    }

                    if vertex != THRONE && self.get(&vertex) != Space::Black {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }

                if played_to_capture {
                    captures.push(kings_vertex);
                    return Ok(true);
                }

                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    fn exit_forts(&self) -> anyhow::Result<bool> {
        match self.find_the_king()? {
            Some(kings_vertex) => {
                if !kings_vertex.touches_wall()
                    || !self.able_to_move(&kings_vertex)
                    || !self.flood_fill_white_wins(&kings_vertex)?
                {
                    return Ok(false);
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    fn flood_fill_black_wins(&self) -> anyhow::Result<bool> {
        match self.find_the_king()? {
            Some(kings_vertex) => {
                let hasher = FxBuildHasher;
                let mut already_checked = FxHashSet::with_capacity_and_hasher(11 * 11, hasher);

                already_checked.insert(kings_vertex.clone());
                let mut stack = Vec::new();
                stack.push(kings_vertex);

                while !stack.is_empty() {
                    if let Some(vertex) = stack.pop() {
                        let space = self.get(&vertex);
                        if space == Space::Empty || space.color() == Color::White {
                            if !expand_flood_fill(vertex.up(), &mut already_checked, &mut stack) {
                                return Ok(false);
                            }
                            if !expand_flood_fill(vertex.left(), &mut already_checked, &mut stack) {
                                return Ok(false);
                            }
                            if !expand_flood_fill(vertex.down(), &mut already_checked, &mut stack) {
                                return Ok(false);
                            }
                            if !expand_flood_fill(vertex.right(), &mut already_checked, &mut stack)
                            {
                                return Ok(false);
                            }
                        }
                    }
                }

                for y in 0..11 {
                    for x in 0..11 {
                        let vertex = Vertex { x, y };
                        if self.get(&vertex).color() == Color::White
                            && !already_checked.contains(&vertex)
                        {
                            return Ok(false);
                        }
                    }
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    #[allow(clippy::too_many_lines)]
    pub fn flood_fill_white_wins(&self, vertex: &Vertex) -> anyhow::Result<bool> {
        let mut black_has_enough_pieces = false;
        let mut count = 0;
        'outer: for y in 0..11 {
            for x in 0..11 {
                let vertex = Vertex { x, y };
                if self.get(&vertex).color() == Color::Black {
                    count += 1;
                }

                if count > 1 {
                    black_has_enough_pieces = true;
                    break 'outer;
                }
            }
        }

        let mut already_checked = FxHashSet::default();
        let mut stack = vec![];

        if let Some(vertex) = vertex.up() {
            stack.push((vertex, Direction::LeftRight));
        }
        if let Some(vertex) = vertex.left() {
            stack.push((vertex, Direction::UpDown));
        }
        if let Some(vertex) = vertex.down() {
            stack.push((vertex, Direction::LeftRight));
        }
        if let Some(vertex) = vertex.right() {
            stack.push((vertex, Direction::UpDown));
        }

        while !stack.is_empty() {
            if let Some((vertex, direction)) = stack.pop() {
                let space = self.get(&vertex);
                if space == Space::Empty {
                    if let Some(vertex) = vertex.up() {
                        if !already_checked.contains(&vertex) {
                            stack.push((vertex.clone(), Direction::LeftRight));
                            already_checked.insert(vertex);
                        }
                    }
                    if let Some(vertex) = vertex.left() {
                        if !already_checked.contains(&vertex) {
                            stack.push((vertex.clone(), Direction::UpDown));
                            already_checked.insert(vertex);
                        }
                    }
                    if let Some(vertex) = vertex.down() {
                        if !already_checked.contains(&vertex) {
                            stack.push((vertex.clone(), Direction::LeftRight));
                            already_checked.insert(vertex);
                        }
                    }
                    if let Some(vertex) = vertex.right() {
                        if !already_checked.contains(&vertex) {
                            stack.push((vertex.clone(), Direction::UpDown));
                            already_checked.insert(vertex);
                        }
                    }
                } else if space.color() == Color::Black {
                    return Ok(false);
                } else if direction == Direction::UpDown {
                    let mut vertex_1 = false;
                    let mut vertex_2 = false;

                    if let Some(vertex) = vertex.up() {
                        if self.get(&vertex).color() == Color::White {
                            vertex_1 = true;
                        }
                    } else {
                        vertex_1 = true;
                    }
                    if let Some(vertex) = vertex.down() {
                        if self.get(&vertex).color() == Color::White {
                            vertex_2 = true;
                        }
                    } else {
                        vertex_2 = true;
                    }

                    if !vertex_1 && !vertex_2 && black_has_enough_pieces {
                        return Ok(false);
                    }
                } else {
                    let mut vertex_1 = false;
                    let mut vertex_2 = false;

                    if let Some(vertex) = vertex.right() {
                        if self.get(&vertex).color() == Color::White {
                            vertex_1 = true;
                        }
                    } else {
                        vertex_1 = true;
                    }
                    if let Some(vertex) = vertex.left() {
                        if self.get(&vertex).color() == Color::White {
                            vertex_2 = true;
                        }
                    } else {
                        vertex_2 = true;
                    }

                    if !vertex_1 && !vertex_2 && black_has_enough_pieces {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    #[must_use]
    pub fn get(&self, vertex: &Vertex) -> Space {
        self.spaces[vertex.y * 11 + vertex.x]
    }

    #[must_use]
    fn no_black_pieces_left(&self) -> bool {
        for y in 0..11 {
            for x in 0..11 {
                let v = Vertex { x, y };
                if self.get(&v).color() == Color::Black {
                    return false;
                }
            }
        }

        true
    }

    #[must_use]
    pub fn pieces_taken(&self) -> (u8, u8) {
        let mut black_pieces = 0;
        let mut white_pieces = 0;

        for space in self.spaces {
            match space {
                Space::Black => black_pieces += 1,
                Space::Empty => {}
                Space::King | Space::White => white_pieces += 1,
            }
        }

        (24 - black_pieces, 13 - white_pieces)
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    pub fn play(
        &mut self,
        play: &Plae,
        status: &Status,
        turn: &Color,
        previous_boards: &mut PreviousBoards,
    ) -> anyhow::Result<(Vec<Vertex>, Status)> {
        let (board, captures, status) = self.play_internal(play, status, turn, previous_boards)?;
        previous_boards.0.insert(board.clone());
        *self = board;

        Ok((captures, status))
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::missing_errors_doc
    )]
    pub fn play_internal(
        &self,
        play: &Plae,
        status: &Status,
        turn: &Color,
        previous_boards: &PreviousBoards,
    ) -> anyhow::Result<(Board, Vec<Vertex>, Status)> {
        if *status != Status::Ongoing {
            return Err(anyhow::Error::msg(
                "play: the game has to be ongoing to play",
            ));
        }

        let play = match play {
            Plae::BlackResigns => return Ok((self.clone(), Vec::new(), Status::WhiteWins)),
            Plae::WhiteResigns => return Ok((self.clone(), Vec::new(), Status::BlackWins)),
            Plae::Play(play) => play,
        };

        let space_from = self.get(&play.from);
        let color_from = space_from.color();

        if color_from == Color::Colorless {
            return Err(anyhow::Error::msg("play: you didn't select a color"));
        } else if *turn != color_from {
            return Err(anyhow::Error::msg("play: it isn't your turn"));
        }

        let x_diff = play.from.x as i32 - play.to.x as i32;
        let y_diff = play.from.y as i32 - play.to.y as i32;

        if x_diff != 0 && y_diff != 0 {
            return Err(anyhow::Error::msg(
                "play: you can only play in a straight line",
            ));
        }

        if x_diff == 0 && y_diff == 0 {
            return Err(anyhow::Error::msg("play: you have to change location"));
        }

        if x_diff != 0 {
            let x_diff_sign = x_diff.signum();
            for x_diff in 1..=x_diff.abs() {
                let vertex = Vertex {
                    x: (play.from.x as i32 - (x_diff * x_diff_sign)) as usize,
                    y: play.from.y,
                };

                let space = self.get(&vertex);
                if space != Space::Empty {
                    return Err(anyhow::Error::msg(
                        "play: you have to play through empty locations",
                    ));
                }
            }
        } else {
            let y_diff_sign = y_diff.signum();
            for y_diff in 1..=y_diff.abs() {
                let vertex = Vertex {
                    x: play.from.x,
                    y: (play.from.y as i32 - (y_diff * y_diff_sign)) as usize,
                };
                let space = self.get(&vertex);
                if space != Space::Empty {
                    return Err(anyhow::Error::msg(
                        "play: you have to play through empty locations",
                    ));
                }
            }
        }

        if space_from != Space::King && RESTRICTED_SQUARES.contains(&play.to) {
            return Err(anyhow::Error::msg(
                "play: only the king may move to a restricted square",
            ));
        }

        let mut board = self.clone();
        board.set(&play.from, Space::Empty);
        board.set(&play.to, space_from);

        if previous_boards.0.contains(&board) {
            return Err(anyhow::Error::msg(
                "play: you already reached that position",
            ));
        }

        let mut captures = Vec::new();
        board.captures(&play.to, &color_from, &mut captures);
        board.captures_shield_wall(&color_from, &play.to, &mut captures);

        if EXIT_SQUARES.contains(&play.to) {
            return Ok((board, captures, Status::WhiteWins));
        }

        if board.capture_the_king(&play.to, &mut captures)? {
            return Ok((board, captures, Status::BlackWins));
        }

        if board.exit_forts()? {
            return Ok((board, captures, Status::WhiteWins));
        }
        if board.flood_fill_black_wins()? {
            return Ok((board, captures, Status::BlackWins));
        }

        if board.no_black_pieces_left() {
            return Ok((board, captures, Status::WhiteWins));
        }

        // Todo: Is a draw possible, how?

        Ok((board, captures, Status::Ongoing))
    }

    fn set(&mut self, vertex: &Vertex, space: Space) {
        self.spaces[vertex.y * 11 + vertex.x] = space;
    }

    #[must_use]
    fn set_if_not_king(&mut self, vertex: &Vertex, space: Space) -> bool {
        if self.get(vertex) == Space::King {
            false
        } else {
            self.set(vertex, space);
            true
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Direction {
    LeftRight,
    UpDown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalMoves {
    pub color: Color,
    pub moves: HashMap<Vertex, Vec<Vertex>>,
}

#[must_use]
fn expand_flood_fill(
    vertex: Option<Vertex>,
    already_checked: &mut FxHashSet<Vertex>,
    stack: &mut Vec<Vertex>,
) -> bool {
    if let Some(vertex) = vertex {
        if !already_checked.contains(&vertex) {
            stack.push(vertex.clone());
            already_checked.insert(vertex);
        }

        true
    } else {
        false
    }
}
