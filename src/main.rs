use chess::{Board, ChessMove, Piece, BitBoard, Square, EMPTY, Color, ALL_SQUARES, MoveGen};

use raster::error::RasterError;
use raster::{Image, BlendMode, PositionMode, editor};

use regex::Regex;

use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::*;

struct Game {
    board: Board,
    user_id_white: UserId,
    user_id_black: UserId,
    offered_draw: Option<UserId>,
    whose_turn: chess::Color
}

struct MoveRequest {
    piece_type: Piece,
    square_from: Option<Square>,
    square_to: Square,
    promotion: Option<Piece>
}

fn requested_move(s: String, game: &Game) -> Result<MoveRequest, &'static str> {
    // Handle special cases of notation first
    if s == "O-O" {
        // Castle short (kingside)
        if game.whose_turn == Color::Black {
            Ok(MoveRequest {
                square_from: Some(Square::E8),
                square_to: Square::G8,
                piece_type: Piece::King,
                promotion: None
            })
        }
        else {
            Ok(MoveRequest {
                square_from: Some(Square::E1),
                square_to: Square::G1,
                piece_type: Piece::King,
                promotion: None
            })
        }
    }
    else if s == "O-O-O" {
        // Castle long (queenside)
        if game.whose_turn == Color::Black {
            Ok(MoveRequest {
                square_from: Some(Square::E8),
                square_to: Square::C8,
                piece_type: Piece::King,
                promotion: None
            })
        }
        else {
            Ok(MoveRequest {
                square_from: Some(Square::E1),
                square_to: Square::C1,
                piece_type: Piece::King,
                promotion: None
            })
        }
    }
    else {
        // I am a fool for trying to do this without regular expressions.
        let re = Regex::new(r"([a-h]?[1-8]?)([NKQBR]?)x?([a-h][1-8])[+#]?").unwrap();
        let mut promotion: Option<Piece> = None;

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

            if piece_type == Piece::Pawn && [0, 7].contains(&square_to.get_rank().to_index()) {
                // promotion!
                if s.contains("=")
                {
                    let promotion_re = Regex::new(r"=([QRNB])").unwrap();
                    if let Some(promotion_capture) = promotion_re.captures(&s) {
                        let (_full, [promotion_type]) = promotion_capture.extract();
                        promotion = Some(match promotion_type {
                            "Q" => Piece::Queen,
                            "R" => Piece::Rook,
                            "N" => Piece::Knight,
                            "B" => Piece::Bishop,
                            _ => panic!("Unexpected item in matching area")
                        });
                    }
                } 
                else {
                    return Err("Please denote the piece you wish to promote to with an equals sign, e.g. `e8=Q`.");
                }
            }

            Ok(MoveRequest {
                square_from,
                square_to,
                piece_type,
                promotion
            })
        } else {
            Err("Unable to parse move string.")
        }
    }
}

// Lord, forgive me.
static mut GAMES: Vec<Game> = Vec::new();

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    println!("ABOUT");
    msg.reply(&ctx.http, "Lets you play chess in an inconvenient way.").await?;

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    println!("PING");
    msg.reply(&ctx.http, "pong!").await?;

    Ok(())
}

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    unsafe {
        // First, check to see if this user is already in a game.
        if GAMES.iter().any(|game| game.user_id_black == msg.author.id || game.user_id_white == msg.author.id) {
            msg.reply(&ctx.http, "You're already in a game with someone else. Use `!resign` or `!quit` before starting a new game.").await?;
            return Ok(());
        }
    }

    // Any start message must have a mentioned user.

    let user_id_requester = msg.author.id;

    if msg.mentions.len() != 1 || msg.mentions.as_slice()[0].id == user_id_requester || msg.mentions.as_slice()[0].bot
    {
        msg.reply(&ctx.http, "Start commands must mention exactly one other user that isn't yourself or a bot.").await?;
        return Ok(());
    }

    let user_id_opponent = msg.mentions.as_slice()[0].id;

    unsafe {
        // Check to see if that user is already in a game.
        if GAMES.iter().any(|game| game.user_id_black == user_id_opponent || game.user_id_white == user_id_opponent) {
            msg.reply(&ctx.http, "That user is already in a game.").await?;
            return Ok(());
        }
    }

    let wants_black = msg.content.contains("as black");

    unsafe {
        GAMES.push(Game { 
            board: Board::default(), 
            user_id_white: if wants_black {user_id_opponent} else {user_id_requester}, 
            user_id_black:  if wants_black {user_id_requester} else {user_id_opponent},
            offered_draw: None,
            whose_turn: Color::White }
        );
    }

    msg.reply(&ctx.http, "Game started!").await?;

    Ok(())
}

