## Commands

- `move (default)`
    - Syntax: `move <notation>` OR `<notation>` where `notation` is a *valid, unambiguous, and legal move* in algebraic chess [notation](https://en.wikipedia.org/wiki/Algebraic_notation_(chess))
    - Moves a piece, and passes play to the other player. Errors if the player is not in a game, or if the move is invalid, illegal, or could refer to more than one legal move.
- `show`
    - Syntax: `show <space>`
    - If a piece exists at `space`, renders all legal moves for the piece at `space` on the board. Errors if the player is not in a game.
- `start`
    - Syntax: `start with <user> [as white|black]`
    - Attempts to start a new game with the user, optionally as the color provided. Errors if either player is already in a game.
- `draw`
    - No arguments. Offers a draw. Errors if the player is not in a game.
- `accept`
    - No arguments. Accepts a draw offer. Errors if the player is not in a game.
- `resign` or `quit`
    - Resign from the current game. Errors if the player is not in a game.
