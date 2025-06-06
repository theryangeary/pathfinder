function isAdjacent(pos1, pos2) {
  const rowDiff = Math.abs(pos1.row - pos2.row);
  const colDiff = Math.abs(pos1.col - pos2.col);
  return (rowDiff <= 1 && colDiff <= 1) && !(rowDiff === 0 && colDiff === 0);
}

function isDiagonalMove(pos1, pos2) {
  const rowDiff = Math.abs(pos1.row - pos2.row);
  const colDiff = Math.abs(pos1.col - pos2.col);
  return rowDiff === 1 && colDiff === 1;
}

function findAllPaths(board, word, wildcardConstraints = {}) {
  const paths = [];
  
  function dfs(currentPath, remainingWord, usedPositions) {
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

function scorePathByPreference(board, path) {
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

export function findBestPath(board, word, wildcardConstraints = {}) {
  const allPaths = findAllPaths(board, word, wildcardConstraints);
  
  if (allPaths.length === 0) return null;
  
  allPaths.sort((a, b) => {
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
  
  return allPaths[0];
}

export function getWildcardConstraintsFromPath(board, word, path) {
  const constraints = {};
  
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

export function getWildcardAmbiguity(board, wildcardConstraints, answers, validAnswers) {
  // Find wildcard positions
  const wildcardPositions = [];
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      if (board[row][col]?.isWildcard) {
        wildcardPositions.push({ row, col, key: `${row}-${col}` });
      }
    }
  }
  
  const ambiguity = {};
  
  // For each wildcard, collect all possible letters it could represent
  // We need to check ALL possible assignments, even if constraints exist
  for (const wildcard of wildcardPositions) {
    const possibleLetters = new Set();
    
    // For each valid answer, find ALL possible paths WITHOUT applying current constraints
    // This allows us to see if there are alternative assignments
    for (let i = 0; i < answers.length; i++) {
      if (!validAnswers[i] || !answers[i]) continue;
      
      const word = answers[i];
      // Get ALL possible paths with NO constraints to see all possibilities
      const allPaths = findAllPaths(board, word, {});
      
      // For each path, check what letters this wildcard could represent
      for (const path of allPaths) {
        for (let j = 0; j < path.length; j++) {
          const pos = path[j];
          if (pos.row === wildcard.row && pos.col === wildcard.col) {
            possibleLetters.add(word[j].toLowerCase());
            break;
          }
        }
      }
    }
    
    if (possibleLetters.size > 1) {
      ambiguity[wildcard.key] = Array.from(possibleLetters).sort();
    } else {
      ambiguity[wildcard.key] = null;
    }
  }
  
  return ambiguity;
}