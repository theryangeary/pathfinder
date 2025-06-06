import React from 'react';

function Tile({ tile, isHighlighted, connections, wildcardValue }) {
  const displayLetter = tile.isWildcard 
    ? (wildcardValue || '*')
    : tile.letter.toUpperCase();

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