fn readd_game(game: Game) {
    unsafe {
        GAMES.push(game);
    }
}

// Can't use a HashMap here, since it has a non-const new() fn
// Guess it's O(n) for us, boys.
static mut PIECE_IMGS: Vec<(Piece, Color, Image)> = Vec::new();

fn init_imgs() {
    unsafe {
        PIECE_IMGS.push((Piece::Pawn,   Color::White, raster::open("res/pawn_white.png").unwrap()));
        PIECE_IMGS.push((Piece::Rook,   Color::White, raster::open("res/rook_white.png").unwrap()));
        PIECE_IMGS.push((Piece::Knight, Color::White, raster::open("res/knight_white.png").unwrap()));
        PIECE_IMGS.push((Piece::Bishop, Color::White, raster::open("res/bishop_white.png").unwrap()));
        PIECE_IMGS.push((Piece::King,   Color::White, raster::open("res/king_white.png").unwrap()));
        PIECE_IMGS.push((Piece::Queen,  Color::White, raster::open("res/queen_white.png").unwrap()));

        PIECE_IMGS.push((Piece::Pawn,   Color::Black, raster::open("res/pawn_black.png").unwrap()));
        PIECE_IMGS.push((Piece::Rook,   Color::Black, raster::open("res/rook_black.png").unwrap()));
        PIECE_IMGS.push((Piece::Knight, Color::Black, raster::open("res/knight_black.png").unwrap()));
        PIECE_IMGS.push((Piece::Bishop, Color::Black, raster::open("res/bishop_black.png").unwrap()));
        PIECE_IMGS.push((Piece::King,   Color::Black, raster::open("res/king_black.png").unwrap()));
        PIECE_IMGS.push((Piece::Queen,  Color::Black, raster::open("res/queen_black.png").unwrap()));
    }
}

fn piece_img(color: Color, piece: Piece) -> &'static Image {
    unsafe {
        &PIECE_IMGS.iter().find(|p| p.0 == piece && p.1 == color).unwrap().2
    }
}

fn render_board(game: &Game) -> Result<(), RasterError> {
    let mut res = raster::open("res/chessboard.png").unwrap();
    let square_size = 60;

    for square in ALL_SQUARES {
        if let Some(piece) = game.board.piece_on(square) {
            let color = game.board.color_on(square).unwrap();
            // Render it!
            let img = piece_img(color, piece);

            res = editor::blend(&res, img, 
                BlendMode::Normal, 1.0, 
                PositionMode::TopLeft,
                ((square.get_file().to_index()) * square_size) as i32,
                ((7 - square.get_rank().to_index()) * square_size) as i32)?;
        }
    }

    raster::save(&res, "res/out.png")?;

    Ok(())
}

fn pop_game(who: UserId) -> Option<Game> {
    unsafe {
        if let Some(pos) = GAMES.iter().position(|g| g.user_id_black == who || g.user_id_white == who) {
            Some(GAMES.remove(pos))
        } else {
            None
        }
    }
}

#[command]
async fn draw(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(mut game) = pop_game(msg.author.id) {
        if game.offered_draw.is_none() {
            game.offered_draw = Some(msg.author.id);
            msg.channel_id.send_message(&ctx.http, |m| {
                m.content("Your opponent has offered a draw. You can use `!draw` to accept, or you may keep playing.");
                m
            }).await?;
            readd_game(game);
        } else {
            if game.offered_draw.unwrap() != msg.author.id {
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.content("Draw accepted.");
                    m
                }).await?;
            } 
            else {
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.content("You've already offered a draw.");
                    m
                }).await?;
                readd_game(game);
            }
        }
    } else {
        msg.reply(&ctx.http, "I don't see a game you're in. You can start a new one with `!start with <@user> [as black|white]`").await?;
    }

    Ok(())
}

