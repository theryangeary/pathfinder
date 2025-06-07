import { letterFrequencies, Tile } from './scoring';

const letters = 'abcdefghijklmnopqrstuvwxyz';

// Interpolation constant: 0 = pure random, 1 = pure frequency-based
const FREQUENCY_INTERPOLATION = 0.5;

function getRandomLetter(): string {
  // Interpolate between uniform (1/26) and actual frequencies
  const uniformFreq = 1 / 26;
  const cumulativeFreqs = [];
  let total = 0;
  
  for (const letter of letters) {
    const actualFreq = letterFrequencies[letter];
    const interpolatedFreq = uniformFreq * (1 - FREQUENCY_INTERPOLATION) + actualFreq * FREQUENCY_INTERPOLATION;
    total += interpolatedFreq;
    cumulativeFreqs.push({ letter, cumulative: total });
  }
  
  const random = Math.random() * total;
  
  for (const entry of cumulativeFreqs) {
    if (random <= entry.cumulative) {
      return entry.letter;
    }
  }
  
  return 'e'; // fallback to most common letter
}

function findWildcardPositions(): Array<{ row: number; col: number }> {
  // Only two non-adjacent positions exist in the 2x2 interior: diagonally opposite corners
  const nonAdjacentPairs = [
    [{ row: 1, col: 1 }, { row: 2, col: 2 }], // top-left and bottom-right
    [{ row: 1, col: 2 }, { row: 2, col: 1 }]  // top-right and bottom-left
  ];
  
  const randomPair = Math.floor(Math.random() * nonAdjacentPairs.length);
  return nonAdjacentPairs[randomPair];
}

export function generateBoard(): Tile[][] {
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