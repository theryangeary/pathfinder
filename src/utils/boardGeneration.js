const letters = 'abcdefghijklmnopqrstuvwxyz';

function getRandomLetter() {
  return letters[Math.floor(Math.random() * letters.length)];
}

function findWildcardPositions() {
  // Only two non-adjacent positions exist in the 2x2 interior: diagonally opposite corners
  const nonAdjacentPairs = [
    [{ row: 1, col: 1 }, { row: 2, col: 2 }], // top-left and bottom-right
    [{ row: 1, col: 2 }, { row: 2, col: 1 }]  // top-right and bottom-left
  ];
  
  const randomPair = Math.floor(Math.random() * nonAdjacentPairs.length);
  return nonAdjacentPairs[randomPair];
}

export function generateBoard() {
  const board = Array(4).fill(null).map(() => Array(4).fill(null));
  const wildcardPositions = findWildcardPositions();
  
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      const isWildcard = wildcardPositions.some(pos => pos.row === row && pos.col === col);
      
      board[row][col] = {
        letter: isWildcard ? '*' : getRandomLetter(),
        isWildcard,
        row,
        col
      };
    }
  }
  
  return board;
}