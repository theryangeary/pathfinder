import { useState } from 'react';
import { ApiGameStats } from '../api/gameApi';
import { Tile } from '../utils/models';

interface HeatmapModalProps {
  isOpen: boolean;
  onClose: () => void;
  tileUsage: number[][];
  board: Tile[][];
  totalScore: number;
  scores: number[];
  gameStats: ApiGameStats | null;
}

function HeatmapModal({ isOpen, onClose, tileUsage, board, totalScore, scores, gameStats }: HeatmapModalProps) {
  const [showCopyNotification, setShowCopyNotification] = useState<boolean>(false);

  if (!isOpen) return null;

  // Get the maximum usage count to normalize the heat scale
  const maxUsage = Math.max(...tileUsage.flat(), 1);

  // Calculate the best word score
  const bestWordScore = Math.max(...scores);

  const ranking = !gameStats ? `` : `Rank: ${gameStats.user_rank} of ${gameStats.total_players}`;

  const copyToClipboard = (): void => {
    const heatmapText = board.map((row, rowIndex) =>
      row.map((_tile, colIndex) => getHeatEmoji(tileUsage[rowIndex][colIndex])).join('')
    ).join('\n');

    const currentUrl = window.location.href;

    const textToCopy = `${heatmapText}\n\nTotal Score: ${totalScore}\nBest Word: ${bestWordScore}\n${ranking}\n\n${currentUrl}`;

    navigator.clipboard.writeText(textToCopy).then(() => {
      setShowCopyNotification(true);
      setTimeout(() => setShowCopyNotification(false), 1000);
    }).catch(err => {
      console.error('Failed to copy to clipboard:', err);
    });
  };

  // Define emoji heat scale from black (0 uses) to red (max uses)
  const getHeatEmoji = (usageCount: number): string => {
    if (usageCount === 0) return '⬜️'; // Black for no usage

    const intensity = usageCount / maxUsage;

    if (intensity <= 0.2) return '🟪'; // Blue for very low usage
    if (intensity <= 0.4) return '🟦'; // Green for low usage  
    if (intensity <= 0.6) return '🟨'; // Yellow for medium usage
    if (intensity <= 0.8) return '🟧'; // Orange for high usage
    return '🟥'; // Red for maximum usage
  };

  const buttonStyle = {
    padding: '10px 20px',
    fontSize: '14px',
    border: 'none',
    borderRadius: '6px',
    cursor: 'pointer',
    transition: 'all 0.1s ease',
    transform: 'scale(1)',
    boxShadow: '0 2px 4px rgba(0,0,0,0.2)'
  };


  return (
    <div
      onClick={onClose}
      style={{
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
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          backgroundColor: 'white',
          padding: '30px',
          borderRadius: '12px',
          maxWidth: '500px',
          width: '90%',
          textAlign: 'center',
          boxShadow: '0 4px 20px rgba(0,0,0,0.3)'
        }}
      >
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
              {row.map((_tile, colIndex) =>
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

        {gameStats && (
          <div style={{
            marginBottom: '20px',
            fontSize: '16px',
            fontWeight: 'normal',
            color: '#555'
          }}>
            {ranking}
          </div>
        )}

        <div style={{
          display: 'flex',
          gap: '10px',
          justifyContent: 'center',
          flexWrap: 'wrap'
        }}>
          {/* <button
            onClick={() => {}}
            onMouseDown={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(0.95)'}
            onMouseUp={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(1)'}
            onMouseLeave={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(1)'}
            style={{
              ...buttonStyle,
              backgroundColor: '#2196F3',
              color: 'white'
            }}
          >
            Share
          </button> */}

          <button
            onClick={copyToClipboard}
            onMouseDown={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(0.95)'}
            onMouseUp={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(1)'}
            onMouseLeave={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(1)'}
            style={{
              ...buttonStyle,
              backgroundColor: '#FF9800',
              color: 'white'
            }}
          >
            Copy
          </button>

          <button
            onClick={onClose}
            onMouseDown={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(0.95)'}
            onMouseUp={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(1)'}
            onMouseLeave={(e) => (e.target as HTMLButtonElement).style.transform = 'scale(1)'}
            style={{
              ...buttonStyle,
              backgroundColor: '#757575',
              color: 'white'
            }}
          >
            Close
          </button>
        </div>

        {showCopyNotification && (
          <div style={{
            position: 'fixed',
            bottom: '20px',
            left: '50%',
            transform: 'translateX(-50%)',
            backgroundColor: '#4CAF50',
            color: 'white',
            padding: '8px 16px',
            borderRadius: '4px',
            fontSize: '14px',
            fontWeight: 'bold',
            zIndex: 1001,
            boxShadow: '0 2px 8px rgba(0,0,0,0.3)',
            animation: 'fadeInOut 1s ease-in-out'
          }}>
            Copied to clipboard!
          </div>
        )}
      </div>
    </div>
  );
}

export default HeatmapModal;
