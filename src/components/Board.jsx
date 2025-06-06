import React from 'react';
import Tile from './Tile';
import { getWildcardAmbiguity, getWildcardNotation } from '../utils/pathfinding';

function Board({ board, highlightedPaths, wildcardConstraints, answers, validAnswers, currentWord }) {
  const getWildcardDisplay = (tile) => {
    if (!tile.isWildcard) return null;
    
    const constraintKey = `${tile.row}-${tile.col}`;
    
    // Use the new notation system that considers current typing context
    if (board.length > 0) {
      const notation = getWildcardNotation(board, wildcardConstraints, currentWord, highlightedPaths, answers, validAnswers);
      return notation[constraintKey] || '*';
    }
    
    const constraint = wildcardConstraints[constraintKey];
    return constraint ? constraint.toUpperCase() : '*';
  };

  return (
    <div 
      className="board"
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(4, 1fr)',
        gap: '4px',
        padding: '20px',
        backgroundColor: '#f5f5f5',
        borderRadius: '12px',
        maxWidth: '300px',
        margin: '0 auto'
      }}
    >
      {board.map((row, rowIndex) =>
        row.map((tile, colIndex) => {
          const isHighlighted = highlightedPaths.some(path =>
            path.some(pos => pos.row === rowIndex && pos.col === colIndex)
          );
          
          return (
            <Tile
              key={`${rowIndex}-${colIndex}`}
              tile={tile}
              isHighlighted={isHighlighted}
              wildcardValue={getWildcardDisplay(tile)}
            />
          );
        })
      )}
    </div>
  );
}

export default Board;