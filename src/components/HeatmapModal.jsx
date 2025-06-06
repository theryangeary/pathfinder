import React from 'react';

function HeatmapModal({ isOpen, onClose, tileUsage, board, totalScore, scores }) {
  if (!isOpen) return null;

  // Get the maximum usage count to normalize the heat scale
  const maxUsage = Math.max(...tileUsage.flat(), 1);
  
  // Calculate the best word score
  const bestWordScore = Math.max(...scores);

  const copyToClipboard = () => {
    const heatmapText = board.map((row, rowIndex) => 
      row.map((tile, colIndex) => getHeatEmoji(tileUsage[rowIndex][colIndex])).join('')
    ).join('\n');
    
    const currentUrl = window.location.href;
    
    const textToCopy = `${heatmapText}\n\nTotal Score: ${totalScore}\nBest Word: ${bestWordScore}\n\n${currentUrl}`;
    
    navigator.clipboard.writeText(textToCopy).catch(err => {
      console.error('Failed to copy to clipboard:', err);
    });
  };

  // Define emoji heat scale from black (0 uses) to red (max uses)
  const getHeatEmoji = (usageCount) => {
    if (usageCount === 0) return '⬛'; // Black for no usage
    
    const intensity = usageCount / maxUsage;
    
    if (intensity <= 0.2) return '🟪'; // Purple for very low usage
    if (intensity <= 0.4) return '🟦'; // Blue for low usage  
    if (intensity <= 0.6) return '🟩'; // Green for medium usage
    if (intensity <= 0.8) return '🟨'; // Yellow for high usage
    return '🟥'; // Red for maximum usage
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.7)',
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      zIndex: 1000
    }}>
      <div style={{
        backgroundColor: 'white',
        padding: '30px',
        borderRadius: '12px',
        maxWidth: '500px',
        width: '90%',
        textAlign: 'center',
        boxShadow: '0 4px 20px rgba(0,0,0,0.3)'
      }}>
        <h2 style={{ marginBottom: '20px', color: '#333' }}>Nice Work!</h2>
        
        <div style={{
          fontFamily: 'monospace',
          fontSize: '24px',
          lineHeight: '1',
          margin: '0 auto 20px',
          textAlign: 'center',
          userSelect: 'text'
        }}>
          {board.map((row, rowIndex) => (
            <div key={rowIndex}>
              {row.map((tile, colIndex) => 
                getHeatEmoji(tileUsage[rowIndex][colIndex])
              ).join('')}
            </div>
          ))}
        </div>

        <div style={{ 
          marginBottom: '20px', 
          fontSize: '16px', 
          fontWeight: 'normal',
          color: '#555'
        }}>
          Total Score: {totalScore}
        </div>

        <div style={{ 
          marginBottom: '20px', 
          fontSize: '16px', 
          fontWeight: 'normal',
          color: '#555'
        }}>
          Best Word: {bestWordScore}
        </div>

        <div style={{ 
          display: 'flex', 
          gap: '10px', 
          justifyContent: 'center',
          flexWrap: 'wrap'
        }}>
          <button
            onClick={() => {/* Share functionality placeholder */}}
            style={{
              padding: '10px 20px',
              fontSize: '14px',
              backgroundColor: '#2196F3',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer'
            }}
          >
            Share
          </button>
          
          <button
            onClick={copyToClipboard}
            style={{
              padding: '10px 20px',
              fontSize: '14px',
              backgroundColor: '#FF9800',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer'
            }}
          >
            Copy
          </button>
          
          <button
            onClick={onClose}
            style={{
              padding: '10px 20px',
              fontSize: '14px',
              backgroundColor: '#757575',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer'
            }}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

export default HeatmapModal;