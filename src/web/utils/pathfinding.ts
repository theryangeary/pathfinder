import { AnswerGroupConstraintSet, PathConstraintSet, PathConstraintType, Position, Tile } from './models';

interface PathScore {
  wildcardCount: number;
}
export interface PathWithConstraints {
  path: Position[];
  constraints: PathConstraintSet;
}

export interface Answer {
  word: string;
  paths: PathWithConstraints[];
  constraintsSet: AnswerGroupConstraintSet;
}

function isAdjacent(pos1: Position, pos2: Position): boolean {
  const rowDiff = Math.abs(pos1.row - pos2.row);
  const colDiff = Math.abs(pos1.col - pos2.col);
  return (rowDiff <= 1 && colDiff <= 1) && !(rowDiff === 0 && colDiff === 0);
}

export function getWildcardPositions(board: Tile[][]): Position[] {
  const wildcardPositions: Position[] = [];
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      if (board[row][col].isWildcard) {
        wildcardPositions.push({ row, col });
      }
    }
  }
  return wildcardPositions;
}

function isFirstWildcard(pos: Position): boolean {
  // Following backend logic: first wildcard is at position where row < 2 && col < 2
  return pos.row < 2 && pos.col < 2;
}

// @ts-ignore
function isSecondWildcard(pos: Position): boolean {
  // Following backend logic: second wildcard is any wildcard that's not the first
  return !isFirstWildcard(pos);
}

function createConstraintFromWildcard(pos: Position, char: string): PathConstraintSet {
  if (isFirstWildcard(pos)) {
    return {
      type: PathConstraintType.FirstDecided,
      firstLetter: char
    };
  } else {
    return {
      type: PathConstraintType.SecondDecided,
      secondLetter: char
    };
  }
}

function mergeConstraints(constraint1: PathConstraintSet, constraint2: PathConstraintSet): PathConstraintSet | null {
  // Handle Unconstrained cases
  if (constraint1.type === PathConstraintType.Unconstrained) {
    return constraint2;
  }
  if (constraint2.type === PathConstraintType.Unconstrained) {
    return constraint1;
  }

  // Handle FirstDecided cases
  if (constraint1.type === PathConstraintType.FirstDecided) {
    if (constraint2.type === PathConstraintType.FirstDecided) {
      if (constraint1.firstLetter === constraint2.firstLetter) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.SecondDecided) {
      return {
        type: PathConstraintType.BothDecided,
        firstLetter: constraint1.firstLetter,
        secondLetter: constraint2.secondLetter
      };
    } else if (constraint2.type === PathConstraintType.BothDecided) {
      if (constraint1.firstLetter === constraint2.firstLetter) {
        return constraint2;
      } else {
        return null; // Unsatisfiable
      }
    }
  }

  // Handle SecondDecided cases
  if (constraint1.type === PathConstraintType.SecondDecided) {
    if (constraint2.type === PathConstraintType.FirstDecided) {
      return {
        type: PathConstraintType.BothDecided,
        firstLetter: constraint2.firstLetter,
        secondLetter: constraint1.secondLetter
      };
    } else if (constraint2.type === PathConstraintType.SecondDecided) {
      if (constraint1.secondLetter === constraint2.secondLetter) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.BothDecided) {
      if (constraint1.secondLetter === constraint2.secondLetter) {
        return constraint2;
      } else {
        return null; // Unsatisfiable
      }
    }
  }

  // Handle BothDecided cases
  if (constraint1.type === PathConstraintType.BothDecided) {
    if (constraint2.type === PathConstraintType.FirstDecided) {
      if (constraint1.firstLetter === constraint2.firstLetter) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.SecondDecided) {
      if (constraint1.secondLetter === constraint2.secondLetter) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.BothDecided) {
      if (constraint1.firstLetter === constraint2.firstLetter &&
        constraint1.secondLetter === constraint2.secondLetter) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    }
  }

  return null; // Fallback
}

