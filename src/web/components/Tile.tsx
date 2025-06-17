import { getLetterPoints } from '../utils/scoring';
import { Tile as TileType } from '../utils/models';

interface TileProps {
  tile: TileType;
  isHighlighted: boolean;
  isLastLetter: boolean;
  wildcardValue: string | null;
}

function Tile({ tile, isHighlighted, isLastLetter, wildcardValue }: TileProps) {
  const displayLetter = tile.isWildcard 
    ? (wildcardValue || '*')
    : tile.letter.toUpperCase();

  // Calculate point value for non-wildcard tiles
  const getPointValue = (): number | null => {
    if (tile.isWildcard) return null;
    const letterPoints = getLetterPoints();
    return letterPoints[tile.letter.toLowerCase()];
  };

  const pointValue = getPointValue();

  return (
    <div 
      className={`tile ${isHighlighted ? 'highlighted' : ''} ${tile.isWildcard ? 'wildcard' : ''}`}
      style={{
        width: '60px',
        height: '60px',
        border: '2px solid #333',
        borderRadius: '8px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '20px',
        fontWeight: 'bold',
        backgroundColor: isLastLetter ? '#ffeb3b' : (isHighlighted ? '#f7d452' : (tile.isWildcard ? '#e0e0e0' : '#fff')),
        cursor: 'default',
        position: 'relative'
      }}
    >
      {displayLetter}
      {pointValue !== null && (
        <div
          style={{
            position: 'absolute',
            bottom: '2px',
            right: '4px',
            fontSize: '10px',
            fontWeight: 'normal',
            color: '#666'
          }}
        >
          {pointValue}
        </div>
      )}
    </div>
  );
}

export default Tile;
