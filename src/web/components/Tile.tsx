import React from 'react';
import { convertConstraintSetsToConstraints } from '../utils/constraintResolution';
import { AnswerGroupConstraintSet, Tile as TileType } from '../utils/models';
import { getLetterPoints } from '../utils/scoring';

interface TileProps {
  tile: TileType;
  isHighlighted: boolean;
  isLastLetter: boolean;
  board: TileType[][];
  wildcardConstraints: AnswerGroupConstraintSet;
}

function Tile({ tile, isHighlighted, isLastLetter, board, wildcardConstraints}: TileProps) {
  // Compute wildcard notation for this tile
  const getWildcardNotation = (): string => {
    if (!tile.isWildcard) return '';
    
    // Convert AnswerGroupConstraintSet to Record<string, string> for display
    const constraints = convertConstraintSetsToConstraints(wildcardConstraints, board);
    const constraintKey = `${tile.row}-${tile.col}`;
    return constraints[constraintKey] || '*';
  };
  
  const displayLetter = tile.isWildcard 
    ? getWildcardNotation()
    : tile.letter.toUpperCase();

  // Render wildcard notation with proper line wrapping (no trailing slashes)
  const renderWildcardContent = () => {
    if (!tile.isWildcard) {
      return displayLetter;
    }
    
    const notation = getWildcardNotation();
    const values = notation.split(' / ');
    
    if (values.length <= 2) {
      // Short enough to display on one line
      return notation;
    }
    
    // For longer notations, create elements that can wrap naturally
    // but only show slashes between items that stay on the same line
    const elements: React.ReactNode[] = [];
    
    values.forEach((value, index) => {
      if (index % 2 != 0) {
        // Add a separator that can wrap, but make slash and next value stay together
        elements.push(
          <React.Fragment key={`sep-${index}`}>
            <span style={{ whiteSpace: 'nowrap' }}> / {value}</span>
            <br />
          </React.Fragment>
        );
      } else {
        // First value, no separator needed
        elements.push(
          <span key={`val-${index}`}>{value}</span>
        );
      }
    });
    
    return (
      <div style={{ 
        textAlign: 'center', 
        lineHeight: '1.3',
        hyphens: 'none'
      }}>
        {elements}
      </div>
    );
  };


  // Calculate dynamic font size for wildcard tiles based on number of possible values
  const getFontSize = (): string => {
    if (!tile.isWildcard) return '20px';
    
    const notation = getWildcardNotation();
    const valueCount = notation.split(' / ').length;
    
    // Reduce font size by 2px for every multiple of 2 values beyond 2
    // 2 values: 20px (default)
    // 3-4 values: 18px 
    // 5-6 values: 16px, etc.
    const baseFontSize = 20;
    const reductionAmount = Math.floor(valueCount / 2) * 2;
    const finalFontSize = Math.max(baseFontSize - reductionAmount, 10); // Minimum 10px
    
    return `${finalFontSize}px`;
  };

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
        fontSize: getFontSize(),
        fontWeight: 'bold',
        backgroundColor: isLastLetter ? '#ffeb3b' : (isHighlighted ? '#f7d452' : (tile.isWildcard ? '#e0e0e0' : '#fff')),
        cursor: 'default',
        position: 'relative'
      }}
    >
      {renderWildcardContent()}
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