function findPathsForWordFromPosition(
  board: Tile[][],
  word: string,
  startRow: number,
  startCol: number,
  visited: Set<string>
): PathWithConstraints[] {
  const result: PathWithConstraints[] = [];

  if (word.length === 0 || visited.has(`${startRow}-${startCol}`)) {
    return result;
  }

  const currentChar = word[0].toLowerCase();
  const tile = board[startRow][startCol];

  // Check if current tile can represent the current character
  if (!tile.isWildcard && tile.letter.toLowerCase() !== currentChar) {
    return result;
  }

  if (word.length === 1) {
    // Base case: word is complete
    const path = [{ row: startRow, col: startCol }];
    const constraints = tile.isWildcard
      ? createConstraintFromWildcard({ row: startRow, col: startCol }, currentChar)
      : { type: PathConstraintType.Unconstrained };

    result.push({ path, constraints });
    return result;
  }

  // Recursive case: continue building path
  visited.add(`${startRow}-${startCol}`);

  const directions = [
    [-1, -1], [-1, 0], [-1, 1],
    [0, -1], [0, 1],
    [1, -1], [1, 0], [1, 1]
  ];

  for (const [deltaRow, deltaCol] of directions) {
    const nextRow = startRow + deltaRow;
    const nextCol = startCol + deltaCol;

    if (nextRow >= 0 && nextRow < 4 && nextCol >= 0 && nextCol < 4) {
      const nextPaths = findPathsForWordFromPosition(
        board,
        word.slice(1),
        nextRow,
        nextCol,
        visited
      );

      for (const nextPath of nextPaths) {
        const currentConstraints = tile.isWildcard
          ? createConstraintFromWildcard({ row: startRow, col: startCol }, currentChar)
          : { type: PathConstraintType.Unconstrained };

        const mergedConstraints = mergeConstraints(currentConstraints, nextPath.constraints);

        if (mergedConstraints !== null) {
          result.push({
            path: [{ row: startRow, col: startCol }, ...nextPath.path],
            constraints: mergedConstraints
          });
        }
      }
    }
  }

  visited.delete(`${startRow}-${startCol}`);
  return result;
}

export function findAllPaths(board: Tile[][], word: string): Answer {
  const allPaths: PathWithConstraints[] = [];

  // Try starting from each position
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      const pathsFromPosition = findPathsForWordFromPosition(
        board,
        word,
        row,
        col,
        new Set()
      );
      allPaths.push(...pathsFromPosition);
    }
  }

  // Create AnswerGroupConstraintSet from all path constraints
  const constraintsSet: AnswerGroupConstraintSet = {
    pathConstraintSets: allPaths.map(pathWithConstraints => pathWithConstraints.constraints)
  };

  return {
    word,
    paths: allPaths,
    constraintsSet
  };
}

export function findAllPathsGivenConstraints(board: Tile[][], word: string, constraintSet: AnswerGroupConstraintSet = { pathConstraintSets: [] }): Position[][] {
  const paths: Position[][] = [];

  function dfs(currentPath: Position[], remainingWord: string, usedPositions: Set<string>): void {
    if (remainingWord.length === 0) {
      paths.push([...currentPath]);
      return;
    }

    const nextLetter = remainingWord[0].toLowerCase();
    const lastPos = currentPath[currentPath.length - 1];

    for (let row = 0; row < 4; row++) {
      for (let col = 0; col < 4; col++) {
        const posKey = `${row}-${col}`;
        if (usedPositions.has(posKey)) continue;

        const currentPos = { row, col };
        if (lastPos && !isAdjacent(lastPos, currentPos)) continue;

        const tile = board[row][col];
        let canUse = false;

        if (tile.isWildcard) {
          canUse = canWildcardBeUsedForLetter(currentPos, nextLetter, constraintSet);
        } else if (tile.letter.toLowerCase() === nextLetter) {
          canUse = true;
        }

        if (canUse) {
          const newUsedPositions = new Set(usedPositions);
          newUsedPositions.add(posKey);

          dfs(
            [...currentPath, currentPos],
            remainingWord.slice(1),
            newUsedPositions
          );
        }
      }
    }
  }

  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      const tile = board[row][col];
      const firstLetter = word[0].toLowerCase();
      let canStart = false;

      if (tile.isWildcard) {
        canStart = canWildcardBeUsedForLetter({ row, col }, firstLetter, constraintSet);
      } else if (tile.letter.toLowerCase() === firstLetter) {
        canStart = true;
      }

      if (canStart) {
        const usedPositions = new Set([`${row}-${col}`]);
        dfs([{ row, col }], word.slice(1), usedPositions);
      }
    }
  }

  return paths;
}

