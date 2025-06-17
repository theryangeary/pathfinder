import { AnswerGroupConstraintSet, PathConstraintSet, PathConstraintType, Position, Tile } from './models';
import { getWildcardConstraintsFromPath } from './pathfinding';


export class UnsatisfiableConstraint extends Error {
  constructor() {
    super('Constraints cannot be satisfied');
  }
}

// Convert Record<string, string> constraints to PathConstraintSet
export function constraintsToPathConstraintSet(constraints: Record<string, string>, board: Tile[][]): PathConstraintSet {
  // Find wildcard positions
  const wildcardPositions: Array<{row: number, col: number, index: number}> = [];
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      if (board[row][col].isWildcard) {
        wildcardPositions.push({row, col, index: wildcardPositions.length});
      }
    }
  }
  
  if (wildcardPositions.length !== 2) {
    throw new Error('Expected exactly 2 wildcards');
  }
  
  const firstWildcard = wildcardPositions[0];
  const secondWildcard = wildcardPositions[1];
  
  const firstKey = `${firstWildcard.row}-${firstWildcard.col}`;
  const secondKey = `${secondWildcard.row}-${secondWildcard.col}`;
  
  const firstLetter = constraints[firstKey];
  const secondLetter = constraints[secondKey];
  
  if (firstLetter && secondLetter) {
    return {
      type: PathConstraintType.BothDecided,
      firstLetter,
      secondLetter
    };
  } else if (firstLetter) {
    return {
      type: PathConstraintType.FirstDecided,
      firstLetter
    };
  } else if (secondLetter) {
    return {
      type: PathConstraintType.SecondDecided,
      secondLetter
    };
  } else {
    return {
      type: PathConstraintType.Unconstrained
    };
  }
}

// Merge two PathConstraintSets
export function mergePathConstraintSets(a: PathConstraintSet, b: PathConstraintSet): PathConstraintSet {
  if (a.type === PathConstraintType.Unconstrained) {
    return b;
  }
  
  if (b.type === PathConstraintType.Unconstrained) {
    return a;
  }
  
  if (a.type === PathConstraintType.FirstDecided) {
    if (b.type === PathConstraintType.FirstDecided) {
      if (a.firstLetter === b.firstLetter) {
        return a;
      } else {
        throw new UnsatisfiableConstraint();
      }
    } else if (b.type === PathConstraintType.SecondDecided) {
      return {
        type: PathConstraintType.BothDecided,
        firstLetter: a.firstLetter,
        secondLetter: b.secondLetter
      };
    } else if (b.type === PathConstraintType.BothDecided) {
      if (a.firstLetter === b.firstLetter) {
        return b;
      } else {
        throw new UnsatisfiableConstraint();
      }
    }
  }
  
  if (a.type === PathConstraintType.SecondDecided) {
    if (b.type === PathConstraintType.FirstDecided) {
      return {
        type: PathConstraintType.BothDecided,
        firstLetter: b.firstLetter,
        secondLetter: a.secondLetter
      };
    } else if (b.type === PathConstraintType.SecondDecided) {
      if (a.secondLetter === b.secondLetter) {
        return a;
      } else {
        throw new UnsatisfiableConstraint();
      }
    } else if (b.type === PathConstraintType.BothDecided) {
      if (a.secondLetter === b.secondLetter) {
        return b;
      } else {
        throw new UnsatisfiableConstraint();
      }
    }
  }
  
  if (a.type === PathConstraintType.BothDecided) {
    if (b.type === PathConstraintType.FirstDecided) {
      if (a.firstLetter === b.firstLetter) {
        return a;
      } else {
        throw new UnsatisfiableConstraint();
      }
    } else if (b.type === PathConstraintType.SecondDecided) {
      if (a.secondLetter === b.secondLetter) {
        return a;
      } else {
        throw new UnsatisfiableConstraint();
      }
    } else if (b.type === PathConstraintType.BothDecided) {
      if (a.firstLetter === b.firstLetter && a.secondLetter === b.secondLetter) {
        return a;
      } else {
        throw new UnsatisfiableConstraint();
      }
    }
  }
  
  throw new UnsatisfiableConstraint();
}

// Intersection of two AnswerGroupConstraintSets
export function intersectAnswerGroupConstraintSets(a: AnswerGroupConstraintSet, b: AnswerGroupConstraintSet): AnswerGroupConstraintSet {
  const resultSets: PathConstraintSet[] = [];
  
  for (const aConstraint of a.pathConstraintSets) {
    for (const bConstraint of b.pathConstraintSets) {
      try {
        const merged = mergePathConstraintSets(aConstraint, bConstraint);
        resultSets.push(merged);
      } catch (e) {
        // Merge failed, skip this combination
      }
    }
  }
  
  if (resultSets.length === 0) {
    throw new UnsatisfiableConstraint();
  }
  
  return {
    pathConstraintSets: resultSets
  };
}

