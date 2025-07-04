This is a word game. The game is a blend between Boggle and Scrabble.

# Game rules

## Board

The board is a 4x4 grid of tiles. At each tile is a letter. Each letter has a point value. Python code to calculate the point value of each letter is as follows:
```
letter_frequencies={'a':0.078,
'b':0.02,
'c':0.04,
'd':0.038,
'e':0.11,
'f':0.014,
'g':0.03,
'h':0.023,
'i':0.086,
'j':0.0021,
'k':0.0097,
'l':0.053,
'm':0.027,
'n':0.072,
'o':0.061,
'p':0.028,
'q':0.0019,
'r':0.073,
's':0.087,
't':0.067,
'u':0.033,
'v':0.01,
'w':0.0091,
'x':0.0027,
'y':0.016,
'z':0.0044,
}

n=4
na=5#number of answers
max_word_length=n*n
import math

def points_for_letter(letter,letter_frequencies):
    return math.floor(math.log(letter_frequencies['e']/letter_frequencies[letter],2))+1

def points_for_letters(letter_frequencies):
    m={'*':0}
    for letter,frequency in letter_frequencies.items():
        m[letter]=points_for_letter(letter,letter_frequencies)
    return m
```

Two non-adjacent tiles which are each surrounded on all 4 sides by other tiles (not the edge of the board) are wildcard tiles. Wildcard tiles do not have a letter and have a point value of 0. Wildcard tiles can represent any letter (but still have 0 points) but they can only represent that same one letter across all answers given by the player.

## answers

The player will provide 5 answers. Each answer is a word which can be formed by following a path of tiles on the board, moving from one tile to a tile that is either directly adjacent or diagonally adjacent, and not repeating any tile within a word. An answer is valid if and only if it is a valid word according to the word list, there is a valid path on the board to spell that word, and any wildcard tiles required to form that path do not require the wildcard tile to hold a different letter from the letter required of the wildcard tile by any previous word.

Answer scores are the sum of the point values of each tile in the path for the word. Remember, wildcard tiles have a 0 point value.

### resolving duplicate paths

If there is more than one path to form a word on the board, ignore the rest after finding the path that satisfies these constraints in descending precedence (i.e. most important is first)

1. the path that has no wildcard tiles
2. the path that has one wildcard tile
3. the path that uses the fewest diagonal moves
4. the path that uses the latest diagonal move
5. the first path you come across that is valid

# Game interface

the game will exist as a webpage. the webpage will display the board as a grid of 4x4 tiles. the tiles will have subtly rounded edges. below the board will be 5 text boxes for inputing answers to. each answer box except the first will be disabled, until the answer box above it has a valid answer. each answer box will display a red X emoji to the left while it does not have a valid answer. each answer box will display a green checkmark emoji to the left while it does have a valid answer. each answer box will display the score for the word to the right. if an answer is not valid it has a score of 0.

as the user types in the answer box, the tiles that form a valid path will light up to demonstrate the path. The path highlighting follows constraint minimization principles:

1. **If ANY paths use 0 wildcards**: Light up ALL valid paths (including those with wildcards)
2. **If NO paths use 0 wildcards**: Light up only necessary wildcard paths that follow these rules:
   - Rule 2a: Don't use wildcards if non-wildcard alternatives exist for the same letter
   - Rule 2b: Consider both wildcards if both are accessible and can reach the next letter

### Wildcard Notation System

Wildcard tiles display different notations based on current constraints and typing context:

- **Single letter** (e.g., "E"): Wildcard must be that specific letter
- **Letter / \*** (e.g., "S / \*"): Wildcard could be that letter, but the other wildcard remains free for other uses  
- **Letter1 / Letter2** (e.g., "E / L"): Wildcard must be one of two letters, with paired constraints between wildcards
- **\***: No constraints on this wildcard tile

This system ensures maximum flexibility for future answers while clearly communicating current constraint relationships.

Be sure to be on the lookout for a later answer trying to overwrite the constraints imposed on a wildcard tile by a previous answer.

# references

