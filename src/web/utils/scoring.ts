import { PathConstraintSet, PathConstraintType } from '../utils/models';
import { mergeAllAnswerGroupConstraintSets, mergePathConstraintSets } from './constraintResolution';
import { Position, Tile } from './models';
import { findAllPaths } from './pathfinding';

export const letterFrequencies: Record<string, number> = {
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

function pointsForLetter(letter: string): number {
  if (letter === '*') return 0;
  return Math.floor(Math.log2(letterFrequencies['e'] / letterFrequencies[letter.toLowerCase()])) + 1;
}

export function getLetterPoints(): Record<string, number> {
  const points: Record<string, number> = { '*': 0 };
  for (const letter in letterFrequencies) {
    points[letter] = pointsForLetter(letter);
  }
  return points;
}

export function calculateWordScore(_word: string, path: Position[], board: Tile[][]): number {
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

export function scoreAnswerGroup(words: string[], board: Tile[][]): { scores: Record<string, number>, optimalConstraintSets: PathConstraintSet[] } {
  if (words.length === 0) {
    return { scores: {}, optimalConstraintSets: [] };
  }

  // Find all possible paths for each answer
  const answerObjects = [];
  for (const word of words) {
    const answer = findAllPaths(board, word);
    if (answer.paths.length === 0) {
      throw new Error(`Word '${word}' cannot be formed on this board`);
    }
    answerObjects.push(answer);
  }

  // Find all constraint sets that can satisfy all answers together
  const constraintSets = answerObjects.map(answer => answer.constraintsSet);
  
  let validConstraintSet;
  try {
    validConstraintSet = mergeAllAnswerGroupConstraintSets(constraintSets);
  } catch (error) {
    throw new Error("Answers cannot coexist due to conflicting wildcard constraints");
  }

  // For each valid path constraint set, calculate the maximum possible score
  let maxTotalScore = 0;
  let bestScoresByWord: Record<string, number> = {};
  let optimalConstraintSets: PathConstraintSet[] = [];
  
  for (const pathConstraint of validConstraintSet.pathConstraintSets) {
    let totalScore = 0;
    const scoresByWord: Record<string, number> = {};
    let pathConstraintsUsed: Record<string, boolean> = {};
    
    // For each answer, find the best scoring path that satisfies this constraint
    for (const answerObj of answerObjects) {
      let bestPathScore = 0;
      let bestPathConstraintsUsed: Record<string, boolean> = {};
      
      // Check all paths for this answer to find the one that works with current constraints
      for (const path of answerObj.paths) {
        // Check if this path's constraints are compatible with the current pathConstraint
        try {
          mergePathConstraintSets(path.constraints, pathConstraint);
          const pathScore = path.path.reduce((sum, pos) => sum + board[pos.row][pos.col].points, 0);
          
          // Track which wildcard positions are used by this path
          const pathConstraintsUsedThisPath: Record<string, boolean> = {};
          path.path.forEach((pos) => { 
            if (board[pos.row][pos.col].isWildcard) { 
              pathConstraintsUsedThisPath[`${pos.row}-${pos.col}`] = true;
            }
          });
          
          // Update best path if this one scores better
          if (pathScore > bestPathScore) {
            bestPathScore = pathScore;
            bestPathConstraintsUsed = pathConstraintsUsedThisPath;
          } else if (pathScore === bestPathScore) {
            for (const pos in pathConstraintsUsedThisPath) {
            bestPathConstraintsUsed[pos] = pathConstraintsUsedThisPath[pos];
            }
          }
        } catch (error) {
          // Merge failed, skip this path
        }
      }
      
      // Record this answer's best score and add to total
      scoresByWord[answerObj.word] = bestPathScore;
      totalScore += bestPathScore;
      
      // Accumulate which wildcard constraints are actually used by the best paths
      Object.keys(bestPathConstraintsUsed).forEach(key => {
        pathConstraintsUsed[key] = true;
      });
    }
    // Check if this constraint set actually uses the specific constrained wildcard tiles
    let constraintSetUsesAllConstrainedWildcards = false;
    
    // Find wildcard positions on the board to map them to first/second
    const wildcardPositions: Array<{row: number, col: number, isFirst: boolean}> = [];
    for (let row = 0; row < 4; row++) {
      for (let col = 0; col < 4; col++) {
        if (board[row][col].isWildcard) {
          wildcardPositions.push({ 
            row, 
            col, 
            isFirst: row < 2 && col < 2 // Backend logic for determining first wildcard
          });
        }
      }
    }
    
    const firstWildcardKey = wildcardPositions.find(w => w.isFirst) ? 
      `${wildcardPositions.find(w => w.isFirst)!.row}-${wildcardPositions.find(w => w.isFirst)!.col}` : null;
    const secondWildcardKey = wildcardPositions.find(w => !w.isFirst) ? 
      `${wildcardPositions.find(w => !w.isFirst)!.row}-${wildcardPositions.find(w => !w.isFirst)!.col}` : null;
    
    // Validate wildcard usage based on constraint type
    switch (pathConstraint.type) {
      case PathConstraintType.Unconstrained:
        constraintSetUsesAllConstrainedWildcards = true; // Always valid for unconstrained
        break;
      case PathConstraintType.FirstDecided:
        // Must use the first wildcard
        constraintSetUsesAllConstrainedWildcards = firstWildcardKey ? pathConstraintsUsed[firstWildcardKey] === true : false;
        break;
      case PathConstraintType.SecondDecided:
        // Must use the second wildcard
        constraintSetUsesAllConstrainedWildcards = secondWildcardKey ? pathConstraintsUsed[secondWildcardKey] === true : false;
        break;
      case PathConstraintType.BothDecided:
        // Must use both wildcards
        constraintSetUsesAllConstrainedWildcards = firstWildcardKey && secondWildcardKey ? 
          (pathConstraintsUsed[firstWildcardKey] === true && pathConstraintsUsed[secondWildcardKey] === true) : false;
        break;
    }
  
    
    // If this constraint set gives us a better total score, use it
    if (totalScore > maxTotalScore) {
      maxTotalScore = totalScore;
      bestScoresByWord = scoresByWord;
      // Only use this constraint set if it properly uses the required wildcards
      if (constraintSetUsesAllConstrainedWildcards) {
        optimalConstraintSets = [pathConstraint];
      } else {
        optimalConstraintSets = [];
      }
    } else if (totalScore === maxTotalScore) {
      // Only add to optimal constraint sets if it properly uses the required wildcards
      if (constraintSetUsesAllConstrainedWildcards) {
        optimalConstraintSets.push(pathConstraint);
      }
    }
  }

  return { scores: bestScoresByWord, optimalConstraintSets };
}
