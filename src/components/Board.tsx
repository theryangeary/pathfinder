import Tile from './Tile';
import { getWildcardNotation } from '../utils/pathfinding';
import { Position, Tile as TileType } from '../utils/scoring';

interface BoardProps {
  board: TileType[][];
  highlightedPaths: Position[][];
  wildcardConstraints: Record<string, string>;
  answers: string[];
  validAnswers: boolean[];
  currentWord: string;
}

function Board({ board, highlightedPaths, wildcardConstraints, answers, validAnswers, currentWord }: BoardProps) {
  const getWildcardDisplay = (tile: TileType): string | null => {
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
          
          // Check if this tile is the last letter in any highlighted path
          const isLastLetter = highlightedPaths.some(path => {
            if (path.length === 0) return false;
            const lastPos = path[path.length - 1];
            return lastPos.row === rowIndex && lastPos.col === colIndex;
          });
          
          return (
            <Tile
              key={`${rowIndex}-${colIndex}`}
              tile={tile}
              isHighlighted={isHighlighted}
              isLastLetter={isLastLetter}
              wildcardValue={getWildcardDisplay(tile)}
            />
          );
        })
      )}
    </div>
  );
}

export default Board;