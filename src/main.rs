#![allow(non_snake_case)]
use std::cell::Cell;
use core::fmt::Display;
use dioxus::prelude::*;
use rand::Rng;
use tracing::Level;
use tracing;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum CellContent {
    Mine,
    Empty(usize),
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum CellVisibility {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Debug, Clone, PartialEq, Copy)]
struct CellStatus{
    content: CellContent,
    status: CellVisibility
}

impl CellStatus {
    fn new() -> CellStatus {
        CellStatus {
            content: CellContent::Empty(0),
            status: CellVisibility::Hidden,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Playing,
    Won,
    Lost,
}


impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::Playing => write!(f, "Playing"),
            GameState::Won => write!(f, "Won"),
            GameState::Lost => write!(f, "Lost"),
        }
    
    }
}

impl  GameState {
    fn is_playing(&self) -> bool {
        *self == GameState::Playing
    }
}

struct Coordinate {
    x: usize,
    y: usize,
}
struct Game{
    pub width: usize,
    pub height: usize,
    pub field: Vec<Vec<CellStatus>>,
    pub state: GameState,
}

impl Game {
    fn new(width: usize, height: usize, mines: usize) -> Game {
        let mut game = Game {
            width: width,
            height: height,
            field: vec![vec![CellStatus::new(); width]; height],
            state: GameState::Playing
        };

        let mut rng = rand::thread_rng();

        let mut placed_bombs: usize = 0;

        loop {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            
            if let CellContent::Empty(_) = game.field[y][x].content {
                game.field[y][x] = CellStatus{content: CellContent::Mine, status: CellVisibility::Hidden};
                placed_bombs += 1;
            }
            if placed_bombs >= mines {
                break;
            }
        }

        for x in 0..width {
            for y in 0..height {
                let coord = Coordinate{x, y};
                if let Some(CellStatus{content: CellContent::Empty(_),status: _}) = game.get_cell(&coord){
                    let mut mine_count = 0;
                    game.get_neighbours(&coord).for_each(|n| {
                        if let Some(CellStatus{content: CellContent::Mine, status: _}) = game.get_cell(&n) {
                            mine_count += 1;
                        }
                    });

                    if let Some(CellStatus{content: CellContent::Empty(n), status: _}) = game.get_cell_mut(&coord) {
                        *n = mine_count;
                    }
                }
            }
        };

        game
    }

    fn get_cell(&self, coord: &Coordinate) -> Option<&CellStatus> {
        self.field.get(coord.y).and_then(|row| row.get(coord.x))
    }

    fn get_cell_mut(&mut self, coord: &Coordinate) -> Option<&mut CellStatus> {
        self.field.get_mut(coord.y).and_then(|row| row.get_mut(coord.x))
    }
    

    fn get_neighbours<'a>(&'a self, coord: &'a Coordinate) -> impl Iterator<Item = Coordinate> + '_ {
        (-1..=1).flat_map(
            move |dx: isize| (-1..=1).map(
                move |dy| (
                    coord.x as isize + dx,
                    coord.y as isize + dy,
                )
            )
        ).filter(move |(x, y)| {
                *x >= 0 && *x < (self.width as isize) && 
                *y >= 0 && *y < (self.height as isize)
            }
        ).map(move |(x, y)| Coordinate{x: x as usize, y: y as usize})
    }

    fn reveal_field_checked(&mut self, coord: Coordinate) {
        if ! self.state.is_playing() {return;}

        if let Some(cell) = self.get_cell_mut(&coord) {
            let cell_clone = cell.clone();
            if cell.status == CellVisibility::Hidden {
                cell.status = CellVisibility::Revealed;
                if let CellContent::Empty(0) = cell.content {
                    self.get_neighbours(&coord).collect::<Vec<_>>().into_iter().for_each(|n| {
                        self.reveal_field_checked(n);
                    });
                }
                if cell_clone.content == CellContent::Mine {
                    self.state = GameState::Lost;
                } else if self.is_fully_revealed_and_marked() {
                    self.state = GameState::Won;
                }
            }
        }
    }   