// Merge all AnswerGroupConstraintSets (equivalent to Rust merge_all)
export function mergeAllAnswerGroupConstraintSets(sets: AnswerGroupConstraintSet[]): AnswerGroupConstraintSet {
  let cumulativeConstraints: AnswerGroupConstraintSet | null = null;
  for (const set of sets) {
    if (cumulativeConstraints === null) {
      cumulativeConstraints = set;
    } else {
      cumulativeConstraints = intersectAnswerGroupConstraintSets(cumulativeConstraints, set);
    }
  }
  
  if (cumulativeConstraints === null) {
    throw new UnsatisfiableConstraint();
  }
  
  // Remove duplicates by converting to Set and back to array
  const uniqueConstraints = Array.from(
    new Set(cumulativeConstraints.pathConstraintSets.map(constraint => JSON.stringify(constraint)))
  ).map(constraintStr => JSON.parse(constraintStr) as PathConstraintSet);
  
  return {
    pathConstraintSets: uniqueConstraints
  };
}

// Convert paths and answers to AnswerGroupConstraintSets
export function getAnswerGroupConstraintSets(
  board: Tile[][],
  answers: string[],
  validAnswers: boolean[],
  validPaths: (Position[] | null)[]
): AnswerGroupConstraintSet[] {
  const constraintSets: AnswerGroupConstraintSet[] = [];
  
  for (let i = 0; i < answers.length; i++) {
    if (validAnswers[i] && validPaths[i] && answers[i]) {
      const constraints = getWildcardConstraintsFromPath(board, answers[i], validPaths[i]!);
      const pathConstraintSet = constraintsToPathConstraintSet(constraints, board);
      
      constraintSets.push({
        pathConstraintSets: [pathConstraintSet]
      });
    }
  }
  
  return constraintSets;
}

// Format constraint set for display
export function formatConstraintSet(constraintSet: AnswerGroupConstraintSet): string {
  if (constraintSet.pathConstraintSets.length === 0) {
    return 'No valid constraint sets';
  }
  
  const formattedSets = constraintSet.pathConstraintSets.map((pathSet, index) => {
    let description = '';
    
    switch (pathSet.type) {
      case PathConstraintType.Unconstrained:
        description = 'No wildcard constraints';
        break;
      case PathConstraintType.FirstDecided:
        description = `First wildcard = ${pathSet.firstLetter?.toUpperCase()}`;
        break;
      case PathConstraintType.SecondDecided:
        description = `Second wildcard = ${pathSet.secondLetter?.toUpperCase()}`;
        break;
      case PathConstraintType.BothDecided:
        description = `First wildcard = ${pathSet.firstLetter?.toUpperCase()}, Second wildcard = ${pathSet.secondLetter?.toUpperCase()}`;
        break;
    }
    
    return `Option ${index + 1}: ${description}`;
  });
  
  return formattedSets.join('\n');
}

// Helper function to convert AnswerGroupConstraintSet to Record<string, string> format
// The constraintSets parameter already contains only the optimal constraint sets that provide maximum score
export function convertConstraintSetsToConstraints(constraintSets: AnswerGroupConstraintSet, board: Tile[][]): Record<string, string> {
  const constraints: Record<string, string> = {};
  
  if (constraintSets.pathConstraintSets.length === 0) {
    return constraints;
  }

  // Check if any of the PathConstraintsSets is "Unconstrained" and if so, return empty constraints
  // this is because if there is a way to form the highest possible score set without using wildcards, that is the optimal use of the wildcards.
  if (constraintSets.pathConstraintSets.some(pathSet => pathSet.type === PathConstraintType.Unconstrained)) {
    return {};
  }
  
  // Find wildcard positions on the board
  const wildcardPositions: Array<{row: number, col: number, isFirst: boolean}> = [];
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      if (board[row][col]?.isWildcard) {
        wildcardPositions.push({ 
          row, 
          col, 
          isFirst: row < 2 && col < 2 // Backend logic for determining first wildcard
        });
      }
    }
  }
      
  // Extract all possible letter assignments for each wildcard from the optimal constraint sets
  // These constraint sets already represent only the maximum-scoring options
  let uniqueFirstLetters = [...new Set(constraintSets.pathConstraintSets.filter(constraint => constraint.type == PathConstraintType.FirstDecided || constraint.type == PathConstraintType.BothDecided).map(constraint => constraint.firstLetter?.toUpperCase()).filter(Boolean))];
  let uniqueSecondLetters = [...new Set(constraintSets.pathConstraintSets.filter(constraint => constraint.type == PathConstraintType.SecondDecided || constraint.type == PathConstraintType.BothDecided).map(constraint => constraint.secondLetter?.toUpperCase()).filter(Boolean))];
  
  if (constraintSets.pathConstraintSets.filter(constraint => constraint.type == PathConstraintType.Unconstrained || constraint.type == PathConstraintType.SecondDecided).length > 0) {
    uniqueFirstLetters.push("*");
  }
  if (constraintSets.pathConstraintSets.filter(constraint => constraint.type == PathConstraintType.Unconstrained || constraint.type == PathConstraintType.FirstDecided).length > 0) {
    uniqueSecondLetters.push("*");
  }
  
  // Convert unique letter sets to position-based constraints using slash notation for multiple options
  const firstWildcard = wildcardPositions.find(w => w.isFirst);
  const secondWildcard = wildcardPositions.find(w => !w.isFirst);
  
  if (firstWildcard && uniqueFirstLetters.length > 0) {
    constraints[`${firstWildcard.row}-${firstWildcard.col}`] = uniqueFirstLetters.join(' / ');
  }
  if (secondWildcard && uniqueSecondLetters.length > 0) {
    constraints[`${secondWildcard.row}-${secondWildcard.col}`] = uniqueSecondLetters.join(' / ');
  }
  console.log(constraints); 
  return constraints;
}
