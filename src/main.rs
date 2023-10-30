use chess::{Board, MoveGen, BoardStatus, ChessMove, Piece, BitBoard, Square, EMPTY};
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
    square_to: Square
}

impl TryFrom<String> for MoveRequest {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        // I am a fool for trying to do this without regular expressions.
        let re = Regex::new(r"(?:<square_from>[a-h][1-8])?(?:<piece_type>[NKQBR])?x?(?:<square_to>[a-h][1-8])[+#]?").unwrap();

        if let Some(captures) = re.captures(&s) {
            let square_from_opt = captures.name("square_from");
            let square_to_opt = captures.name("square_to");
            let piece_type_opt = captures.name("piece_type");

            let square_from = if square_from_opt.is_some() {Some(square_from_opt.unwrap().as_str().parse::<Square>().unwrap())} else {None};
            let square_to = square_to_opt.unwrap().as_str().parse::<Square>().unwrap();
            let piece_type = match piece_type_opt.unwrap().as_str().bytes().next().unwrap() as char {
                'K' =>  Piece::King,
                'Q' =>  Piece::Queen,
                'N' =>  Piece::Knight,
                'B' =>  Piece::Bishop,
                'R' =>  Piece::Rook,
                _ =>    Piece::Pawn
            };

            Ok(Self {
                square_from,
                square_to,
                piece_type
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
}

fn move_piece(game: &mut Game, movestr: String) -> Result<(), &str> {
    // Get the piece implicated by movestr, and look for those respective pieces
    if let Ok(move_requested) = MoveRequest::try_from(movestr) {
        let mut relevant_pieces: BitBoard = game.board.pieces(move_requested.piece_type).clone();
        let mut source_squares: Vec<Square> = Vec::new();
    
        while relevant_pieces != EMPTY {
            let square = relevant_pieces.to_square();
            relevant_pieces ^= EMPTY | BitBoard::from_square(square);
            // Filter any pieces whose turn it isn't.
            if game.board.color_on(square).unwrap() == game.whose_turn {
                source_squares.push(square);
            }
        }
    
        // Are there any pieces left?
        if source_squares.len() < 0 {
            return Err("You have no pieces of that type.");
        }
    
        // Of the remaining pieces, can any move to the specified square?
        let valid_source_squares = source_squares.iter()
                                                .map(|&sq| (game.board.legal(ChessMove::new(sq, target_square, None)), ChessMove::new(sq, target_square, None)))
                                                .filter(|(valid, _)| *valid)
                                                .collect::<Vec<(bool, ChessMove)>>();
    
        if valid_source_squares.len() > 1 {
            return Err("More than one move is implied by that notation -- prepend the piece name with the square of the piece you want to move")
        }
    }


    Ok(())
}