function canWildcardBeUsedForLetter(pos: Position, letter: string, constraintSet: AnswerGroupConstraintSet): boolean {
  // If no constraints exist, wildcard can be used for any letter
  if (!constraintSet.pathConstraintSets || constraintSet.pathConstraintSets.length === 0) {
    return true;
  }

  const isFirst = isFirstWildcard(pos);
  
  // Check if any constraint set allows this wildcard to be used for this letter
  return constraintSet.pathConstraintSets.some(constraint => {
    switch (constraint.type) {
      case PathConstraintType.Unconstrained:
        return true;
      
      case PathConstraintType.FirstDecided:
        if (isFirst) {
          return constraint.firstLetter?.toLowerCase() === letter.toLowerCase();
        }
        return true; // Second wildcard is not constrained
      
      case PathConstraintType.SecondDecided:
        if (!isFirst) {
          return constraint.secondLetter?.toLowerCase() === letter.toLowerCase();
        }
        return true; // First wildcard is not constrained
      
      case PathConstraintType.BothDecided:
        if (isFirst) {
          return constraint.firstLetter?.toLowerCase() === letter.toLowerCase();
        } else {
          return constraint.secondLetter?.toLowerCase() === letter.toLowerCase();
        }
      
      default:
        return false;
    }
  });
}

function scorePathByPreference(board: Tile[][], path: Position[]): PathScore {
  let wildcardCount = 0;

  for (let i = 0; i < path.length; i++) {
    const { row, col } = path[i];
    const tile = board[row][col];

    if (tile.isWildcard) {
      wildcardCount++;
    }
  }

  return {
    wildcardCount
  };
}

export function findBestPath(board: Tile[][], word: string, constraintSet: AnswerGroupConstraintSet = { pathConstraintSets: [] }): Position[] | null {
  const allPaths = findAllPathsGivenConstraints(board, word, constraintSet);

  if (allPaths.length === 0) return null;

  // Separate paths by wildcard usage
  const pathsWithoutWildcards: Position[][] = [];
  const pathsWithWildcards: Position[][] = [];

  for (const path of allPaths) {
    const score = scorePathByPreference(board, path);
    if (score.wildcardCount === 0) {
      pathsWithoutWildcards.push(path);
    } else {
      pathsWithWildcards.push(path);
    }
  }

  // If there are ANY paths without wildcards, only consider those
  const pathsToConsider = pathsWithoutWildcards.length > 0 ? pathsWithoutWildcards : pathsWithWildcards;

  pathsToConsider.sort((a, b) => {
    const scoreA = scorePathByPreference(board, a);
    const scoreB = scorePathByPreference(board, b);

    return scoreA.wildcardCount - scoreB.wildcardCount;
  });

  return pathsToConsider[0];
}

