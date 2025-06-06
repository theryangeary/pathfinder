import React from 'react';
import Tile from './Tile';

function Board({ board, highlightedPath, wildcardConstraints }) {
  const getWildcardDisplay = (tile) => {
    if (!tile.isWildcard) return null;
    
    const constraintKey = `${tile.row}-${tile.col}`;
    const constraint = wildcardConstraints[constraintKey];
    
    if (constraint) {
      return constraint.letter;
    }
    
    return '*';
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
          const isHighlighted = highlightedPath.some(
            pos => pos.row === rowIndex && pos.col === colIndex
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