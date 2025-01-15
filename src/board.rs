use std::{collections::HashSet, fmt};

use anyhow::Context;

use crate::{
    color::Color,
    game::PreviousBoards,
    play::{Play, Vertex},
    status::Status,
};

use super::space::Space;

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

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Board {
    spaces: [[Space; 11]; 11],
}

impl Default for Board {
    fn default() -> Self {
        STARTING_POSITION.try_into().unwrap()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for line in self.spaces {
            write!(f, r#"""#)?;
            for space in line {
                match space {
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
        let letters = "   ABCDEFGHJKL";
        let bar = "─".repeat(11);

        writeln!(f, "\n{letters}\n  ┌{bar}┐")?;
        for (mut i, line) in self.spaces.iter().enumerate() {
            i = 11 - i;

            write!(f, "{i:2}│")?;
            for (j, space) in line.iter().enumerate() {
                if ((i, j) == (1, 0)
                    || (i, j) == (11, 0)
                    || (i, j) == (1, 10)
                    || (i, j) == (11, 10)
                    || (i, j) == (6, 5))
                    && *space == Space::Empty
                {
                    write!(f, "■")?;
                } else {
                    write!(f, "{space}")?;
                }
            }
            writeln!(f, "│{i:2}")?;
        }
        write!(f, "  └{bar}┘\n{letters}")
    }
}

impl TryFrom<[&str; 11]> for Board {
    type Error = anyhow::Error;

    fn try_from(value: [&str; 11]) -> anyhow::Result<Self> {
        let mut spaces = [[Space::Empty; 11]; 11];
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

                spaces[y][x] = space;
            }
        }

        Ok(Self { spaces })
    }
}

impl Board {
    fn able_to_move(&self, play_from: &Vertex) -> anyhow::Result<bool> {
        if let Some(vertex) = play_from.up() {
            if self.get(&vertex)? == Space::Empty {
                return Ok(true);
            }
        }

        if let Some(vertex) = play_from.left() {
            if self.get(&vertex)? == Space::Empty {
                return Ok(true);
            }
        }

        if let Some(vertex) = play_from.down() {
            if self.get(&vertex)? == Space::Empty {
                return Ok(true);
            }
        }

        if let Some(vertex) = play_from.right() {
            if self.get(&vertex)? == Space::Empty {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    pub fn a_legal_move_exists(
        &self,
        status: &Status,
        turn: &Color,
        previous_boards: &PreviousBoards,
    ) -> anyhow::Result<bool> {
        let mut possible_vertexes = Vec::new();

        for y in 0..11 {
            for x in 0..11 {
                let vertex = Vertex { x, y };
                if self.get(&vertex)?.color() == *turn {
                    possible_vertexes.push(vertex);
                }
            }
        }

        if possible_vertexes.is_empty() {
            return Ok(false);
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

                    let mut board = self.clone();
                    if let Ok(_status) =
                        board.play_internal(&play, status, turn, &mut previous_boards.clone())
                    {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    pub fn all_legal_moves(
        &self,
        status: &Status,
        turn: &Color,
        previous_boards: &PreviousBoards,
    ) -> anyhow::Result<Vec<LegalMoves>> {
        let mut legal_moves_all = Vec::new();
        let mut possible_vertexes = Vec::new();

        for y in 0..11 {
            for x in 0..11 {
                let vertex = Vertex { x, y };
                if self.get(&vertex)?.color() == *turn {
                    possible_vertexes.push(vertex);
                }
            }
        }

        for vertex_from in possible_vertexes {
            let mut vertexes_to = Vec::new();

            for y in 0..11 {
                for x in 0..11 {
                    let vertex_to = Vertex { x, y };
                    let play = Play {
                        color: turn.clone(),
                        from: vertex_from.clone(),
                        to: vertex_to.clone(),
                    };

                    let mut board = self.clone();
                    if let Ok(_status) =
                        board.play_internal(&play, status, turn, &mut previous_boards.clone())
                    {
                        vertexes_to.push(vertex_to);
                    }
                }
            }

            if !vertexes_to.is_empty() {
                let legal_moves = LegalMoves {
                    color: turn.clone(),
                    from: vertex_from,
                    to: vertexes_to,
                };

                legal_moves_all.push(legal_moves);
            }
        }

        Ok(legal_moves_all)
    }

    #[allow(clippy::collapsible_if)]
    fn captures(
        &mut self,
        play_to: &Vertex,
        color_from: &Color,
        captures: &mut Vec<Vertex>,
    ) -> anyhow::Result<()> {
        if let Some(up_1) = play_to.up() {
            let space = self.get(&up_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(up_2) = up_1.up() {
                    if (RESTRICTED_SQUARES.contains(&up_2) && self.get(&up_2)? != Space::King)
                        || self.get(&up_2)?.color() == *color_from
                    {
                        if self.set_if_not_king(&up_1, Space::Empty)? {
                            captures.push(up_1);
                        }
                    }
                }
            }
        }

        if let Some(left_1) = play_to.left() {
            let space = self.get(&left_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(left_2) = left_1.left() {
                    if (RESTRICTED_SQUARES.contains(&left_2) && self.get(&left_2)? != Space::King)
                        || self.get(&left_2)?.color() == *color_from
                    {
                        if self.set_if_not_king(&left_1, Space::Empty)? {
                            captures.push(left_1);
                        }
                    }
                }
            }
        }

        if let Some(down_1) = play_to.down() {
            let space = self.get(&down_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(down_2) = down_1.down() {
                    if (RESTRICTED_SQUARES.contains(&down_2) && self.get(&down_2)? != Space::King)
                        || self.get(&down_2)?.color() == *color_from
                    {
                        if self.set_if_not_king(&down_1, Space::Empty)? {
                            captures.push(down_1);
                        }
                    }
                }
            }
        }

        if let Some(right_1) = play_to.right() {
            let space = self.get(&right_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(right_2) = right_1.right() {
                    if (RESTRICTED_SQUARES.contains(&right_2) && self.get(&right_2)? != Space::King)
                        || self.get(&right_2)?.color() == *color_from
                    {
                        if self.set_if_not_king(&right_1, Space::Empty)? {
                            captures.push(right_1);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // y counts up going down.
    #[allow(clippy::too_many_lines)]
    fn captures_shield_wall(
        &mut self,
        color_from: &Color,
        captures: &mut Vec<Vertex>,
    ) -> anyhow::Result<()> {
        // bottom row
        for x_1 in 0..11 {
            let vertex_1 = Vertex { x: x_1, y: 10 };
            if self.get(&vertex_1)?.color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if x_1 == 10 {
                    break;
                }
                let start = x_1 + 1;

                for x_2 in start..11 {
                    let vertex_2 = Vertex { x: x_2, y: 10 };
                    let vertex_3 = Vertex { x: x_2, y: 9 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: finish, y: 10 };
                let color = self.get(&vertex)?.color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for x_2 in start..finish {
                        let vertex = Vertex { x: x_2, y: 10 };
                        if self.set_if_not_king(&vertex, Space::Empty)? {
                            captures.push(vertex);
                        }
                    }
                }
            }
        }

        // top row
        for x_1 in 0..11 {
            let vertex_1 = Vertex { x: x_1, y: 0 };
            if self.get(&vertex_1)?.color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if x_1 == 10 {
                    break;
                }
                let start = x_1 + 1;

                for x_2 in start..11 {
                    let vertex_2 = Vertex { x: x_2, y: 0 };
                    let vertex_3 = Vertex { x: x_2, y: 1 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: finish, y: 0 };
                let color = self.get(&vertex)?.color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for x_2 in start..finish {
                        let vertex = Vertex { x: x_2, y: 0 };
                        if self.set_if_not_king(&vertex, Space::Empty)? {
                            captures.push(vertex);
                        }
                    }
                }
            }
        }

        // left row
        for y_1 in 0..11 {
            let vertex_1 = Vertex { x: 0, y: y_1 };
            if self.get(&vertex_1)?.color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if y_1 == 10 {
                    break;
                }
                let start = y_1 + 1;

                for y_2 in start..11 {
                    let vertex_2 = Vertex { x: 0, y: y_2 };
                    let vertex_3 = Vertex { x: 1, y: y_2 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: 0, y: finish };
                let color = self.get(&vertex)?.color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for y_2 in start..finish {
                        let vertex = Vertex { x: 0, y: y_2 };
                        if self.set_if_not_king(&vertex, Space::Empty)? {
                            captures.push(vertex);
                        }
                    }
                }
            }
        }

        // right row
        for y_1 in 0..11 {
            let vertex_1 = Vertex { x: 10, y: y_1 };
            if self.get(&vertex_1)?.color() == *color_from || RESTRICTED_SQUARES.contains(&vertex_1)
            {
                let mut count = 0;

                if y_1 == 10 {
                    break;
                }
                let start = y_1 + 1;

                for y_2 in start..11 {
                    let vertex_2 = Vertex { x: 10, y: y_2 };
                    let vertex_3 = Vertex { x: 9, y: y_2 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: 10, y: finish };
                let color = self.get(&vertex)?.color();
                if count > 1 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for y_2 in start..finish {
                        let vertex = Vertex { x: 10, y: y_2 };
                        if self.set_if_not_king(&vertex, Space::Empty)? {
                            captures.push(vertex);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    fn find_the_king(&self) -> anyhow::Result<Option<Vertex>> {
        for y in 0..11 {
            for x in 0..11 {
                let v = Vertex { x, y };
                if self.get(&v)? == Space::King {
                    return Ok(Some(v));
                }
            }
        }

        Ok(None)
    }

    fn capture_the_king(&self, captures: &mut Vec<Vertex>) -> anyhow::Result<bool> {
        if let Some(kings_vertex) = self.find_the_king()? {
            if let Some(vertex) = kings_vertex.up() {
                if vertex != THRONE && self.get(&vertex)? != Space::Black {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            if let Some(vertex) = kings_vertex.left() {
                if vertex != THRONE && self.get(&vertex)? != Space::Black {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            if let Some(vertex) = kings_vertex.down() {
                if vertex != THRONE && self.get(&vertex)? != Space::Black {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            if let Some(vertex) = kings_vertex.right() {
                if vertex != THRONE && self.get(&vertex)? != Space::Black {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            captures.push(kings_vertex);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    fn exit_forts(&self) -> anyhow::Result<bool> {
        if let Some(kings_vertex) = self.find_the_king()? {
            if !kings_vertex.touches_wall()
                || !self.able_to_move(&kings_vertex)?
                || !self.flood_fill_white_wins(&kings_vertex)?
            {
                return Ok(false);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    fn flood_fill_black_wins(&self) -> anyhow::Result<bool> {
        if let Some(kings_vertex) = self.find_the_king()? {
            let mut already_checked = HashSet::new();
            already_checked.insert(kings_vertex.clone());
            let mut stack = Vec::new();
            stack.push(kings_vertex);

            while !stack.is_empty() {
                if let Some(vertex) = stack.pop() {
                    let space = self.get(&vertex)?;
                    if space == Space::Empty || space.color() == Color::White {
                        if let Some(vertex) = vertex.up() {
                            if !already_checked.contains(&vertex) {
                                stack.push(vertex.clone());
                                already_checked.insert(vertex);
                            }
                        } else {
                            return Ok(false);
                        }
                        if let Some(vertex) = vertex.left() {
                            if !already_checked.contains(&vertex) {
                                stack.push(vertex.clone());
                                already_checked.insert(vertex);
                            }
                        } else {
                            return Ok(false);
                        }
                        if let Some(vertex) = vertex.down() {
                            if !already_checked.contains(&vertex) {
                                stack.push(vertex.clone());
                                already_checked.insert(vertex);
                            }
                        } else {
                            return Ok(false);
                        }
                        if let Some(vertex) = vertex.right() {
                            if !already_checked.contains(&vertex) {
                                stack.push(vertex.clone());
                                already_checked.insert(vertex);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                }
            }

            for y in 0..11 {
                for x in 0..11 {
                    let vertex = Vertex { x, y };
                    if self.get(&vertex)?.color() == Color::White
                        && !already_checked.contains(&vertex)
                    {
                        return Ok(false);
                    }
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// # Errors
    ///
    /// If the vertex is out of bounds.
    pub fn flood_fill_white_wins(&self, vertex: &Vertex) -> anyhow::Result<bool> {
        let mut black_has_enough_pieces = false;
        let mut count = 0;
        'outer: for y in 0..11 {
            for x in 0..11 {
                let vertex = Vertex { x, y };
                if self.get(&vertex)?.color() == Color::Black {
                    count += 1;
                }

                if count > 1 {
                    black_has_enough_pieces = true;
                    break 'outer;
                }
            }
        }

        let mut already_checked = HashSet::new();
        let mut stack = Vec::new();

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
                let space = self.get(&vertex)?;
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
                        if self.get(&vertex)?.color() == Color::White {
                            vertex_1 = true;
                        }
                    }
                    if let Some(vertex) = vertex.down() {
                        if self.get(&vertex)?.color() == Color::White {
                            vertex_2 = true;
                        }
                    }

                    if !vertex_1 && !vertex_2 && black_has_enough_pieces {
                        return Ok(false);
                    }
                } else {
                    let mut vertex_1 = false;
                    let mut vertex_2 = false;

                    if let Some(vertex) = vertex.right() {
                        if self.get(&vertex)?.color() == Color::White {
                            vertex_1 = true;
                        }
                    }
                    if let Some(vertex) = vertex.left() {
                        if self.get(&vertex)?.color() == Color::White {
                            vertex_2 = true;
                        }
                    }

                    if !vertex_1 && !vertex_2 && black_has_enough_pieces {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// # Errors
    ///
    /// If the play is out of bounds.
    pub fn get(&self, vertex: &Vertex) -> anyhow::Result<Space> {
        let column = self
            .spaces
            .get(vertex.y)
            .context("get: index is out of y bounds")?;

        Ok(*column
            .get(vertex.x)
            .context("get: index is out of x bounds")?)
    }

    /// # Errors
    ///
    /// If the play is out of bounds.
    fn no_black_pieces_left(&self) -> anyhow::Result<bool> {
        for y in 0..11 {
            for x in 0..11 {
                let v = Vertex { x, y };
                if self.get(&v)?.color() == Color::Black {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// # Errors
    ///
    /// If the play is illegal.
    pub fn play(
        &mut self,
        play: &Play,
        status: &Status,
        turn: &Color,
        previous_boards: &mut PreviousBoards,
    ) -> anyhow::Result<(Vec<Vertex>, Status)> {
        match self.play_internal(play, status, turn, previous_boards) {
            Ok((captures, Status::Ongoing)) => {
                if self.a_legal_move_exists(status, &turn.opposite(), previous_boards)? {
                    Ok((captures, Status::Ongoing))
                } else {
                    if turn.opposite() == Color::White {
                        return Ok((captures, Status::BlackWins));
                    }

                    Ok((captures, Status::WhiteWins))
                }
            }
            result => result,
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    fn play_internal(
        &mut self,
        play: &Play,
        status: &Status,
        turn: &Color,
        previous_boards: &mut PreviousBoards,
    ) -> anyhow::Result<(Vec<Vertex>, Status)> {
        if *status != Status::Ongoing {
            return Err(anyhow::Error::msg(
                "play: the game has to be ongoing to play",
            ));
        }

        let space_from = self.get(&play.from)?;
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

                let space = self.get(&vertex)?;
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
                let space = self.get(&vertex)?;
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

        previous_boards.0.insert(board);
        self.set(&play.from, Space::Empty);
        self.set(&play.to, space_from);

        let mut captures = Vec::new();

        if EXIT_SQUARES.contains(&play.to) || self.exit_forts()? {
            return Ok((captures, Status::WhiteWins));
        }

        if self.capture_the_king(&mut captures)? || self.flood_fill_black_wins()? {
            return Ok((captures, Status::BlackWins));
        }

        self.captures(&play.to, &color_from, &mut captures)?;
        self.captures_shield_wall(&color_from, &mut captures)?;

        if self.no_black_pieces_left()? {
            return Ok((captures, Status::WhiteWins));
        }

        // Todo: Is a draw possible, how?

        Ok((captures, Status::Ongoing))
    }

    fn set(&mut self, vertex: &Vertex, space: Space) {
        self.spaces[vertex.y][vertex.x] = space;
    }

    fn set_if_not_king(&mut self, vertex: &Vertex, space: Space) -> anyhow::Result<bool> {
        if self.get(vertex)? == Space::King {
            Ok(false)
        } else {
            self.set(vertex, space);
            Ok(true)
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
    pub from: Vertex,
    pub to: Vec<Vertex>,
}
