## Commands

- `movepiece`
    - Syntax: `movepiece <notation>` where `notation` is a *valid, unambiguous, and legal move* in [algebraic chess notation](https://en.wikipedia.org/wiki/Algebraic_notation_(chess))
    - Moves a piece, and passes play to the other player. Errors if the player is not in a game, it isn't their turn,x or if the move is invalid, illegal, or could refer to more than one legal move.
- `start`
    - Syntax: `start with <user> [as white|black]`
    - Attempts to start a new game with the user, optionally as the color provided. Errors if either player is already in a game.
- `draw`
    - No arguments. Offers a draw, or accepts a draw offer. Errors if the player is not in a game.
- `resign`
    - Resign from the current game. Errors if the player is not in a game.
