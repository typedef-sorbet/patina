use chess::{Board, ChessMove, Piece, BitBoard, Square, EMPTY, Color};
use regex::Regex;

struct Game {
    board: Board,
    user_id_white: String,
    user_id_black: String,
    whose_turn: chess::Color
}

struct MoveRequest {
    piece_type: Piece,
    square_from: Option<Square>,
    square_to: Square,
    promotion: Option<Piece>
}

impl TryFrom<String> for MoveRequest {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        // I am a fool for trying to do this without regular expressions.
        let re = Regex::new(r"([a-h]?[1-8]?)([NKQBR]?)x?([a-h][1-8])[+#]?").unwrap();

        if let Some(captures) = re.captures(&s) {
            let (_full, [square_from_str, piece_type_str, square_to_str]) = captures.extract();

            let square_from = if !square_from_str.is_empty() {Some(square_from_str.parse::<Square>().unwrap())} else {None};
            let square_to = square_to_str.parse::<Square>().unwrap();
            let piece_type = if !piece_type_str.is_empty() {
                match piece_type_str.bytes().next().unwrap() as char {
                    'K' =>  Piece::King,
                    'Q' =>  Piece::Queen,
                    'N' =>  Piece::Knight,
                    'B' =>  Piece::Bishop,
                    'R' =>  Piece::Rook,
                    _ =>    Piece::Pawn
                }
            } else {
                Piece::Pawn
            };

            // TODO handle promotion syntax
            Ok(Self {
                square_from,
                square_to,
                piece_type,
                promotion: None
            })
        } else {
            Err("Regex match failed")
        }
    }
}

fn main() {
    let mut game = Game {
        board: Board::default(),
        user_id_white: String::new(),
        user_id_black: String::new(),
        whose_turn: chess::Color::White
    };

    println!("{}", game.board);
    println!("Making move e4");
    match move_piece(&mut game, "e4".to_string()) {
        Ok(())      => println!("{}", game.board),
        Err(reason) => println!("Got error: {}", reason)
    };
}

fn move_piece(game: &mut Game, movestr: String) -> Result<(), &str> {
    // Get the piece implicated by movestr, and look for those respective pieces
    if let Ok(move_requested) = MoveRequest::try_from(movestr) {
        let mut relevant_pieces: BitBoard = game.board.pieces(move_requested.piece_type).clone();
        let mut source_squares: Vec<Square> = Vec::new();
        let source_square: Square;

        if move_requested.square_from.is_none()
        {
            while relevant_pieces != EMPTY {
                let square = relevant_pieces.to_square();
                // xor-equals to clear this bit from the bitmap
                relevant_pieces ^= EMPTY | BitBoard::from_square(square);
                // Filter any pieces whose turn it isn't.
                // NOTE: This unwrap() should be safe, since we're only looking at squares where a piece exists
                if game.board.color_on(square).unwrap() == game.whose_turn {
                    source_squares.push(square);
                }
            }
        
            // Are there any pieces left?
            if source_squares.len() == 0 {
                return Err("You have no pieces of that type.");
            }

            // Of the remaining pieces, can any move to the specified square?
            let valid_source_squares = source_squares.iter()
                                                    .map(|&sq| (game.board.legal(ChessMove::new(sq, move_requested.square_to, None)), sq))
                                                    .filter(|(valid, _)| *valid)
                                                    .collect::<Vec<(bool, Square)>>();
        
            if valid_source_squares.len() > 1 {
                return Err("More than one legal move is implied by that notation -- prepend the piece name with the square of the piece you want to move")
            }
            else if valid_source_squares.len() == 0 {
                return Err("Given movestring is not a legal move");
            }
            else {
                source_square = valid_source_squares.as_slice()[0].1;
            }
        }
        else {
            source_square = move_requested.square_from.unwrap();
        }

        // We now have a known good source and target square. Let's make a move!
        // TODO handle promotion
        game.board = game.board.make_move_new(ChessMove::new(source_square, move_requested.square_to, None));
        game.whose_turn = match game.whose_turn {
            Color::White => Color::Black,
            Color::Black => Color::White
        };

        Ok(())
    }
    else {
        Err("Improperly formatted move string")
    }
}
