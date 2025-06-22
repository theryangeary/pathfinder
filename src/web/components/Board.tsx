import { AnswerGroupConstraintSet, Position, Tile as TileType } from '../utils/models';
import Tile from './Tile';

interface BoardProps {
  board: TileType[][];
  highlightedPaths: Position[][];
  wildcardConstraints: AnswerGroupConstraintSet;
}

function Board({ board, highlightedPaths, wildcardConstraints}: BoardProps) {

  // Show empty tiles if board is empty
  const boardToRender = board.length === 0 
    ? Array(4).fill(null).map(() => Array(4).fill(null))
    : board;

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
      {boardToRender.map((row, rowIndex) =>
        row.map((tile, colIndex) => {
          // If tile is null (loading state), show empty tile
          if (tile === null) {
            return (
              <div
                key={`${rowIndex}-${colIndex}`}
                style={{
                  width: '60px',
                  height: '60px',
                  border: '2px solid #333',
                  borderRadius: '8px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  backgroundColor: '#fff',
                  cursor: 'default'
                }}
              />
            );
          }
          
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
              board={board}
              wildcardConstraints={wildcardConstraints}
            />
          );
        })
      )}
    </div>
  );
}

export default Board;
