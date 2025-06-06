import React from 'react';

function Tile({ tile, isHighlighted, connections, wildcardValue }) {
  const displayLetter = tile.isWildcard 
    ? (wildcardValue || '*')
    : tile.letter.toUpperCase();

  // Calculate point value for non-wildcard tiles
  const getPointValue = () => {
    if (tile.isWildcard) return null;
    const letterFrequencies = {
      'a': 0.078, 'b': 0.02, 'c': 0.04, 'd': 0.038, 'e': 0.11, 'f': 0.014,
      'g': 0.03, 'h': 0.023, 'i': 0.086, 'j': 0.0021, 'k': 0.0097, 'l': 0.053,
      'm': 0.027, 'n': 0.072, 'o': 0.061, 'p': 0.028, 'q': 0.0019, 'r': 0.073,
      's': 0.087, 't': 0.067, 'u': 0.033, 'v': 0.01, 'w': 0.0091, 'x': 0.0027,
      'y': 0.016, 'z': 0.0044,
    };
    return Math.floor(Math.log2(letterFrequencies['e'] / letterFrequencies[tile.letter.toLowerCase()])) + 1;
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
        backgroundColor: isHighlighted ? '#ffeb3b' : (tile.isWildcard ? '#e0e0e0' : '#fff'),
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
      {connections && connections.map((connection, index) => (
        <div
          key={index}
          className="connection-line"
          style={{
            position: 'absolute',
            width: '2px',
            backgroundColor: '#ff5722',
            transformOrigin: 'bottom',
            zIndex: 1,
            ...connection.style
          }}
        />
      ))}
    </div>
  );
}

export default Tile;