#[command]
async fn resign(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(_) = pop_game(msg.author.id) {
        msg.reply(&ctx.http, "Resigned.").await?;
    } else {
        msg.reply(&ctx.http, "I don't see a game you're in. You can start a new one with `!start with <@user> [as black|white]`").await?;
    }

    Ok(())
}

#[command]
async fn movepiece(ctx: &Context, msg: &Message) -> CommandResult {
    // Get the relevant game.
    let mut game: Game;

    if let Some(mut game) = pop_game(msg.author.id) {
        if (game.whose_turn == Color::Black && game.user_id_white == msg.author.id) || (game.whose_turn == Color::White && game.user_id_black == msg.author.id) {
            msg.reply(&ctx.http, "It's not your turn.").await?;
            readd_game(game);
            return Ok(());
        }

        // Clear any draw offer when a move is made.
        if game.offered_draw.is_some() {
            game.offered_draw = None;
        }
    
        // slice off "!move "
        let movestr = &msg.content[6..];
    
        match requested_move(movestr.to_string(), &game) {
            Ok(move_requested) => {
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
                        msg.reply(&ctx.http, "You have no pieces of that type.").await?;
                        readd_game(game);
                        return Ok(());
                    }
    
                    // Of the remaining pieces, can any move to the specified square?
                    let valid_source_squares = source_squares.iter()
                                                            .map(|&sq| (game.board.legal(ChessMove::new(sq, move_requested.square_to, None)), sq))
                                                            .filter(|(valid, _)| *valid)
                                                            .collect::<Vec<(bool, Square)>>();
                
                    if valid_source_squares.len() > 1 {
                        msg.reply(&ctx.http, "More than one legal move is implied by that notation -- prepend the piece name with the square of the piece you want to move").await?;
                        readd_game(game);
                        return Ok(());
                    }
                    else if valid_source_squares.len() == 0 {
                        msg.reply(&ctx.http, "Given movestring is not a legal move").await?;
                        readd_game(game);
                        return Ok(());
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
    
                render_board(&game).expect("oopsie");
    
                let f = [(&tokio::fs::File::open("res/out.png").await?, "out.png")];
    
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.reference_message(msg);
                    m.files(f);
                    m
                }).await?;
    
                // We've successfully moved. Is the game over now?
                if MoveGen::new_legal(&game.board).len() == 0 {
                    msg.channel_id.send_message(&ctx.http, |m| {
                        m.content("Game over!");
                        m
                    }).await?;
                }
                else {
                    readd_game(game);
                }
    
    
                return Ok(());
            },
            Err(reason) => {
                msg.reply(&ctx.http, format!("Improperly formatted move string: {}", reason)).await?;
            }
        }
    }
    else {
        msg.reply(&ctx.http, "I don't see a game you're in. You can start a new one with `!start with <@user> [as black|white]`").await?;
    }


    Ok(())
}

struct DiscordHandler;
impl EventHandler for DiscordHandler {}

#[group]
#[commands(about, ping, start, draw, resign, movepiece)]
struct General;

#[tokio::main]
async fn main() {
    // let mut game = Game {
    //     board: Board::default(),
    //     user_id_white: String::new(),
    //     user_id_black: String::new(),
    //     whose_turn: chess::Color::White
    // };

    // println!("{}", game.board);
    // println!("Making move e4");
    // match move_piece(&mut game, "e4".to_string()) {
    //     Ok(())      => println!("{}", game.board),
    //     Err(reason) => println!("Got error: {}", reason)
    // };

    init_imgs();

    let token = std::env::var("DISCORD_TOKEN").expect("Need DISCORD_TOKEN to be defined in the environment");

    let framework = StandardFramework::new().configure(|c| c.prefix("!")).group(&GENERAL_GROUP);
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
                            .event_handler(DiscordHandler)
                            .framework(framework)
                            .await
                            .expect("Error creating client");

    if let Err(reason) = client.start().await {
        println!("Client err: {:?}", reason);
    }
}
