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

as the user types in the answer box, the tiles that form a valid path will light up to demonstrate the path. use the first two steps of "resolving duplicate paths" section above to decide which paths not to show, and then show the rest. i.e. if the user has only typed the letter "h" so far, every "h" tile will light up. draw connections between the tiles indicating the paths taken to find each word.

if an answer requires a wildcard tile to have a particular value, display the value on the wildcard tile. If an answer requires either wildcard tile (but not one specific wildcard tile) to have some value, display it as "<letter> / *". If no answer requires a wildcard tile have a particular value, display "*".

Be sure to be on the lookout for a later answer trying to overwrite the constraints imposed on a wildcard tile by a previous answer.

# references

check ../wordgame and ../word-game for a python cli version of the game and a partially implemented rust web implementation of the game. Feel free to ignore the rust version as it isn't going anywhere and wasm is hard and introduces some weird workarounds. with the python version keep in mind that it is a very minimal build that does not meet all the requirements here. of particular note is the fact that it requires the user to specify the values for the wildcard spots before giving answers, rather than intuiting them as the user gives answers.

# tech stack

this is up in the air. I'm open to vanilla javascript components and have a bit of a soft spot for them, however if there is a meaningful simplification to be had from the added overhead of some javascript frameworks, I'm open to it. please evaluate the options and present what you think is best
