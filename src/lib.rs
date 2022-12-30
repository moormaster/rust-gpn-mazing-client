use std::fmt::Display;

#[derive(Clone)]
#[derive(PartialEq)]

pub enum State {
    Up,
    Down,
    Left,
    Right,
    Visited,
    Unvisited
}

#[derive(Clone)]
#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right   
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => f.write_str("up"),
            Direction::Down => f.write_str("down"),
            Direction::Left => f.write_str("left"),
            Direction::Right => f.write_str("right")
        }?;

        std::fmt::Result::Ok(())
    }
}

impl State {
    fn from_direction(direction: &Direction) -> State {
        match direction {
            Direction::Up => State::Up,
            Direction::Down => State::Down,
            Direction::Left => State::Left,
            Direction::Right => State::Right
        }
    }
}

impl Direction {
    fn inverted_direction_from_state(state: &State) -> Option<Direction> {
        match state {
            State::Up => Some(Direction::Down),
            State::Down => Some(Direction::Up),
            State::Left => Some(Direction::Right),
            State::Right => Some(Direction::Left),
            _ => None
        }
    }
}

#[derive(PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position {x, y}
    }
}

pub struct FieldEnvironment {
    pub has_left_wall: bool,
    pub has_right_wall: bool,
    pub has_upper_wall: bool,
    pub has_lower_wall: bool
}

impl FieldEnvironment {
    pub fn new(has_left_wall: bool, has_right_wall: bool, has_upper_wall: bool, has_lower_wall: bool) -> FieldEnvironment {
        FieldEnvironment {
            has_left_wall,
            has_right_wall,
            has_upper_wall,
            has_lower_wall
        }
    }
}

pub struct Game {
    pub width: usize,
    pub height: usize,
    pub goal_position: Position,
    pub current_position: Option<Position>,
    pub visited_positions: Vec<Vec<State>>
}

impl Game {
    pub fn new(width: usize, height: usize, goal_position: Position) -> Game {
        Game {
            width,
            height,
            goal_position,
            current_position: None,
            visited_positions: vec![vec![State::Unvisited; height]; width]
        }
    }

    pub fn is_started(&self) -> bool {
        match self.current_position.as_ref() {
            None => false,
            Some(position) => true
        }
    }

    pub fn start(&mut self, position: &Position) -> Result<(), &'static str> {
        if self.is_started() {
            Err("Game already started")
        } else {
            self.current_position = Some(Position::new(position.x, position.y));
            Ok(())
        }
    }

    pub fn current_position(&self) -> Result<&Position, &'static str> {
        match self.current_position.as_ref() {
            None => {
                return Err("Game not started");
            },
            Some(position) => {
                Ok(position)
            }
        }
    }

    pub fn move_to(&mut self, direction: &Direction) -> Result<Direction, &'static str> {
        let current_position = self.current_position()?;

        let next_position = self.next_position_from_direction(current_position, direction)?;
        let Position{x, y} = next_position;

        // store direction that lead us here
        self.visited_positions[x][y] = State::from_direction(direction);
        self.current_position = Some(next_position);

        Ok(direction.clone())
    }

    pub fn move_backwards(&mut self) -> Result<Direction, &'static str> {
        let (x, y) = {
            let Position{x, y} = self.current_position()?;
            (*x, *y)
        };

        let direction = { Direction::inverted_direction_from_state( &self.visited_positions[x][y] ).expect("Inverted direction of an unvisited field") };
        self.visited_positions[x][y] = State::Visited;

        let current_position = self.current_position()?;
        self.current_position = Some( self.next_position_from_direction(current_position, &direction)? );

        Ok(direction)
    }

    pub fn get_next_unvisited_direction(&self, environment: FieldEnvironment) -> Result<Option<Direction>, &'static str>{
        let current_position = self.current_position()?;

        if let Ok(Position{x, y}) = self.next_position_from_direction(current_position, &Direction::Left) {
            if !environment.has_left_wall && self.visited_positions[x][y] == State::Unvisited {
                return Ok(Some(Direction::Left));
            }
        }

        if let Ok(Position{x, y}) = self.next_position_from_direction(current_position, &Direction::Right) {
            if !environment.has_right_wall && self.visited_positions[x][y] == State::Unvisited {
                return Ok(Some(Direction::Right));
            }
        }

        if let Ok(Position{x, y}) = self.next_position_from_direction(current_position, &Direction::Up) {
            if !environment.has_upper_wall && self.visited_positions[x][y] == State::Unvisited {
                return Ok(Some(Direction::Up));
            }
        }

        if let Ok(Position{x, y}) = self.next_position_from_direction(current_position, &Direction::Down) {
            if !environment.has_lower_wall && self.visited_positions[x][y] == State::Unvisited {
                return Ok(Some(Direction::Down));
            }
        }

        Ok(None)
    }

    fn next_position_from_direction(&self, position: &Position, direction: &Direction) -> Result<Position, &'static str> {
        match direction {
            Direction::Left => {
                if position.x == 0 { 
                    return Err("Left edge reached");
                }

                Ok(Position{ x: position.x-1, y: position.y })
            }
            Direction::Right => {
                if position.x >= self.width-1 { 
                    return Err("Right edge reached");
                }
                    
                Ok(Position{ x: position.x+1, y: position.y })
            }
            Direction::Up => {
                if position.y == 0 { 
                    return Err("Upper edge reached");
                }

                Ok(Position{ x: position.x, y: position.y-1 })
            }
            Direction::Down => {
                if position.y >= self.height-1 { 
                    return Err("Lower edge reached");
                }
                    
                Ok(Position{ x: position.x, y: position.y+1 })
            }
        }
    }
}