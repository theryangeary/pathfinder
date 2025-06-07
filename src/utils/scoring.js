export const letterFrequencies = {
  'a': 0.078,
  'b': 0.02,
  'c': 0.04,
  'd': 0.038,
  'e': 0.11,
  'f': 0.014,
  'g': 0.03,
  'h': 0.023,
  'i': 0.086,
  'j': 0.0021,
  'k': 0.0097,
  'l': 0.053,
  'm': 0.027,
  'n': 0.072,
  'o': 0.061,
  'p': 0.028,
  'q': 0.0019,
  'r': 0.073,
  's': 0.087,
  't': 0.067,
  'u': 0.033,
  'v': 0.01,
  'w': 0.0091,
  'x': 0.0027,
  'y': 0.016,
  'z': 0.0044,
};

function pointsForLetter(letter) {
  if (letter === '*') return 0;
  return Math.floor(Math.log2(letterFrequencies['e'] / letterFrequencies[letter.toLowerCase()])) + 1;
}

export function getLetterPoints() {
  const points = { '*': 0 };
  for (const letter in letterFrequencies) {
    points[letter] = pointsForLetter(letter);
  }
  return points;
}

export function calculateWordScore(word, path, board) {
  let score = 0;
  for (let i = 0; i < path.length; i++) {
    const { row, col } = path[i];
    const tile = board[row][col];
    if (tile.isWildcard) {
      score += 0;
    } else {
      score += pointsForLetter(tile.letter);
    }
  }
  return score;
}