export function findPathsForHighlighting(board: Tile[][], word: string, constraintSet: AnswerGroupConstraintSet): Position[][] {
  const allPaths = findAllPaths(board, word);

  if (allPaths.paths.length === 0) return [];

  // Separate paths by wildcard usage
  const pathsWithoutWildcards: PathWithConstraints[] = [];
  const pathsWithWildcards: PathWithConstraints[] = [];

  for (const path of allPaths.paths) {
    const score = scorePathByPreference(board, path.path);
    if (score.wildcardCount === 0) {
      pathsWithoutWildcards.push(path);
    } else {
      pathsWithWildcards.push(path);
    }
  }

  // Rule 1: If ANY paths use 0 wildcards, highlight only non-wildcard paths
  if (pathsWithoutWildcards.length > 0) {
    return [pathsWithoutWildcards[0].path];
  }

  // Rule 2: Only wildcard paths exist, highlight paths that are compatible with current constraints
  const validPaths = [];
  
  // If no constraint sets exist, all paths are valid
  if (!constraintSet || !constraintSet.pathConstraintSets || constraintSet.pathConstraintSets.length === 0) {
    // this shouldn't really happen, if there is a non-wildcard option it will have returned on Rule 1
    validPaths.push(...pathsWithWildcards);
  } else {
    for (const path of pathsWithWildcards) {
      // Check if this path is compatible with any of the constraint sets
      const isCompatible = constraintSet.pathConstraintSets.some(constraintSetItem => 
        isPathCompatibleWithConstraints(path, constraintSetItem, board, word)
      );
      console.log("iscompat", isCompatible);
      
      if (isCompatible) {
        validPaths.push(path);
      }
    }
  }

  validPaths.sort((a, b) => {
    const scoreA = scorePathByPreference(board, a.path);
    const scoreB = scorePathByPreference(board, b.path);

    return scoreA.wildcardCount - scoreB.wildcardCount;
  });

  return validPaths.length > 0 ? [validPaths[0].path] : [];
}

export function isPathCompatibleWithConstraints(
  pathWithConstraints: PathWithConstraints, 
  constraintSet: PathConstraintSet, 
  board: Tile[][], 
  word: string
): boolean {
  // Check if the path's constraints can be merged with the given constraint set
  const mergedConstraints = mergeConstraints(pathWithConstraints.constraints, constraintSet);
  // If constraints cannot be merged, they are incompatible
  if (mergedConstraints === null) {
    return false;
  }
  
  // Additional validation: check that the path actually uses wildcards as specified by constraints
  for (let i = 0; i < pathWithConstraints.path.length; i++) {
    const pos = pathWithConstraints.path[i];
    console.log(i, word, pos);
    
    if (board[pos.row][pos.col].isWildcard) {
      const wordLetter = word[i].toUpperCase();
      const isFirst = isFirstWildcard(pos);
      
      // Check if wildcard usage matches the merged constraints
      if (isFirst && mergedConstraints.firstLetter) {
        if (mergedConstraints.firstLetter.toUpperCase() !== wordLetter) {
          return false;
        }
      } else if (!isFirst && mergedConstraints.secondLetter) {
        if (mergedConstraints.secondLetter.toUpperCase() !== wordLetter) {
          return false;
        }
      }
    }
  }
  
  return true;
}

export function getWildcardConstraintsFromPath(board: Tile[][], word: string, path: Position[]): PathConstraintSet | null {
  let constraints: PathConstraintSet = { type: PathConstraintType.Unconstrained };

  for (let i = 0; i < path.length; i++) {
    const { row, col } = path[i];
    const tile = board[row][col];
    let constraint: PathConstraintSet = { type: PathConstraintType.Unconstrained };

    if (tile.isWildcard) {
      if (isFirstWildcard(path[i])) {
        constraint = {
          type: PathConstraintType.FirstDecided,
          firstLetter: word[i],
        }
      }
      if (isSecondWildcard(path[i])) {
        constraint = {
          type: PathConstraintType.SecondDecided,
          secondLetter: word[i],
        }
      }
    }

    console.log(constraints, constraint)
    const newConstraints = mergeConstraints(constraints, constraint);
    if (newConstraints === null) {
      console.log("null")
      return null;
    } else {
      console.log(newConstraints);
      constraints = newConstraints;
    }
  }

  return constraints;
}