check ../wordgame and ../word-game for a python cli version of the game and a partially implemented rust web implementation of the game. Feel free to ignore the rust version as it isn't going anywhere and wasm is hard and introduces some weird workarounds. with the python version keep in mind that it is a very minimal build that does not meet all the requirements here. of particular note is the fact that it requires the user to specify the values for the wildcard spots before giving answers, rather than intuiting them as the user gives answers.

# database

There is a database to store relevant application data. Any model that will go from frontend to backend or from backend to database should use protocol buffers. RPC calls between the frontend and backend should be defined using protocol buffers Services.

## database models

### Game table

A "game" table will store game boards. Each board is a "daily puzzle" similar to how there is a specific puzzle of the day in the NYT crossword and similar. The boards will be generated randomly, checked for some quality measures (more on this in "backend") and then stored for future retreival. the primary key should be an id, but this table should also be indexable by the date of that puzzle.

### GameEntry table

This table represents one user entry in a specific game. It must have a foreign key for the user, and a foreign key for the game. It will store each answer given, and that answer's score. It will also store the total score.

### User table

This table represents a user. It will have a user id and a way to connect a cookie stored in the browser to the user. When a user first loads the game, a cookie should be set to identify the browser the next time the same browser loads the game. There will not be an "account" in the typical sense where a user registers or has a username or password. We will simply do a best effort "this is the same browser, we assume this is the same user". Because no sensitive information about the user is ever stored, this approach is sufficient for our needs.

# backend

There is a backend application for the frontend to interface with the database. The frontend code will call it to retrieve the daily puzzle, retrieve a specific historical puzzle or a random historical puzzle (by historical I mean a daily puzzle from a past day), store the user's GameEntry for the current puzzle, and store a user's cookie for identifying the same user coming back on a future day. Any single historical game should be able to be retrieved based on either ID, date, or random selection. It will also expose endpoints for the frontend to acquire some aggregate data about a GameEntry's rank against all GameEntrys for a given puzzle (i.e. this score is in the top 10% of scores for this Game). The ranking calculation will be performed on demand for now.

## game generation

game generation should not be an RPC call, but instead run in a cron-like fashion, at backend startup as well as at midnight UTC each day, to generate games for any day between the current day and 3 days in the future that do not already exist in the database.

### generated game reproducibility

game generation should not be truly random, but instead be seeded by the date the game is to be the daily puzzle for combined with the attempt number, so that the generation can be reproduced if needed.

### generated game quality

the process of generating a game looks like this:
1. randomly (pseudorandomly with game date and attempt number as seed) generate a board
2.a. check that the sum of the 5 highest-scoring valid words that can be formed on the board meets or exceeds a threshold score. the threshold score should be configurable but for the time being set it to 40.
2.b. if the game does not pass the threshold score, repeat 1 and 2.a. up to 5 times until 2.a. is met.
2.c. if 2.a. is still not met after 2.b., decrease the threshold score by 25% (from 40 to 30). again retry up to 5 times until 2.a. is met with this new lower threshold score.
2.d. if 2.a. is still not met, cause an alert and log an error

# tech stack

## Backend Application
The backend will be implemented in **Rust** to leverage existing code from ../word-game and provide excellent performance for game logic operations. Key libraries:
- **Axum** for HTTP server and routing
- **Tonic** for gRPC and protocol buffer support
- **SQLx** for database operations (chosen for async-first design and database migration flexibility)
- **Tokio** for async runtime
- **Tokio-cron-scheduler** for scheduled game generation

## Database Migration Plan
**Phase 1: SQLite** (0-10,000 daily users)
- File-based database for simple deployment
- Sufficient for initial launch and development

**Phase 2: PostgreSQL** (10,000+ daily users)
- Better concurrency and performance
- JSON support for complex data types
- Full-text search capabilities
- Horizontal scaling options

**Migration Strategy:**
- Design schema to be PostgreSQL-compatible from the start
- Use SQLx for database-agnostic queries
- Test with both SQLite and PostgreSQL during development
- Plan migration scripts and data export/import procedures

## Protocol Buffers
All data models transferred between frontend/backend and backend/database will use protocol buffers for type safety and efficient serialization.
