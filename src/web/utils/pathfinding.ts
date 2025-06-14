import { Position, Tile } from './scoring';

interface PathScore {
  wildcardCount: number;
  diagonalCount: number;
  lastDiagonalIndex: number;
}

// TypeScript equivalents of backend constraint types
export enum PathConstraintType {
  Unconstrained = 'Unconstrained',
  FirstDecided = 'FirstDecided',
  SecondDecided = 'SecondDecided',
  BothDecided = 'BothDecided'
}

export interface PathConstraintSet {
  type: PathConstraintType;
  firstChar?: string;
  secondChar?: string;
}

export interface AnswerGroupConstraintSet {
  pathConstraintSets: PathConstraintSet[];
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

function isDiagonalMove(pos1: Position, pos2: Position): boolean {
  const rowDiff = Math.abs(pos1.row - pos2.row);
  const colDiff = Math.abs(pos1.col - pos2.col);
  return rowDiff === 1 && colDiff === 1;
}

function getWildcardPositions(board: Tile[][]): Position[] {
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

function isSecondWildcard(pos: Position): boolean {
  // Following backend logic: second wildcard is any wildcard that's not the first
  return !isFirstWildcard(pos);
}

function createConstraintFromWildcard(pos: Position, char: string): PathConstraintSet {
  if (isFirstWildcard(pos)) {
    return {
      type: PathConstraintType.FirstDecided,
      firstChar: char
    };
  } else {
    return {
      type: PathConstraintType.SecondDecided,
      secondChar: char
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
      if (constraint1.firstChar === constraint2.firstChar) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.SecondDecided) {
      return {
        type: PathConstraintType.BothDecided,
        firstChar: constraint1.firstChar,
        secondChar: constraint2.secondChar
      };
    } else if (constraint2.type === PathConstraintType.BothDecided) {
      if (constraint1.firstChar === constraint2.firstChar) {
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
        firstChar: constraint2.firstChar,
        secondChar: constraint1.secondChar
      };
    } else if (constraint2.type === PathConstraintType.SecondDecided) {
      if (constraint1.secondChar === constraint2.secondChar) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.BothDecided) {
      if (constraint1.secondChar === constraint2.secondChar) {
        return constraint2;
      } else {
        return null; // Unsatisfiable
      }
    }
  }

  // Handle BothDecided cases
  if (constraint1.type === PathConstraintType.BothDecided) {
    if (constraint2.type === PathConstraintType.FirstDecided) {
      if (constraint1.firstChar === constraint2.firstChar) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.SecondDecided) {
      if (constraint1.secondChar === constraint2.secondChar) {
        return constraint1;
      } else {
        return null; // Unsatisfiable
      }
    } else if (constraint2.type === PathConstraintType.BothDecided) {
      if (constraint1.firstChar === constraint2.firstChar && 
          constraint1.secondChar === constraint2.secondChar) {
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
    [0, -1],           [0, 1],
    [1, -1],  [1, 0],  [1, 1]
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

export function findAllPathsGivenWildcards(board: Tile[][], word: string, wildcardConstraints: Record<string, string> = {}): Position[][] {
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
          const constraintKey = `${row}-${col}`;
          const existingConstraint = wildcardConstraints[constraintKey];
          
          if (!existingConstraint || existingConstraint === nextLetter) {
            canUse = true;
          }
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
        const constraintKey = `${row}-${col}`;
        const existingConstraint = wildcardConstraints[constraintKey];
        
        if (!existingConstraint || existingConstraint === firstLetter) {
          canStart = true;
        }
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

function scorePathByPreference(board: Tile[][], path: Position[]): PathScore {
  let wildcardCount = 0;
  let diagonalCount = 0;
  let lastDiagonalIndex = -1;
  
  for (let i = 0; i < path.length; i++) {
    const { row, col } = path[i];
    const tile = board[row][col];
    
    if (tile.isWildcard) {
      wildcardCount++;
    }
    
    if (i > 0 && isDiagonalMove(path[i-1], path[i])) {
      diagonalCount++;
      lastDiagonalIndex = i;
    }
  }
  
  return {
    wildcardCount,
    diagonalCount,
    lastDiagonalIndex: lastDiagonalIndex === -1 ? 0 : lastDiagonalIndex
  };
}

export function findBestPath(board: Tile[][], word: string, wildcardConstraints: Record<string, string> = {}): Position[] | null {
  const allPaths = findAllPathsGivenWildcards(board, word, wildcardConstraints);
  
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
    
    if (scoreA.wildcardCount !== scoreB.wildcardCount) {
      return scoreA.wildcardCount - scoreB.wildcardCount;
    }
    
    if (scoreA.diagonalCount !== scoreB.diagonalCount) {
      return scoreA.diagonalCount - scoreB.diagonalCount;
    }
    
    return scoreB.lastDiagonalIndex - scoreA.lastDiagonalIndex;
  });
  
  return pathsToConsider[0];
}

export function findPathsForHighlighting(board: Tile[][], word: string, wildcardConstraints: Record<string, string> = {}): Position[][] {
  const allPaths = findAllPathsGivenWildcards(board, word, wildcardConstraints);
  
  if (allPaths.length === 0) return [];
  
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
  
  // Rule 1: If ANY paths use 0 wildcards, highlight only non-wildcard paths
  if (pathsWithoutWildcards.length > 0) {
    return pathsWithoutWildcards;
  }
  
  // Rule 2: Only wildcard paths exist, apply constraint minimization
  return getMinimalConstraintPaths(board, word, pathsWithWildcards);
}

interface PathAnalysis {
  path: Position[];
  wildcardAssignments: Record<string, string>;
  wildcardPositions: string[];
  wildcardCount: number;
}

function getMinimalConstraintPaths(board: Tile[][], word: string, wildcardPaths: Position[][]): Position[][] {
  if (wildcardPaths.length === 0) return [];
  
  // Analyze each path to understand wildcard usage patterns
  const pathAnalysis: PathAnalysis[] = wildcardPaths.map(path => {
    const wildcardAssignments = getWildcardConstraintsFromPath(board, word, path);
    const wildcardPositions = Object.keys(wildcardAssignments);
    
    return {
      path,
      wildcardAssignments,
      wildcardPositions,
      wildcardCount: wildcardPositions.length
    };
  });
  
  // Group paths by wildcard count - prefer fewer wildcards
  const pathsByWildcardCount: Record<number, PathAnalysis[]> = {};
  pathAnalysis.forEach(analysis => {
    const count = analysis.wildcardCount;
    if (!pathsByWildcardCount[count]) {
      pathsByWildcardCount[count] = [];
    }
    pathsByWildcardCount[count].push(analysis);
  });
  
  // Find the minimum wildcard count that has valid paths
  const minWildcardCount = Math.min(...Object.keys(pathsByWildcardCount).map(Number));
  const minimalPaths = pathsByWildcardCount[minWildcardCount];
  
  // Apply Rule 2a: Check if wildcards are necessary
  const necessaryPaths: PathAnalysis[] = [];
  
  for (const pathAnalysis of minimalPaths) {
    const { path, wildcardAssignments } = pathAnalysis;
    let pathIsNecessary = true;
    
    // For each wildcard used in this path, check if there's an alternative non-wildcard tile
    for (const [wildcardKey, letter] of Object.entries(wildcardAssignments)) {
      const [row, col] = wildcardKey.split('-').map(Number);
      const wildcardIndex = path.findIndex(pos => pos.row === row && pos.col === col);
      
      if (wildcardIndex === -1) continue;
      
      const prevPos = wildcardIndex > 0 ? path[wildcardIndex - 1] : null;
      const nextPos = wildcardIndex < path.length - 1 ? path[wildcardIndex + 1] : null;
      
      // Check if there's a non-wildcard tile with the same letter that's adjacent to both prev and next
      const hasAlternative = checkForNonWildcardAlternative(board, letter, prevPos, nextPos, path, wildcardIndex);
      
      if (hasAlternative) {
        pathIsNecessary = false;
        break;
      }
    }
    
    if (pathIsNecessary) {
      necessaryPaths.push(pathAnalysis);
    }
  }
  
  // If no paths are necessary, use all minimal paths (Rule 2b applies)
  const finalPaths = necessaryPaths.length > 0 ? necessaryPaths : minimalPaths;
  
  return finalPaths.map(analysis => analysis.path);
}

function checkForNonWildcardAlternative(board: Tile[][], letter: string, prevPos: Position | null, nextPos: Position | null, currentPath: Position[], _wildcardIndex: number): boolean {
  // Find all non-wildcard tiles with the target letter
  const alternatives: Position[] = [];
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      const tile = board[row][col];
      if (!tile.isWildcard && tile.letter.toLowerCase() === letter.toLowerCase()) {
        alternatives.push({ row, col });
      }
    }
  }
  
  // Check if any alternative can connect to both prev and next positions
  for (const alt of alternatives) {
    // Skip if this position is already used in the path
    if (currentPath.some(pos => pos.row === alt.row && pos.col === alt.col)) {
      continue;
    }
    
    let canReachPrev = !prevPos || isAdjacent(prevPos, alt);
    let canReachNext = !nextPos || isAdjacent(alt, nextPos);
    
    if (canReachPrev && canReachNext) {
      return true;
    }
  }
  
  return false;
}

export function getWildcardConstraintsFromPath(board: Tile[][], word: string, path: Position[]): Record<string, string> {
  const constraints: Record<string, string> = {};
  
  for (let i = 0; i < path.length; i++) {
    const { row, col } = path[i];
    const tile = board[row][col];
    
    if (tile.isWildcard) {
      const constraintKey = `${row}-${col}`;
      constraints[constraintKey] = word[i].toLowerCase();
    }
  }
  
  return constraints;
}

export function getWildcardNotation(board: Tile[][], wildcardConstraints: Record<string, string>, currentWord: string, highlightedPaths: Position[][], answers: string[], validAnswers: boolean[]): Record<string, string> {
  // Find wildcard positions
  const wildcardPositions: Array<{row: number, col: number, key: string}> = [];
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      if (board[row][col]?.isWildcard) {
        wildcardPositions.push({ row, col, key: `${row}-${col}` });
      }
    }
  }
  
  const notation: Record<string, string> = {};
  
  // First, analyze all valid answers to find forced constraints
  const forcedConstraints = analyzeForcedConstraints(board, answers, validAnswers, wildcardPositions);
  
  for (const wildcard of wildcardPositions) {
    const existingConstraint = wildcardConstraints[wildcard.key];
    const forcedConstraint = forcedConstraints[wildcard.key];
    
    // If there's a forced constraint from valid answers, show it
    if (forcedConstraint) {
      notation[wildcard.key] = forcedConstraint.toUpperCase();
      continue;
    }
    
    // If no current word is being typed, show existing constraint or *
    if (!currentWord || !highlightedPaths || highlightedPaths.length === 0) {
      notation[wildcard.key] = existingConstraint ? existingConstraint.toUpperCase() : '*';
      continue;
    }
    
    // Analyze current typing context
    const possibleAssignments = new Set<string>();
    
    // Check what letters this wildcard could represent in the highlighted paths
    for (const path of highlightedPaths) {
      const wildcardIndex = path.findIndex(pos => pos.row === wildcard.row && pos.col === wildcard.col);
      if (wildcardIndex !== -1 && wildcardIndex < currentWord.length) {
        possibleAssignments.add(currentWord[wildcardIndex].toLowerCase());
      }
    }
    
    // Determine notation type
    if (possibleAssignments.size === 0) {
      // Wildcard not used in current paths
      notation[wildcard.key] = existingConstraint ? existingConstraint.toUpperCase() : '*';
    } else if (possibleAssignments.size === 1) {
      const letter = Array.from(possibleAssignments)[0];
      
      // Check if other wildcards could also be this letter (constraint sharing)
      const otherWildcardsCanBeThisLetter = wildcardPositions.some(otherWildcard => {
        if (otherWildcard.key === wildcard.key) return false;
        if (forcedConstraints[otherWildcard.key]) return false; // Other wildcard is already forced
        
        for (const path of highlightedPaths) {
          const otherIndex = path.findIndex(pos => pos.row === otherWildcard.row && pos.col === otherWildcard.col);
          if (otherIndex !== -1 && otherIndex < currentWord.length && 
              currentWord[otherIndex].toLowerCase() === letter) {
            return true;
          }
        }
        return false;
      });
      
      if (otherWildcardsCanBeThisLetter) {
        notation[wildcard.key] = `${letter.toUpperCase()} / *`;
      } else {
        notation[wildcard.key] = letter.toUpperCase();
      }
    } else {
      // Multiple possible letters - show them all
      const letters = Array.from(possibleAssignments).sort();
      notation[wildcard.key] = letters.map(l => l.toUpperCase()).join(' / ');
    }
  }
  
  return notation;
}

function analyzeForcedConstraints(board: Tile[][], answers: string[], validAnswers: boolean[], _wildcardPositions: Array<{row: number, col: number, key: string}>): Record<string, string> {
  const forcedConstraints: Record<string, string> = {};
  
  if (!answers || !validAnswers) return forcedConstraints;
  
  // Build up constraints iteratively, respecting the order answers were entered
  let currentConstraints: Record<string, string> = {};
  
  for (let i = 0; i < answers.length; i++) {
    if (!validAnswers[i] || !answers[i]) continue;
    
    const word = answers[i];
    
    // Find all paths that are compatible with current constraints
    const compatiblePaths = findAllPathsGivenWildcards(board, word, currentConstraints);
    
    if (compatiblePaths.length === 0) continue;
    
    // Get the best path (this is what was actually used for this answer)
    const bestPath = findBestPath(board, word, currentConstraints);
    if (!bestPath) continue;
    
    // Extract constraints from the best path
    const pathConstraints = getWildcardConstraintsFromPath(board, word, bestPath);
    
    // Merge into current constraints
    currentConstraints = { ...currentConstraints, ...pathConstraints };
  }
  
  // The current constraints represent what each wildcard is forced to be
  return currentConstraints;
}

export function getWildcardAmbiguity(board: Tile[][], _wildcardConstraints: Record<string, string>, answers: string[], validAnswers: boolean[]): Record<string, string[] | null> {
  // Find wildcard positions
  const wildcardPositions: Array<{row: number, col: number, key: string}> = [];
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      if (board[row][col]?.isWildcard) {
        wildcardPositions.push({ row, col, key: `${row}-${col}` });
      }
    }
  }
  
  const ambiguity: Record<string, string[] | null> = {};
  
  // For each wildcard, find what letters it could represent based on current valid answers
  for (const wildcard of wildcardPositions) {
    const possibleLetters = new Set<string>();
    
    // For each valid answer, check if multiple valid paths exist that use this wildcard differently
    for (let i = 0; i < answers.length; i++) {
      if (!validAnswers[i] || !answers[i]) continue;
      
      const word = answers[i];
      
      // Find all possible paths for this word (ignoring current constraints to see alternatives)
      const allPaths = findAllPathsGivenWildcards(board, word, {});
      
      // Filter to only paths that use this specific wildcard
      const pathsUsingWildcard = allPaths.filter(path => 
        path.some(pos => pos.row === wildcard.row && pos.col === wildcard.col)
      );
      
      // For each path using this wildcard, see what letter it represents
      for (const path of pathsUsingWildcard) {
        for (let j = 0; j < path.length; j++) {
          const pos = path[j];
          if (pos.row === wildcard.row && pos.col === wildcard.col) {
            possibleLetters.add(word[j].toLowerCase());
            break;
          }
        }
      }
    }
    
    // Only show ambiguity if there are multiple possible letters
    if (possibleLetters.size > 1) {
      ambiguity[wildcard.key] = Array.from(possibleLetters).sort();
    } else {
      ambiguity[wildcard.key] = null;
    }
  }
  
  return ambiguity;
}