    fn toggle_flag_checked(&mut self, coor: Coordinate) {
        if ! self.state.is_playing() {return;}
        if let Some(cell) = self.get_cell_mut(&coor) {
            if cell.status == CellVisibility::Hidden {
                cell.status = CellVisibility::Flagged;
            } else if cell.status == CellVisibility::Flagged {
                cell.status = CellVisibility::Hidden;
            }
            if cell.content == CellContent::Mine && self.is_fully_revealed_and_marked() {
                self.state = GameState::Won;
            }
        }
        
    }

    fn is_lost(&mut self) -> bool {
        for x in 0..self.width {
            for y in 0..self.height {
                let coord = Coordinate{x, y};
                if let Some(CellStatus{content: CellContent::Mine, status: CellVisibility::Revealed}) = self.get_cell(&coord) {
                    return true;
                }
            }
        }
        return false;
    }

    fn is_fully_revealed_and_marked(&mut self) -> bool {
        for x in 0..self.width {
            for y in 0..self.height {
                let coord = Coordinate{x, y};
                if let Some(CellStatus{content: _, status: CellVisibility::Hidden}) = self.get_cell(&coord) {
                    return false;
                }
            }
        }
        return true;
    }


}



#[component]
fn App() -> Element {
    let mut game = use_signal(|| Game::new(10, 10, 10));

    rsx! {
        link{rel:"stylesheet", href: "main.css"}
        div{
            class: "thediv",
            class: if game.read().state == GameState::Playing {"playing"},
            class: if game.read().state == GameState::Lost || game.read().state == GameState::Won {"finished"},
            h1 {
                "Minesweeper"
            },
            p {
               match game.read().state {
                   GameState::Playing => "Playing",
                GameState::Won => "You won! 🏆",
                   GameState::Lost => "You lost! 😢",
               }
            },
            table {
                for (y, row) in game.read().field.iter().enumerate() {
                    tr{
                        class: "row",
                        for (x, cell) in row.iter().enumerate() {
                            td {
                                class: "cell",
                                class: if cell.status == CellVisibility::Hidden {"hidden"},
                                class: if cell.status == CellVisibility::Flagged {"flagged"},
                                class: if cell.status == CellVisibility::Revealed {"revealed"},
                                class: if cell.content == CellContent::Mine {"mine"},
                                class: if let CellContent::Empty(n) = cell.content {format!("empty-{}",n)},
                                prevent_default: "oncontextmenu",
                                onclick: move |e: Event<MouseData>| {
                                    println!("clicked cell {:?}", e);
                                    game.with_mut(
                                        |g| {
                                            g.reveal_field_checked(Coordinate{x, y});
                                        }
                                    );
                                },
                                oncontextmenu: move |e: Event<MouseData>| {
                                    println!("clicked cell {:?}", e);
                                    game.with_mut(
                                        |g| {
                                            g.toggle_flag_checked(Coordinate{x, y});
                                        }
                                    );
                                },
                                span{
                                    class: "inner-cell",
                                    if cell.status == CellVisibility::Hidden {
                                       if cell.content == CellContent::Mine && !game.read().state.is_playing() {
                                           "💣"
                                       } else {
                                           ""
                                       }
                                    } else if cell.status == CellVisibility::Flagged {
                                        "🚩"
                                    } else {
                                        match cell.content {
                                            CellContent::Mine => "💣".to_string(),
                                            CellContent::Empty(0) => " ".to_string(),
                                            CellContent::Empty(n) => n.to_string(),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div{
                style: "margin-top: 1em;",
                button {
                    onclick: move |e| {
                        game.with_mut(|g| {
                            *g = Game::new(9, 9, 10);
                        });
                    },
                    "Start a new easy game"
                }
                button {
                    onclick: move |e| {
                        game.with_mut(|g| {
                            *g = Game::new(16, 16, 40);
                        });
                    },
                    "Start a new medium game"
                    
                }
                button {
                    onclick: move |e| {
                        game.with_mut(|g| {
                            *g = Game::new(30, 16, 99);
                        });
                    },
                    "Start a new hard game"
                }
            }
        }
    }
    }
    